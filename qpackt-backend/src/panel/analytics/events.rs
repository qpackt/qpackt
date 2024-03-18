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


use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use actix_web::{HttpRequest, HttpResponse, Responder, web};
use actix_web::http::StatusCode;
use actix_web::web::{Bytes, Data, Json};
use chrono::DateTime;
use futures::Stream;
use log::{error, warn};
use serde::Serialize;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::mpsc::error::TrySendError;
use tokio::time::sleep;

use crate::dao::Dao;
use crate::dao::events::{EventName, GetEventsFilter, SavedEventData};
use crate::dao::version::VersionName;
use crate::error::QpacktError;
use crate::error::Result;
use crate::panel::analytics::DateRange;
use crate::panel::validate_permission;

#[derive(Serialize)]
struct VersionVisitCount {
    version: VersionName,
    count: u64,
}

#[derive(Serialize)]
struct VersionEventPercent {
    version: VersionName,
    percent: f32,
}

#[derive(Serialize)]
struct EventPercentCounts {
    event: EventName,
    percents: Vec<VersionEventPercent>,
}

#[derive(Serialize)]
pub(crate) struct EventsStats {
    events_percent_list: Vec<EventPercentCounts>,
}


pub(crate) async fn get_events_stats(http: HttpRequest, filter: web::Query<DateRange>, dao: Data<Dao>) -> Result<impl Responder> {
    validate_permission(&http)?;
    let stats = dao.get_events_stats(GetEventsFilter { time_from: filter.from_time.timestamp() as u64, time_to: filter.to_time.timestamp() as u64 }).await?;
    let version_visit_counts = stats.total_visit_count.into_iter().map(|(version, count)| VersionVisitCount { version, count }).collect::<Vec<_>>();
    let mut events_percent_list = Vec::with_capacity(stats.event_version_count.len());
    for (event, count_map) in stats.event_version_count {
        let mut epc = EventPercentCounts { event, percents: Vec::with_capacity(version_visit_counts.len()) };
        for version_visit_count in &version_visit_counts {
            let percent = 100.0 * *count_map.get(&version_visit_count.version).unwrap_or(&0) as f32 / version_visit_count.count as f32;
            epc.percents.push(VersionEventPercent { version: version_visit_count.version.clone(), percent });
        }
        events_percent_list.push(epc);
    }
    events_percent_list.sort_by(|e1, e2| e1.event.cmp(&e2.event));
    let stats = EventsStats {
        events_percent_list,
    };
    Ok(Json(stats))
}

pub(crate) async fn get_events_csv(http: HttpRequest, filter: web::Query<DateRange>, dao: Data<Dao>) -> Result<impl Responder> {
    validate_permission(&http)?;
    let mut response = HttpResponse::build(StatusCode::OK);
    response.append_header(("Content-type", "text/csv"));
    response.append_header(("Content-disposition", "attachment; filename=events.csv"));
    let (response_sender, response_receiver) = channel(65536);
    let (dao_sender, dao_receiver) = channel(65536);
    let filter = GetEventsFilter { time_from: filter.from_time.timestamp() as u64, time_to: filter.to_time.timestamp() as u64 };
    tokio::spawn(get_events_db(dao, filter, dao_sender));
    tokio::spawn(map_to_csv(dao_receiver, response_sender));
    let response_stream = ResponseStream { receiver: response_receiver };
    Ok(response.streaming(response_stream))
}


async fn get_events_db(dao: Data<Dao>, filter: GetEventsFilter, dao_sender: Sender<SavedEventData>) {
    if let Err(e) = dao.get_events(filter, dao_sender).await {
        error!("Unable to get events from db: {}", e);
    }
}

type ResponseItem = std::result::Result<Bytes, QpacktError>;

async fn map_to_csv(mut dao_receiver: Receiver<SavedEventData>, response_sender: Sender<ResponseItem>) {
    let line = "id,time,event,version,visitor,params,path,payload\r\n";
    response_sender.send(Ok(Bytes::from(line))).await.unwrap();
    while let Some(event) = dao_receiver.recv().await {
        let time = DateTime::from_timestamp(event.event.time as i64, 0).unwrap();
        let time = time.format("%Y-%m-%d %H:%M");
        let line = format!("{},{},{},{},{},{},{},{}\r\n",
                           event.id, time, event.event.name, event.event.version, event.event.visitor, event.event.params, event.event.path, event.event.payload
        );
        let mut m_bytes = Some(Ok(Bytes::from(line)));
        while let Some(bytes) = m_bytes.take() {
            if let Err(TrySendError::Full(item)) = response_sender.try_send(bytes) {
                m_bytes = Some(item);
                warn!("http receiver too slow!");
                sleep(Duration::from_millis(1)).await;
            }
        }
    }
}

struct ResponseStream {
    receiver: Receiver<ResponseItem>,
}

impl Stream for ResponseStream {
    type Item = ResponseItem;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.receiver.poll_recv(cx)
    }
}