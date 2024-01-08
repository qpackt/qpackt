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

use crate::analytics::hash::VisitorHash;
use crate::dao::version::VersionName;
use crate::dao::{get_sqlite_connection, Dao};
use crate::error::{Result, VadenError};
use log::debug;

/// Represents a user's visit. One visit may have multiple http requests, hence first/last request time and count.
pub(crate) struct Visit {
    pub(crate) first_request_time: u64,
    pub(crate) last_request_time: u64,
    pub(crate) request_count: u32,
    pub(crate) visitor: VisitorHash,
    pub(crate) version: VersionName,
}

impl Dao {
    /// Updates [Visit]s in DB.
    /// Increases request count and last_request_time for each visitor
    /// or creates a new one with new data when a visitor's hash not found.
    pub(crate) async fn update_visits(&self, visits: &[Visit]) -> Result<()> {
        debug!("Updating visits: {}", visits.len());
        let url = self.inner.get_read_write_url().await;
        let mut conn = get_sqlite_connection(&url).await?;
        for visit in visits {
            let q = sqlx::query(
                "INSERT INTO visits (first_request_time, last_request_time, request_count, visitor, version) values ($1, $2, $3, $4, $5) \
                    ON CONFLICT(visitor) DO UPDATE SET request_count=request_count + $3, last_request_time=$2",
            )
            .bind(visit.first_request_time as i64)
            .bind(visit.last_request_time as i64)
            .bind(visit.request_count)
            .bind::<i64>(visit.visitor.into())
            .bind(visit.version.to_string());
            q.execute(&mut conn).await.map_err(|e| VadenError::DatabaseError(e.to_string()))?;
        }
        debug!("Updated visits: {}", visits.len());
        Ok(())
    }
}
