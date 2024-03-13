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

use std::mem::replace;
use std::time::Duration;

use log::error;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::{Instant, timeout};

use crate::dao::Dao;
use crate::dao::events::EventData;

/// Simple actor to accept [CreateEventRequest]s that need to be written to DB to enable analytics.
#[derive(Clone)]
pub(crate) struct EventWriter {
    sender: Sender<EventData>,
}


const MAX_EVENTS_QUEUE: usize = 1024;

impl EventWriter {
    pub(crate) fn new(dao: Dao) -> Self {
        let (sender, receiver) = tokio::sync::mpsc::channel(65536);
        tokio::spawn(event_receiver(receiver, dao));
        Self { sender }
    }


    pub(crate) async fn save(&self, event: EventData) {
        if let Err(e) = self.sender.send_timeout(event, Duration::from_millis(50)).await {
            error!("Unable to log request: {}", e);
        }
    }
}

async fn event_receiver(mut receiver: Receiver<EventData>, dao: Dao) {
    let mut buffer = Vec::with_capacity(MAX_EVENTS_QUEUE);
    while let Some(request) = receiver.recv().await {
        buffer.push(request);
        let deadline = Instant::now() + Duration::from_secs(1);
        while let Ok(Some(request)) = timeout(deadline - Instant::now(), receiver.recv()).await {
            buffer.push(request);
            if buffer.len() >= MAX_EVENTS_QUEUE {
                replace_and_save(&dao, &mut buffer).await;
                break;
            }
        }
        if !buffer.is_empty() {
            replace_and_save(&dao, &mut buffer).await;
        }
    }
}

async fn replace_and_save(dao: &Dao, buffer: &mut Vec<EventData>) {
    let events = replace(buffer, Vec::with_capacity(MAX_EVENTS_QUEUE));
    if let Err(e) = dao.save_event_data(events).await {
        error!("Unable to save event data: {:?}", e);
    }
}
