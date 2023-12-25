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
use crate::manager::strategy::Strategy;
use crate::proxy::handler::CookieValue;
use awc::cookie::time::format_description::well_known::Iso8601;
use awc::cookie::time::OffsetDateTime;
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use sqlx::SqliteConnection;
use sqlx::{Connection, Row};
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Default file name with main vaden's database.
const SQLITE_FILE: &str = "vaden.sqlite";

/// DAO to encapsulate reading/writing from/to a database.
/// The basic idea is that saving/reading data is completely separated from business logic.
/// Business logic doesn't know how stuff is saved, dao doesn't know what it saves.
#[derive(Clone)]
pub(crate) struct Dao {
    inner: Arc<DaoInner>,
}

impl Dao {}

impl Dao {
    /// Initialize database (ensure app dir exists, create file, etc...)
    /// Arguments:
    ///
    /// * base_dir: path for vaden.sqlite file.
    pub(crate) async fn init(base_dir: &Path) -> Result<Self> {
        let sqlite = base_dir.join(SQLITE_FILE);
        let path = sqlite.to_str().ok_or_else(|| VadenError::DatabaseError("Non-UTF-8 file system detected".into()))?;

        let dao = Self { inner: Arc::new(DaoInner::init(path)) };
        dao.ensure_sqlite_initialized().await?;
        Ok(dao)
    }

    /// Registers new version of the site in database.
    pub(crate) async fn register_version(&self, version: &Version) -> Result<()> {
        let strategy = serde_json::to_string(&version.strategy).unwrap();
        let q = sqlx::query("INSERT INTO versions (web_root, name, strategy) VALUES ($1, $2, $3)")
            .bind(version.web_root.to_str().unwrap())
            .bind(version.name.to_string())
            .bind(&strategy);
        let url = self.inner.get_read_write_url().await;
        let mut connection = get_sqlite_connection(&url).await?;
        q.execute(&mut connection).await.map_err(|e| VadenError::DatabaseError(e.to_string()))?;
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
        let row = q.fetch_optional(&mut connection).await.map_err(|e| VadenError::DatabaseError(e.to_string()))?;
        let row = row.ok_or_else(|| VadenError::DatabaseError("No such version".into()))?;
        let path =
            row.try_get::<String, _>("web_root").map_err(|_| VadenError::DatabaseError("No column 'web_root' in versions table".into()))?;
        Ok(path)
    }

    /// Lists all versions of the site from database in alphabetical order.
    pub(crate) async fn list_versions(&self) -> Result<Vec<Version>> {
        let url = self.inner.get_read_only_url().await;
        let mut conn = get_sqlite_connection(&url).await?;
        let rows = sqlx::query("SELECT name, web_root, strategy FROM versions ORDER BY name")
            .fetch_all(&mut conn)
            .await
            .map_err(|e| VadenError::DatabaseError(e.to_string()))?;
        let mut versions = Vec::with_capacity(rows.len());
        for row in rows {
            let name =
                row.try_get::<String, _>("name").map_err(|_| VadenError::DatabaseError("No column 'name' in versions table".into()))?;
            let web_root = row
                .try_get::<String, _>("web_root")
                .map_err(|_| VadenError::DatabaseError("No column 'web_root' in versions table".into()))?;
            let web_root = PathBuf::from(web_root);
            let strategy = row
                .try_get::<String, _>("strategy")
                .map_err(|_| VadenError::DatabaseError("No column 'strategy' in versions table".into()))?;

            let strategy = serde_json::from_str::<Strategy>(&strategy)
                .map_err(|_| VadenError::DatabaseError(format!("Unable to deserialize strategy '{}' from json", strategy)))?;
            versions.push(Version { name: name.into(), web_root, strategy })
        }
        Ok(versions)
    }

    /// Saves versions to the database
    pub(crate) async fn save_versions(&self, versions: &[Version]) -> Result<()> {
        let url = self.inner.get_read_write_url().await;
        let mut conn = get_sqlite_connection(&url).await?;
        let mut transaction = conn.begin().await.map_err(|e| VadenError::DatabaseError(e.to_string()))?;
        let q = sqlx::query("DELETE FROM versions");
        q.execute(&mut *transaction).await.map_err(|e| VadenError::DatabaseError(e.to_string()))?;
        for version in versions {
            let web_root = version.web_root.to_str().unwrap();
            let strategy = serde_json::to_string(&version.strategy).unwrap();
            let q = sqlx::query("INSERT INTO versions (web_root, name, strategy) VALUES ($1, $2, $3)")
                .bind(web_root)
                .bind(version.name.to_string())
                .bind(&strategy);
            q.execute(&mut *transaction).await.map_err(|e| VadenError::DatabaseError(e.to_string()))?;
        }
        transaction.commit().await.map_err(|e| VadenError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // TODO move url and connection after query building everywhere
    /// Saves cookie and corresponding [VersionName] in the database to be read after restart.
    pub(crate) async fn save_cookie(&self, cookie: &CookieValue, version: &VersionName) -> Result<()> {
        let url = self.inner.get_read_write_url().await;
        let mut conn = get_sqlite_connection(&url).await?;
        let q = sqlx::query("INSERT INTO sessions (cookie, version, creation_time) VALUES ($1, $2, $3)")
            .bind(cookie.to_string())
            .bind(version.to_string())
            .bind(OffsetDateTime::now_utc().format(&Iso8601::DEFAULT).unwrap());
        q.execute(&mut conn).await.map_err(|e| VadenError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    /// Reads [VersionName] for given cookie.
    pub(crate) async fn read_cookie(&self, cookie: &CookieValue) -> Result<Option<VersionName>> {
        debug!("Querying database for cookie {}", cookie);
        let q = sqlx::query("SELECT version FROM sessions WHERE cookie = $1").bind(cookie.to_string());
        let url = self.inner.get_read_only_url().await;
        let mut conn = get_sqlite_connection(&url).await?;
        let row = q.fetch_optional(&mut conn).await.map_err(|e| VadenError::DatabaseError(e.to_string()))?;
        match row {
            None => {
                warn!("Version not found for cookie {}", cookie);
                Ok(None)
            }
            Some(found) => {
                let version = found.try_get::<String, _>("version").map_err(|e| VadenError::DatabaseError(e.to_string()))?;
                debug!("Found version {} for cookie {}", version, cookie);
                Ok(Some(VersionName::from(version)))
            }
        }
    }

    /// Called on startup to ensure that sqlite file exists and all migrations are applied.
    async fn ensure_sqlite_initialized(&self) -> Result<()> {
        let url = self.inner.get_read_write_url().await;
        let mut conn = get_sqlite_connection(&url).await?;
        sqlx::migrate!("db/migrations").run(&mut conn).await.map_err(|e| VadenError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct VersionName(Arc<str>);

impl From<String> for VersionName {
    fn from(value: String) -> Self {
        Self(Arc::from(value))
    }
}

impl Display for VersionName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Version {
    pub name: VersionName,
    pub web_root: PathBuf,
    pub strategy: Strategy,
}

async fn get_sqlite_connection(url: &str) -> Result<SqliteConnection> {
    SqliteConnection::connect(url).await.map_err(|e| VadenError::DatabaseError(e.to_string()))
}
