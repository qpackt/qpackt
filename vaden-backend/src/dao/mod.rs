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

use crate::error::{Result, VadenError};
use sqlx::Connection;
use sqlx::SqliteConnection;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

/// Default file name with main vaden's database.
const SQLITE_FILE: &str = "vaden.sqlite";

/// DAO to encapsulate reading/writing from/to a database.
#[derive(Clone)]
pub(crate) struct Dao {
    inner: Arc<DaoInner>,
}

struct DaoInner {
    rw_url: RwLock<String>,
    ro_url: RwLock<String>,
}

impl DaoInner {
    async fn get_read_only_url(&self) -> RwLockReadGuard<'_, String> {
        self.ro_url.read().await
    }

    async fn get_read_write_url(&self) -> RwLockWriteGuard<'_, String> {
        self.rw_url.write().await
    }
}

impl Dao {
    pub(crate) async fn init(base_dir: &Path) -> Result<Self> {
        ensure_app_dir_exists(base_dir)?;
        let sqlite = base_dir.join(SQLITE_FILE);
        let path = sqlite
            .to_str()
            .ok_or_else(|| VadenError::DatabaseError("Non-UTF-8 file system detected".into()))?;
        let rw_url = format!("sqlite://{path}?mode=rwc");
        let ro_url = format!("sqlite://{path}?mode=r");
        let dao = Self {
            inner: Arc::new(DaoInner {
                rw_url: RwLock::new(rw_url),
                ro_url: RwLock::new(ro_url),
            }),
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

// TODO move to more general module, this has nothing to do with dao
fn ensure_app_dir_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        create_app_dir(path)
    } else if !path.is_dir() {
        Err(VadenError::InvalidConfig(format!(
            "App dir is not a directory: {path:?}"
        )))
    } else {
        Ok(())
    }
}

fn create_app_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path).map_err(|e| {
        VadenError::InvalidConfig(format!(
            "Unable to create app directory {}: {}",
            path.to_string_lossy(),
            e
        ))
    })
}

async fn get_sqlite_connection(url: &str) -> Result<SqliteConnection> {
    SqliteConnection::connect(url)
        .await
        .map_err(|e| VadenError::DatabaseError(e.to_string()))
}
