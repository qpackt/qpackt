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


use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use sqlx::{Connection, Row};
use sqlx::sqlite::SqliteRow;
use tokio::sync::mpsc::error::TrySendError;
use tokio::sync::mpsc::Sender;
use tokio::time::sleep;

use crate::analytics::hash::VisitorHash;
use crate::dao::{Dao, get_sqlite_connection};
use crate::dao::version::VersionName;
use crate::error::{QpacktError, Result};

pub(crate) struct EventData {
    pub(crate) time: u64,
    pub(crate) visitor: VisitorHash,
    pub(crate) version: String,
    pub(crate) name: String,
    pub(crate) params: String,
    pub(crate) path: String,
    pub(crate) payload: String,
}

pub(crate) struct SavedEventData {
    pub(crate) id: i64,
    pub(crate) event: EventData,
}

#[derive(Debug, Deserialize)]
pub(crate) struct GetEventsFilter {
    pub(crate) time_from: u64,
    pub(crate) time_to: u64,
}

impl Dao {
    pub(crate) async fn save_event_data(&self, events: Vec<EventData>) -> Result<()> {
        let len = events.len();
        debug!("Saving {} events", len);
        let url = self.inner.get_read_write_url().await;
        let mut conn = get_sqlite_connection(&url).await?;
        for event in events {
            let q = sqlx::query("INSERT INTO events (time, visitor, version, name, params, path, payload) values ($1, $2, $3, $4, $5, $6, $7)")
                .bind(event.time as i64)
                .bind::<i64>(event.visitor.into())
                .bind(event.version)
                .bind(event.name)
                .bind(event.params)
                .bind(event.path)
                .bind(serde_json::to_string(&event.payload).unwrap());
            q.execute(&mut conn).await.map_err(|e| QpacktError::DatabaseError(e.to_string()))?;
        }
        info!("Saved {} events", len);
        Ok(())
    }

    pub(crate) async fn get_events_stats(&self, filter: GetEventsFilter) -> Result<EventStats> {
        let q = sqlx::query("SELECT COUNT(DISTINCT(visitor)) AS total_visits, version
                                                            FROM visits
                                                            WHERE first_request_time >= $1 AND first_request_time < $2
                                                            GROUP BY version
                                                            ORDER BY version")
            .bind(filter.time_from as i64)
            .bind(filter.time_to as i64);
        let url = self.inner.get_read_only_url().await;
        let mut conn = get_sqlite_connection(&url).await?;
        let mut transaction = conn.begin().await.map_err(|e| QpacktError::DatabaseError(e.to_string()))?;
        let rows = q.fetch_all(&mut *transaction).await.map_err(|e| QpacktError::DatabaseError(e.to_string()))?;
        let mut total_visit_count: Vec<(VersionName, u64)> = vec![];
        for row in rows {
            let total_visits = row
                .try_get::<i64, _>("total_visits")
                .map_err(|_| QpacktError::DatabaseError("Unable to get total_visits from visits table".into()))?;
            let version = row
                .try_get::<&str, _>("version")
                .map_err(|_| QpacktError::DatabaseError("No column 'version' in visits table".into()))?;
            total_visit_count.push((version.to_string().into(), total_visits as u64));
        }
        let q = sqlx::query("SELECT id, time, visitor, version, name, params, path, payload FROM events WHERE time >= $1 AND time < $2")
            .bind(filter.time_from as i64)
            .bind(filter.time_to as i64);
        let rows = q.fetch_all(&mut *transaction).await.map_err(|e| QpacktError::DatabaseError(e.to_string()))?;
        let mut event_version_count = HashMap::new();
        for row in rows {
            let event = saved_event_from_row(row)?;
            let version_map = event_version_count.entry(event.event.name.into()).or_insert_with(HashMap::new);
            let count = version_map.entry(event.event.version.into()).or_insert(0);
            *count += 1;
        }

        Ok(EventStats {
            total_visit_count,
            event_version_count,
        })
    }

    pub(crate) async fn get_events(&self, filter: GetEventsFilter, sender: Sender<SavedEventData>) -> Result<()> {
        let q = sqlx::query("SELECT id, time, visitor, version, name, params, path, payload FROM events WHERE time >= $1 AND time < $2")
            .bind(filter.time_from as i64)
            .bind(filter.time_to as i64);
        let url = self.inner.get_read_only_url().await;
        let mut conn = get_sqlite_connection(&url).await?;
        let rows = q.fetch_all(&mut conn).await.map_err(|e| QpacktError::DatabaseError(e.to_string()))?;
        for row in rows {
            let saved_event = saved_event_from_row(row)?;
            let mut m_event = Some(saved_event);
            while let Some(event) = m_event.take() {
                if let Err(TrySendError::Full(e)) = sender.try_send(event) {
                    warn!("Sending event from database too slow");
                    sleep(Duration::from_millis(1)).await;
                    m_event = Some(e);
                    continue;
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Serialize)]
pub(crate) struct EventName(Arc<str>);

impl From<String> for EventName {
    fn from(value: String) -> Self {
        Self(Arc::from(value))
    }
}


#[derive(Debug)]
pub(crate) struct EventStats {
    pub(crate) total_visit_count: Vec<(VersionName, u64)>,
    pub(crate) event_version_count: HashMap<EventName, HashMap<VersionName, u64>>,
}

fn saved_event_from_row(row: SqliteRow) -> Result<SavedEventData> {
    let id = row
        .try_get::<i64, _>("id")
        .map_err(|_| QpacktError::DatabaseError("No column 'id' in visits table".into()))?;
    let time = row
        .try_get::<i64, _>("time")
        .map_err(|_| QpacktError::DatabaseError("No column 'time' in visits table".into()))?;
    let visitor = row
        .try_get::<i64, _>("visitor")
        .map_err(|_| QpacktError::DatabaseError("No column 'visitor' in visits table".into()))?;
    let version = row
        .try_get::<&str, _>("version")
        .map_err(|_| QpacktError::DatabaseError("No column 'version' in versions table".into()))?;
    let name = row
        .try_get::<&str, _>("name")
        .map_err(|_| QpacktError::DatabaseError("No column 'name' in versions table".into()))?;
    let params = row
        .try_get::<&str, _>("params")
        .map_err(|_| QpacktError::DatabaseError("No column 'params' in versions table".into()))?;
    let path = row
        .try_get::<&str, _>("path")
        .map_err(|_| QpacktError::DatabaseError("No column 'path' in versions table".into()))?;
    let payload = row
        .try_get::<&str, _>("payload")
        .map_err(|_| QpacktError::DatabaseError("No column 'payload' in versions table".into()))?;
    let saved_event = SavedEventData {
        id,
        event: EventData {
            time: time as u64,
            visitor: visitor.into(),
            version: version.to_string(),
            name: name.to_string(),
            params: params.to_string(),
            path: path.to_string(),
            payload: payload.to_string(),
        },
    };
    Ok(saved_event)
}
