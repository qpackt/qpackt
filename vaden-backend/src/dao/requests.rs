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

//! Contains structs and methods to CRUD users' requests.
//! This data is later used in analytics.

use crate::analytics::hash::VisitorHash;
use crate::dao::state::State;
use crate::dao::version::VersionName;
use crate::dao::Dao;
use crate::error::Result;
use actix_web::http::Uri;
use log::debug;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::Instant;

/// Daily seed used to creating visitors' hashes ([VisitorHash]).
/// It isn't really used every time a new hash is needed. Instead, [crate::analytics::hash::CURRENT_INIT] in hash module is used.
#[derive(Serialize, Deserialize)]
pub(crate) struct DailySeed {
    pub(crate) init: u64,
    pub(crate) expiration: SystemTime,
}

#[derive(Debug)]
pub(crate) struct Request {
    time: u64,
    visitor: VisitorHash,
    version: VersionName,
    uri: Uri,
}

impl Request {
    pub(crate) fn new(visitor: VisitorHash, version: VersionName, uri: Uri) -> Self {
        Self { time: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(), visitor, version, uri }
    }
}

impl Dao {
    /// Gets daily seed from database. Used to generate [VisitorHash](crate::analytics::VisitorHash).
    pub(crate) async fn get_daily_seed(&self) -> Result<Option<DailySeed>> {
        self.get_state(DailySeed::name()).await
    }

    pub(crate) async fn save_daily_seed(&self, seed: &DailySeed) -> Result<()> {
        self.set_state(seed).await
    }

    pub(crate) async fn save_requests(&self, requests: Vec<Request>) -> Result<()> {
        debug!("Saving requests: {}", requests.len());
        Ok(())
    }
}
