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

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::time::{SystemTime, UNIX_EPOCH};

use actix_web::{HttpRequest, HttpResponse, web};
use actix_web::http::StatusCode;
use actix_web::web::Data;
use awc::body::BoxBody;
use log::{debug, info};
use serde::Deserialize;
use serde_json::Value;
use web::Json;

use crate::analytics;
use crate::analytics::event_writer::EventWriter;
use crate::analytics::hash::VisitorHash;
use crate::dao::events::EventData;

pub(super) const QPACKT_EVENT_URI: &str = "/qpackt/event";

#[derive(Debug, Deserialize)]
pub(crate) struct CreateEventRequest {
    name: String,
    version: String,
    params: String,
    path: String,
    user_agent: String,
    visitor: VisitorHash,
    payload: Value,
}


/// Saves event sent from the browser.
pub(super) async fn collect_event(http: HttpRequest, Json(event): Json<CreateEventRequest>, event_writer: Data<EventWriter>) -> HttpResponse {
    debug!("Received event {:?}", event);
    let event_writer = event_writer.into_inner();
    info!("Saving event {}", event.name);
    let hash = if event.visitor.is_empty() {
        let peer = http.peer_addr().unwrap_or_else(|| SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 1))).ip();
        analytics::hash::create(peer, event.user_agent.into_bytes())
    } else {
        event.visitor
    };
    let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let payload = event.payload.to_string();
    let event = EventData {
        time,
        visitor: hash,
        name: event.name,
        version: event.version,
        params: event.params,
        path: event.path,
        payload,
    };
    event_writer.save(event).await;
    HttpResponse::new(StatusCode::OK)
}

pub(super) async fn send_event_script() -> HttpResponse {
    let content = include_str!("send_event.js");
    HttpResponse::with_body(StatusCode::OK, BoxBody::new(content))
}