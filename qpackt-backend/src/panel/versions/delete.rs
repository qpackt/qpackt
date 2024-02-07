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

use crate::config::QpacktConfig;
use crate::constants::VERSIONS_SUBDIRECTORY;
use crate::dao::Dao;
use crate::error::{QpacktError, Result};
use crate::panel::validate_permission;
use crate::server::Versions;
use actix_web::web::{Data, Path};
use actix_web::HttpRequest;
use log::{debug, info, warn};
use std::fs;

/// Deletes site's version from file system and the database.
pub(crate) async fn delete_version(
    request: HttpRequest,
    name: Path<String>,
    dao: Data<Dao>,
    app: Data<QpacktConfig>,
    versions: Data<Versions>,
) -> Result<String> {
    validate_permission(&request)?;
    debug!("Deleting version {}", name);
    match dao.delete_version(&name).await {
        Ok(path) => {
            let name = name.into_inner().into();
            versions.delete_version(&name).await;
            let path = app.app_run_directory().join(VERSIONS_SUBDIRECTORY).join(path);
            if let Err(e) = fs::remove_dir_all(&path) {
                warn!("Unable to delete path {:?} for version {}: {}", path, name, e);
                Err(QpacktError::InvalidConfig("Unable to delete path".to_string()))
            } else {
                info!("Removed version {} and path {:?}", name, path);
                Ok("OK".to_string())
            }
        }
        Err(e) => {
            warn!("Unable to delete version {}: {}", name, e);
            Err(e)
        }
    }
}
