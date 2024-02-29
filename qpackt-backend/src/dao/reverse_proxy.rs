// SPDX-License-Identifier: AGPL-3.0
/*
   qpackt: Web & Analytics Server
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
use crate::error::{QpacktError, Result};
use crate::reverse_proxy::ReverseProxy;
use sqlx::Row;

impl Dao {
    pub(crate) async fn list_reverse_proxies(&self) -> Result<Vec<ReverseProxy>> {
        let url = self.inner.get_read_only_url().await;
        let mut conn = get_sqlite_connection(&url).await?;
        let rows = sqlx::query("SELECT id, prefix, target FROM reverse_proxy ORDER BY prefix DESC")
            .fetch_all(&mut conn)
            .await
            .map_err(|e| QpacktError::DatabaseError(e.to_string()))?;
        let mut proxies = Vec::with_capacity(rows.len());
        for row in rows {
            let id = row.try_get::<i32, _>("id").map_err(|_| QpacktError::DatabaseError("No column 'id' in reverse_proxy table".into()))?;
            let prefix = row
                .try_get::<String, _>("prefix")
                .map_err(|_| QpacktError::DatabaseError("No column 'prefix' in reverse_proxy table".into()))?;
            let target = row
                .try_get::<String, _>("target")
                .map_err(|_| QpacktError::DatabaseError("No column 'target' in reverse_proxy table".into()))?;
            proxies.push(ReverseProxy { id, prefix, target })
        }
        Ok(proxies)
    }

    pub(crate) async fn create_reverse_proxy(&self, prefix: &str, target: &str) -> Result<()> {
        let url = self.inner.get_read_write_url().await;
        let mut conn = get_sqlite_connection(&url).await?;
        sqlx::query("INSERT INTO reverse_proxy (prefix, target) VALUES ($1, $2)")
            .bind(prefix)
            .bind(target)
            .execute(&mut conn)
            .await
            .map_err(|e| QpacktError::DatabaseError(format!("Unable to insert reverse_proxy: {}", e)))?;
        Ok(())
    }

    pub(crate) async fn delete_reverse_proxy(&self, id: i32) -> Result<()> {
        let url = self.inner.get_read_write_url().await;
        let mut conn = get_sqlite_connection(&url).await?;
        sqlx::query("DELETE FROM reverse_proxy WHERE id = $1")
            .bind(id)
            .execute(&mut conn)
            .await
            .map_err(|e| QpacktError::DatabaseError(format!("Unable to delete reverse_proxy `{}`: {}", id, e)))?;
        Ok(())
    }
}
