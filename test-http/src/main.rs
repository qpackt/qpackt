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

use actix_files::NamedFile;
use actix_multipart::{Field, Multipart};
use actix_web::http::StatusCode;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
use futures_util::StreamExt;
use futures_util::TryStreamExt;
use std::fs;
use std::path::PathBuf;

/// A simple web server used for testing the main proxy.
#[actix_web::main]
async fn main() {
    HttpServer::new(|| {
        App::new()
            .service(web::resource("/upload").route(web::post().to(upload)))
            .service(web::resource("/download").route(web::get().to(download)))
    })
    .bind(("0.0.0.0", 1111))
    .unwrap()
    .run()
    .await
    .unwrap();
}
/// Accepts multiple files and saves them to disk.
/// Saved files then can be checked for correctness by comparing their hashes to original files.
async fn upload(mut payload: Multipart) -> HttpResponse {
    while let Ok(Some(field)) = payload.try_next().await {
        parse_upload(field).await;
    }
    HttpResponse::new(StatusCode::CREATED)
}

/// Serves data1.bin file to the client
/// File should be saved with different name and its checksum compared with the original.
async fn download(request: HttpRequest) -> HttpResponse {
    let file_path = PathBuf::from("data1.bin");
    let file = NamedFile::open_async(file_path).await.unwrap();
    file.into_response(&request)
}

async fn parse_upload(field: Field) -> HttpResponse {
    let name = field
        .content_disposition()
        .get_filename()
        .unwrap()
        .to_string();
    let content = wait_for_content(field).await.unwrap();
    println!("Have content {} ({} bytes)", name, content.len());
    fs::write(format!("received_{}", name), content).unwrap();
    HttpResponse::new(StatusCode::CREATED)
}

async fn wait_for_content(mut field: Field) -> Result<Vec<u8>, HttpResponse> {
    let mut content = vec![];
    while let Some(chunk) = field.next().await {
        let data = chunk.unwrap();
        content.extend(data.to_vec());
    }
    Ok(content)
}
