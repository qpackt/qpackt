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
pub(crate) mod requests;
mod state;
pub(crate) mod version;
pub(crate) mod visits;

use crate::dao::inner::DaoInner;
use crate::error::{Result, VadenError};
use sqlx::Connection;
use sqlx::SqliteConnection;
use std::path::Path;
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

    /// Called on startup to ensure that sqlite file exists and all migrations are applied.
    async fn ensure_sqlite_initialized(&self) -> Result<()> {
        let url = self.inner.get_read_write_url().await;
        let mut conn = get_sqlite_connection(&url).await?;
        sqlx::migrate!("db/migrations").run(&mut conn).await.map_err(|e| VadenError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}

async fn get_sqlite_connection(url: &str) -> Result<SqliteConnection> {
    SqliteConnection::connect(url).await.map_err(|e| VadenError::DatabaseError(e.to_string()))
}
