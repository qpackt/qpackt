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

use crate::dao::requests::DailySeed;
use crate::dao::{get_sqlite_connection, Dao};
use crate::error::{Result, VadenError};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqliteRow;
use sqlx::{Connection, Row};

pub(crate) trait State: Serialize + for<'a> Deserialize<'a> {
    fn name() -> &'static str;
}

impl Dao {
    /// Gets state from DB. Returns `Ok(None)` if no such value.
    /// Returns error when unable to read DB or unable to deserialize.
    pub(crate) async fn get_state<T: State>(&self, name: &str) -> Result<Option<T>> {
        let row = self.read_row(name).await?;
        match row {
            None => Ok(None),
            Some(row) => deserialize_row(row),
        }
    }

    /// Writes state to DB. In one transaction:
    /// - removes value from DB
    /// - inserts new value (serialized)
    /// Return Err when unable to talk to DB or serialize value.
    pub(crate) async fn set_state<T: State>(&self, state: &T) -> Result<()> {
        let name = T::name();
        debug!("Saving state for {}", name);
        let serialized = serde_json::to_string(state).map_err(|e| {
            error!("Unable to serialize state {}: {}", name, e);
            VadenError::SerializationError
        })?;
        let q = sqlx::query("DELETE FROM state WHERE name = $1").bind(name);
        let url = self.inner.get_read_write_url().await;
        let mut conn = get_sqlite_connection(&url).await?;
        let mut transaction = conn.begin().await.map_err(|e| VadenError::DatabaseError(e.to_string()))?;
        q.execute(&mut *transaction).await.map_err(|e| VadenError::DatabaseError(e.to_string()))?;
        let q = sqlx::query("INSERT INTO state (name, value) VALUES ($1, $2)").bind(name).bind(serialized);
        q.execute(&mut *transaction).await.map_err(|e| VadenError::DatabaseError(e.to_string()))?;
        transaction.commit().await.map_err(|e| VadenError::DatabaseError(e.to_string()))?;
        info!("Saved state for {}", name);
        Ok(())
    }

    async fn read_row(&self, name: &str) -> Result<Option<SqliteRow>> {
        let url = self.inner.get_read_only_url().await;
        let mut connection = get_sqlite_connection(&url).await?;
        let state = sqlx::query("SELECT value FROM state WHERE name = $1")
            .bind(name)
            .fetch_optional(&mut connection)
            .await
            .map_err(|e| VadenError::DatabaseError(e.to_string()))?;
        Ok(state)
    }
}

fn deserialize_row<T: State>(row: SqliteRow) -> Result<Option<T>> {
    let value = row.try_get::<String, _>("value").map_err(|_| VadenError::DatabaseError("No column 'value' in `state` table".into()))?;
    match serde_json::from_str(&value) {
        Ok(v) => Ok(Some(v)),
        Err(e) => {
            error!("Unable to deserialize {} from value in DB: `{}`: {}", T::name(), value, e);
            Err(VadenError::SerializationError)
        }
    }
}

impl State for DailySeed {
    fn name() -> &'static str {
        "DailySeed"
    }
}
