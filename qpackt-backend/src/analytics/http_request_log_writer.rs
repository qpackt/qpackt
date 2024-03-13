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
use std::mem::replace;
use std::time::Duration;

use log::error;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::{Instant, timeout};

use crate::dao::Dao;
use crate::dao::requests::CreateHttpRequestLog;
use crate::dao::visits::Visit;

/// Simple actor to accept [CreateHttpRequestLog]s that need to be written to DB to enable analytics.
#[derive(Clone)]
pub(crate) struct HttpRequestLogWriter {
    sender: Sender<CreateHttpRequestLog>,
}

/// Buffer length for [CreateHttpRequestLog]s before saving to DB.
const MAX_REQUESTS: usize = 1024;

impl HttpRequestLogWriter {
    /// Creates new [HttpRequestLogWriter] and starts background thread for actually saving requests to DB.
    pub(crate) fn new(dao: Dao) -> Self {
        let (sender, receiver) = tokio::sync::mpsc::channel(65536);
        tokio::spawn(request_receiver(receiver, dao));
        Self { sender }
    }

    /// Accepts [CreateHttpRequestLog] that need to be saved to DB for analytics.
    pub(crate) async fn save(&self, request: CreateHttpRequestLog) {
        // Send request to an internal channel. This timeout needs to be fairly short as it will block the client's http request.
        // It will only fail if the channel's buffer is full, which will only happen if DB can't catch up.
        if let Err(e) = self.sender.send_timeout(request, Duration::from_millis(50)).await {
            error!("Unable to log request: {}", e);
        }
    }
}

/// Starts a receiver loop. Once a [CreateHttpRequestLog] is received is waits max 1 second for more requests and
/// then saves them to DB.
async fn request_receiver(mut receiver: Receiver<CreateHttpRequestLog>, dao: Dao) {
    let mut buffer = Vec::with_capacity(MAX_REQUESTS);
    while let Some(request) = receiver.recv().await {
        buffer.push(request);
        let deadline = Instant::now() + Duration::from_secs(1);
        while let Ok(Some(request)) = timeout(deadline - Instant::now(), receiver.recv()).await {
            buffer.push(request);
            if buffer.len() >= MAX_REQUESTS {
                save_requests(&dao, &mut buffer).await;
                break;
            }
        }
        if !buffer.is_empty() {
            save_requests(&dao, &mut buffer).await;
        }
    }
}

/// Calls [Dao] to save [CreateHttpRequestLog]s to DB. Replaces buffer with new one.
async fn save_requests(dao: &Dao, buffer: &mut Vec<CreateHttpRequestLog>) {
    let requests = replace(buffer, Vec::with_capacity(MAX_REQUESTS));
    if let Err(e) = dao.save_requests(&requests).await {
        error!("Unable to save requests to DB: {}", e);
    }
    let visits = merge_requests(requests);
    if let Err(e) = dao.update_visits(&visits).await {
        error!("Unable to update visits in DB: {}", e);
    }
}

/// 'Merges' [CreateHttpRequestLog]s into separate [Visit]s so that they can be shown in analytics.
/// Uses [VisitorHash] to recognize requests from the same client.
fn merge_requests(requests: Vec<CreateHttpRequestLog>) -> Vec<Visit> {
    let mut visits = HashMap::with_capacity(requests.len());
    for r in requests {
        let visit = visits.entry(r.visitor).or_insert_with(|| Visit {
            first_request_time: r.time,
            last_request_time: r.time,
            request_count: 0,
            visitor: r.visitor,
            version: r.version,
        });
        visit.request_count += 1;
        visit.last_request_time = r.time;
    }
    visits.into_values().collect()
}
