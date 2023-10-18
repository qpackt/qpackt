use crate::config::Config;
use crate::dao::Dao;
use crate::error::Result;
use crate::error::VadenError;
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
pub(crate) async fn upload_version(
    payload: Multipart,
    config: Data<Config>,
    dao: Data<Dao>,
) -> HttpResponse {
    if let Err(e) = serve_request(payload, config, dao).await {
        warn!("Unable to upload site: {}", e);
        HttpResponse::new(StatusCode::BAD_REQUEST)
    } else {
        info!("Registered new version");
        HttpResponse::Accepted().finish()
    }
}

//TODO return name and log at caller
async fn serve_request(mut payload: Multipart, config: Data<Config>, dao: Data<Dao>) -> Result<()> {
    let field = payload
        .try_next()
        .await
        .map_err(|e| VadenError::MultipartUploadError(e.to_string()))?
        .ok_or_else(|| {
            VadenError::MultipartUploadError("No `next` field in multipart request".into())
        })?;
    save_site(field, &config.into_inner(), &dao.into_inner()).await
}

async fn save_site(field: Field, config: &Config, dao: &Dao) -> Result<()> {
    let date_time_str = create_date_time_string();
    let target = create_path(config, &date_time_str)?;
    let zip_path = wait_for_content(field, &target).await?;
    unzip_and_register(&zip_path, &target, &date_time_str, dao).await
}

fn create_path(config: &Config, date_time_str: &str) -> Result<PathBuf> {
    let path = config.app_run_directory().join(date_time_str);
    fs::create_dir_all(&path)?;
    Ok(path)
}

fn create_date_time_string() -> String {
    let time: DateTime<Utc> = DateTime::from(SystemTime::now());
    let date_time_str = format!(
        "{}_{:02}_{:02}__{:02}_{:02}_{:02}",
        time.year(),
        time.month(),
        time.day(),
        time.hour(),
        time.minute(),
        time.second()
    );
    date_time_str
}

async fn unzip_and_register(zip_path: &Path, target: &Path, name: &str, dao: &Dao) -> Result<()> {
    let web_root = unzip_site(zip_path, target)?;
    dao.register_version(web_root.to_str().unwrap(), name).await
}

/// Check if there is only one directory in the target dir. If so, it will become web root path.
/// In any other case (multiple directories or any files present) web root path will be same as target.
fn find_web_root(target: &Path) -> Result<PathBuf> {
    let mut dir_count = 0;
    let mut web_root = target.to_path_buf();
    let dir = target.read_dir()?;
    for file in dir {
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
    let file = File::open(path)
        .map_err(|e| VadenError::UnableToProcessSite(format!("unable to open site file: {}", e)))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| VadenError::UnableToProcessSite(format!("unable to read zip: {}", e)))?;
    archive
        .extract(target)
        .map_err(|e| VadenError::UnableToProcessSite(format!("unable to unzip: {}", e)))?;
    remove_file(path)
        .map_err(|e| VadenError::UnableToProcessSite(format!("unable to unzip: {}", e)))?;
    let web_root = find_web_root(target)?;
    Ok(web_root)
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
