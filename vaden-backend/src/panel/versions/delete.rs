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

use crate::config::Config;
use crate::constants::VERSIONS_SUBDIRECTORY;
use crate::dao::Dao;
use crate::VersionHandler;
use actix_web::web::{Data, Path};
use actix_web::HttpResponse;
use awc::http::StatusCode;
use log::{debug, info, warn};
use std::fs;
use tokio::sync::RwLock;

/// Deletes site's version from file system and the database.
pub(crate) async fn delete_version(
    name: Path<String>,
    dao: Data<Dao>,
    app: Data<Config>,
    versions: Data<RwLock<Vec<VersionHandler>>>,
) -> HttpResponse {
    debug!("Deleting version {}", name);
    match dao.delete_version(&name).await {
        Ok(path) => {
            let path = app
                .app_run_directory()
                .join(VERSIONS_SUBDIRECTORY)
                .join(path);
            if let Err(e) = fs::remove_dir_all(&path) {
                warn!(
                    "Unable to delete path {:?} for version {}: {}",
                    path, name, e
                );
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(format!("Unable to delete: {}", e))
            } else {
                info!("Removed version {} and path {:?}", name, path);
                HttpResponse::new(StatusCode::OK)
            }
        }
        Err(e) => {
            warn!("Unable to delete version {}: {}", name, e);
            HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                .body(format!("Unable to delete: {}", e))
        }
    }
}
