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
use crate::dao::{Dao, VersionName};
use crate::manager::strategy::Strategy;
use crate::server::Versions;
use actix_web::web::Data;
use actix_web::{web, HttpResponse};
use awc::http::StatusCode;
use log::{debug, error, info};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct VersionRequest {
    name: VersionName,
    strategy: Strategy,
}

/// Updates configuration for traffic split.
/// * Retrieve versions from database
/// * Update according to request and save to database
/// * Update handlers to be used for actual traffic split
pub(crate) async fn update_versions(
    web::Json(requests): web::Json<Vec<VersionRequest>>,
    versions: Data<Versions>,
    dao: Data<Dao>,
) -> HttpResponse {
    debug!("Received versions update: {:?}", requests);
    let mut current = match dao.list_versions().await {
        Ok(current) => current,
        Err(e) => {
            error!("Unable to list site's versions: {}", e);
            return HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    for current_version in &mut current {
        for new_version in &requests {
            if current_version.name == new_version.name {
                current_version.strategy = new_version.strategy.clone();
                break;
            }
        }
    }
    if let Err(e) = dao.save_versions(&current).await {
        error!("Unable to save new site's versions: {}", e);
        return HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR);
    }
    versions.update_strategies(&current).await;
    info!("Saved new site's versions: {:?}", current);
    HttpResponse::new(StatusCode::OK)
}
