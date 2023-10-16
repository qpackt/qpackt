use crate::config::Config;
use crate::error::Result;
use crate::error::VadenError;
use actix_multipart::{Field, Multipart};
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::{HttpRequest, HttpResponse};
use futures::{StreamExt, TryStreamExt};
use log::warn;
use rand::thread_rng;
use rand::RngCore;
use std::fs::{remove_file, File};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Uploads new site as a zip file, unpacks it and registers in data base.
/// The site can be served after the upload.
pub(crate) async fn upload_site(
    payload: Multipart,
    request: HttpRequest,
    config: Data<Config>,
) -> HttpResponse {
    if let Err(e) = serve_request(payload, request, config).await {
        warn!("Unable to upload site: {}", e);
        HttpResponse::new(StatusCode::BAD_REQUEST)
    } else {
        HttpResponse::Accepted().finish()
    }
}

async fn serve_request(
    mut payload: Multipart,
    request: HttpRequest,
    config: Data<Config>,
) -> Result<()> {
    let field = payload
        .try_next()
        .await
        .map_err(|e| VadenError::MultipartUploadError(e.to_string()))?
        .ok_or_else(|| {
            VadenError::MultipartUploadError("No `next` field in multipart request".into())
        })?;
    parse_site_upload(&request, field, &config.into_inner()).await
}
async fn parse_site_upload(request: &HttpRequest, field: Field, config: &Config) -> Result<()> {
    let zip_path = wait_for_content(field, config.app_run_directory()).await?;
    unzip_and_register(request, &zip_path, config).await
}

async fn unzip_and_register(
    _request: &HttpRequest,
    zip_path: &Path,
    config: &Config,
) -> Result<()> {
    unzip_site(zip_path, config)?;
    Ok(())
}

fn unzip_site(path: &Path, config: &Config) -> Result<()> {
    let file = File::open(path)
        .map_err(|e| VadenError::UnableToProcessSite(format!("unable to open site file: {}", e)))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| VadenError::UnableToProcessSite(format!("unable to read zip: {}", e)))?;
    archive
        .extract(config.app_run_directory())
        .map_err(|e| VadenError::UnableToProcessSite(format!("unable to unzip: {}", e)))?;
    remove_file(path)
        .map_err(|e| VadenError::UnableToProcessSite(format!("unable to unzip: {}", e)))?;
    Ok(())
}

async fn wait_for_content(mut field: Field, run_dir: &Path) -> Result<PathBuf> {
    let zip_path = run_dir.join(format!("{}.zip", thread_rng().next_u64()));
    let mut file = File::create(&zip_path)?;

    while let Some(chunk) = field.next().await {
        let data = chunk.map_err(|e| VadenError::MultipartUploadError(e.to_string()))?;
        file.write_all(data.as_ref())?;
    }
    Ok(zip_path)
}
