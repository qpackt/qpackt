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

use crate::dao::version::VersionName;
use crate::dao::visits::Visit;
use crate::dao::Dao;
use crate::panel::json_response;
use actix_web::web::Data;
use actix_web::{web, HttpRequest, HttpResponse};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Time in seconds below which a visit is counted as a bounce visit
const BOUNCE_VISIT_MAX_LENGTH: u64 = 5;

#[derive(Deserialize)]
pub(super) struct AnalyticsRequest {
    from_time: DateTime<Utc>,
    to_time: DateTime<Utc>,
}

#[derive(Serialize)]
struct AnalyticsResponse {
    total_visit_count: usize,
    versions_stats: HashMap<VersionName, VersionStats>,
}

/// Stats for single [VersionName].
#[derive(Serialize)]
struct VersionStats {
    average_requests: f32,
    average_duration: Duration,
    bounce_rate: f32,
    visit_count: usize,
}

pub(crate) async fn get_analytics(http: HttpRequest, r: web::Json<AnalyticsRequest>, dao: Data<Dao>) -> HttpResponse {
    let from = r.from_time.timestamp() as u64;
    let to = r.to_time.timestamp() as u64;
    let visits = dao.get_visits(from, to).await.unwrap();
    let response = convert_to_response(visits);
    json_response(&http, response)
}

fn convert_to_response(visits: Vec<Visit>) -> AnalyticsResponse {
    let total_visit_count = visits.len();
    let mut versions_stats = HashMap::with_capacity(16);
    // In the first pass add all the numbers
    for visit in &visits {
        let entry = versions_stats.entry(visit.version.clone()).or_insert_with(|| VersionStats {
            average_requests: 0.0,
            average_duration: Default::default(),
            bounce_rate: 0.0,
            visit_count: 0,
        });
        entry.average_requests += visit.request_count as f32;
        let length = visit.last_request_time - visit.first_request_time;
        entry.average_duration += Duration::from_secs(length);
        if length < BOUNCE_VISIT_MAX_LENGTH {
            entry.bounce_rate += 1.0;
        }
        entry.visit_count += 1;
    }
    // Turn numbers into averages
    for stats in versions_stats.values_mut() {
        let visit_count = stats.visit_count as f32;
        stats.average_requests /= visit_count;
        stats.average_duration = Duration::from_secs_f32(stats.average_duration.as_secs_f32() / visit_count);
        stats.bounce_rate = 100.0 * (stats.bounce_rate / visit_count);
    }

    AnalyticsResponse { total_visit_count, versions_stats }
}
