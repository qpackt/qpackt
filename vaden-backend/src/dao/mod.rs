// SPDX-License-Identifier: AGPL-3.0
/*
   Vaden: Versioned Application Deployment Engine
   Copyright (C) 2023 Łukasz Wojtów

   This program is free software: you can redistribute it and/or modify
   it under the terms of the GNU Affero General Public License as
   published by the Free Software Foundation, either version 3 of the
   License.

   This program is distributed in the hope that it will be useful,
   but WITHOUT ANY WARRANTY; without even the implied warranty of
   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
   GNU Affero General Public License for more details.

   You should have received a copy of the GNU Affero General Public License
   along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

mod inner;

use crate::dao::inner::DaoInner;
use crate::error::{Result, VadenError};
use sqlx::SqliteConnection;
use sqlx::{Connection, Row};
use std::path::Path;
use std::sync::Arc;

/// Default file name with main vaden's database.
const SQLITE_FILE: &str = "vaden.sqlite";

/// DAO to encapsulate reading/writing from/to a database.
#[derive(Clone)]
pub(crate) struct Dao {
    inner: Arc<DaoInner>,
}

impl Dao {
    /// Initialize database (ensure app dir exists, create file, etc...)
    /// Arguments:
    ///
    /// * base_dir: path for vaden.sqlite file.
    pub(crate) async fn init(base_dir: &Path) -> Result<Self> {
        let sqlite = base_dir.join(SQLITE_FILE);
        let path = sqlite
            .to_str()
            .ok_or_else(|| VadenError::DatabaseError("Non-UTF-8 file system detected".into()))?;

        let dao = Self {
            inner: Arc::new(DaoInner::init(path)),
        };
        dao.ensure_sqlite_initialized().await?;
        Ok(dao)
    }

    /// Registers new version of the site in database.
    ///
    /// Args:
    /// * web_root - full path to where the files are stored
    /// * name - name of the version
    pub(crate) async fn register_version(&self, web_root: &str, name: &str) -> Result<()> {
        let q = sqlx::query("INSERT INTO versions (web_root, name) VALUES ($1, $2)")
            .bind(web_root)
            .bind(name);
        let url = self.inner.get_read_write_url().await;
        let mut connection = get_sqlite_connection(&url).await?;
        q.execute(&mut connection)
            .await
            .map_err(|e| VadenError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    /// Delete version from the database and return its path so it can be deleted from the filesystem as well.
    /// Arguments:
    ///
    /// - name: name of the version
    pub(crate) async fn delete_version(&self, name: &str) -> Result<String> {
        let q = sqlx::query("DELETE FROM versions WHERE name = $1 RETURNING web_root").bind(name);
        let url = self.inner.get_read_write_url().await;
        let mut connection = get_sqlite_connection(&url).await?;
        let row = q
            .fetch_optional(&mut connection)
            .await
            .map_err(|e| VadenError::DatabaseError(e.to_string()))?;
        let row = row.ok_or_else(|| VadenError::DatabaseError("No such version".into()))?;
        let path = row.try_get::<String, _>("web_root").map_err(|_| {
            VadenError::DatabaseError("No column 'web_root' in versions table".into())
        })?;
        Ok(path)
    }

    /// Lists all versions of the site from database in alphabetical order.
    pub(crate) async fn list_versions(&self) -> Result<Vec<Version>> {
        let url = self.inner.get_read_only_url().await;
        let mut conn = get_sqlite_connection(&url).await?;
        let rows = sqlx::query_as::<_, Version>("SELECT name FROM versions ORDER BY name")
            .fetch_all(&mut conn)
            .await
            .map_err(|e| VadenError::DatabaseError(e.to_string()))?;
        Ok(rows)
    }

    /// Called on startup to ensure that sqlite file exists and all migrations are applied.
    async fn ensure_sqlite_initialized(&self) -> Result<()> {
        let url = self.inner.get_read_write_url().await;
        let mut conn = get_sqlite_connection(&url).await?;
        sqlx::migrate!("db/migrations")
            .run(&mut conn)
            .await
            .map_err(|e| VadenError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct Version {
    pub name: String,
}

async fn get_sqlite_connection(url: &str) -> Result<SqliteConnection> {
    SqliteConnection::connect(url)
        .await
        .map_err(|e| VadenError::DatabaseError(e.to_string()))
}
