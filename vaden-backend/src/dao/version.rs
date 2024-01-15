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

use crate::dao::{get_sqlite_connection, Dao};
use crate::error::VadenError;
use crate::manager::strategy::Strategy;
use serde::{Deserialize, Serialize};
use sqlx::{Connection, Row};
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone, Debug, Serialize, Deserialize, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub(crate) struct VersionName(Arc<str>);

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

impl VersionName {
    pub(crate) fn matches(&self, other: &str) -> bool {
        self.0.deref() == other
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Version {
    pub(crate) name: VersionName,
    pub(crate) web_root: PathBuf,
    pub(crate) strategy: Strategy,
}

impl Dao {
    /// Registers new version of the site in database.
    pub(crate) async fn register_version(&self, version: &Version) -> crate::error::Result<()> {
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
    pub(crate) async fn delete_version(&self, name: &str) -> crate::error::Result<String> {
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
    pub(crate) async fn list_versions(&self) -> crate::error::Result<Vec<Version>> {
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
    pub(crate) async fn save_versions(&self, versions: &[Version]) -> crate::error::Result<()> {
        let q = sqlx::query("DELETE FROM versions");
        let url = self.inner.get_read_write_url().await;
        let mut conn = get_sqlite_connection(&url).await?;
        let mut transaction = conn.begin().await.map_err(|e| VadenError::DatabaseError(e.to_string()))?;
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
}
