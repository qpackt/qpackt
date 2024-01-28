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

use crate::analytics::hash::VisitorHash;
use crate::dao::version::VersionName;
use crate::dao::{get_sqlite_connection, Dao};
use crate::error::{QpacktError, Result};
use log::debug;
use serde::Serialize;
use sqlx::Row;
use std::collections::HashSet;

/// Represents a user's visit. One visit may have multiple http requests, hence first/last request time and count.
#[derive(Serialize)]
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
            q.execute(&mut conn).await.map_err(|e| QpacktError::DatabaseError(e.to_string()))?;
        }
        debug!("Updated visits: {}", visits.len());
        Ok(())
    }

    /// Gets visits that happened between from_ts and to_ts
    pub(crate) async fn get_visits(&self, from_ts: u64, to_ts: u64) -> Result<Vec<Visit>> {
        debug!("Getting visits from {} to {}", from_ts as i64, to_ts as i64);
        let mut found_versions: HashSet<VersionName> = HashSet::with_capacity(1024);
        let mut visits = Vec::with_capacity(65536);
        let url = self.inner.get_read_only_url().await;
        let mut conn = get_sqlite_connection(&url).await?;
        let rows = sqlx::query(
            "SELECT first_request_time, last_request_time, request_count, visitor, version FROM visits WHERE first_request_time >= $1 AND first_request_time <= $2",
        )
        .bind(from_ts as i64)
        .bind(to_ts as i64)
        .fetch_all(&mut conn)
        .await
        .map_err(|e| QpacktError::DatabaseError(e.to_string()))?;
        for row in rows {
            let first_request_time = row
                .try_get::<i64, _>("first_request_time")
                .map_err(|_| QpacktError::DatabaseError("No column 'first_request_time' in visits table".into()))?;
            let last_request_time = row
                .try_get::<i64, _>("last_request_time")
                .map_err(|_| QpacktError::DatabaseError("No column 'last_request_time' in visits table".into()))?;
            let request_count = row
                .try_get::<u32, _>("request_count")
                .map_err(|_| QpacktError::DatabaseError("No column 'request_count' in visits table".into()))?;
            let visitor =
                row.try_get::<i64, _>("visitor").map_err(|_| QpacktError::DatabaseError("No column 'visitor' in visits table".into()))?;
            let version = row
                .try_get::<&str, _>("version")
                .map_err(|_| QpacktError::DatabaseError("No column 'version' in versions table".into()))?;
            // We don't want a new String for every visit. So let's find the right [VersionName] that's cheap to clone.
            // If not found then create one.
            let mut found_version: Option<VersionName> = None;
            for found in &found_versions {
                if found.matches(version) {
                    found_version = Some(found.clone());
                    break;
                }
            }
            let version = found_version.unwrap_or_else(|| {
                let v = VersionName::from(version.to_string());
                found_versions.insert(v.clone());
                v
            });
            visits.push(Visit {
                first_request_time: first_request_time as u64,
                last_request_time: last_request_time as u64,
                request_count,
                visitor: visitor.into(),
                version,
            })
        }
        debug!("Returned {} visits", visits.len());
        Ok(visits)
    }
}
