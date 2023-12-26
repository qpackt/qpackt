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
use crate::dao::version::{Version, VersionName};
use crate::dao::Dao;
use crate::error::Result;
use crate::error::VadenError;
use crate::manager::strategy::Strategy;
use crate::server::Versions;
use actix_multipart::{Field, Multipart};
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::HttpResponse;
use chrono::{DateTime, Datelike, Timelike, Utc};
use futures::{StreamExt, TryStreamExt};
use log::{info, warn};
use std::fs;
use std::fs::{remove_file, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Uploads new site's version as a zip file, unpacks it and registers in database.
/// The site can be served after the upload.
pub(crate) async fn upload_version(payload: Multipart, config: Data<Config>, dao: Data<Dao>, versions: Data<Versions>) -> HttpResponse {
    match serve_request(payload, config, dao, versions).await {
        Ok(name) => {
            info!("Registered new version: {}", name);
            HttpResponse::Accepted().finish()
        }
        Err(e) => {
            warn!("Unable to upload site: {}", e);
            HttpResponse::new(StatusCode::BAD_REQUEST)
        }
    }
}

async fn serve_request(mut payload: Multipart, config: Data<Config>, dao: Data<Dao>, versions: Data<Versions>) -> Result<VersionName> {
    let field = payload
        .try_next()
        .await
        .map_err(|e| VadenError::MultipartUploadError(e.to_string()))?
        .ok_or_else(|| VadenError::MultipartUploadError("No `next` field in multipart request".into()))?;
    let version = save_version(field, &config.clone().into_inner(), &dao.into_inner()).await?;
    let name = version.name.clone();
    versions.add_version(version, config.into_inner().app_run_directory()).await;
    Ok(name)
}

async fn save_version(field: Field, config: &Config, dao: &Dao) -> Result<Version> {
    let name = create_name();
    let target = create_path(config, &name)?;
    let zip_path = wait_for_content(field, &target).await?;
    unzip_and_register(&zip_path, &target, name, config.app_run_directory(), dao).await
}

fn create_path(config: &Config, name: &VersionName) -> Result<PathBuf> {
    let path = config.app_run_directory().join(VERSIONS_SUBDIRECTORY).join(name.to_string());
    fs::create_dir_all(&path)?;
    Ok(path)
}

fn create_name() -> VersionName {
    let time: DateTime<Utc> = DateTime::from(SystemTime::now());
    let date_time_str =
        format!("{}_{:02}_{:02}__{:02}_{:02}_{:02}", time.year(), time.month(), time.day(), time.hour(), time.minute(), time.second());
    date_time_str.into()
}

async fn wait_for_content(mut field: Field, target: &Path) -> Result<PathBuf> {
    let zip_path = target.join("in_progress.zip");
    let mut file = File::create(&zip_path)?;
    while let Some(chunk) = field.next().await {
        let data = chunk.map_err(|e| VadenError::MultipartUploadError(e.to_string()))?;
        file.write_all(data.as_ref())?;
    }
    Ok(zip_path)
}

async fn unzip_and_register(zip_path: &Path, target: &Path, name: VersionName, app_run_dir: &Path, dao: &Dao) -> Result<Version> {
    let web_root = unzip_site(zip_path, target)?;
    let web_root = web_root
        .strip_prefix(app_run_dir.join(VERSIONS_SUBDIRECTORY))
        .map_err(|e| VadenError::UnableToProcessSite(format!("unable to strip site prefix: {}", e)))?;
    let version = Version { name, web_root: web_root.to_path_buf(), strategy: Strategy::Weight(0) };
    dao.register_version(&version).await?;
    Ok(version)
}

/// Check if there is only one directory in the target dir. If so, it will become web root path.
/// In any other case (multiple directories or any files present) web root path will be same as target.
fn find_web_root(target: &Path) -> Result<PathBuf> {
    let mut dir_count = 0;
    let mut web_root = target.to_path_buf();
    for file in target.read_dir()? {
        let file = file?;
        if file.file_type()?.is_dir() {
            dir_count += 1;
            if dir_count > 1 {
                return Ok(target.to_path_buf());
            }
            web_root = file.path();
        } else if file.file_type()?.is_file() {
            return Ok(target.to_path_buf());
        }
    }
    Ok(web_root)
}

fn unzip_site(path: &Path, target: &Path) -> Result<PathBuf> {
    let file = File::open(path).map_err(|e| VadenError::UnableToProcessSite(format!("unable to open site file: {}", e)))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| VadenError::UnableToProcessSite(format!("unable to read zip: {}", e)))?;
    archive.extract(target).map_err(|e| VadenError::UnableToProcessSite(format!("unable to unzip: {}", e)))?;
    remove_file(path).map_err(|e| VadenError::UnableToProcessSite(format!("unable to unzip: {}", e)))?;
    let web_root = find_web_root(target)?;
    Ok(web_root)
}
