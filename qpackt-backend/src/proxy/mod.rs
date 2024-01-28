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

use crate::analytics::writer::RequestWriter;
use crate::proxy::handler::proxy_handler;
use crate::server::Versions;
use crate::ssl::challenge::AcmeChallenge;
use actix_web::web::{Data, Path};
use actix_web::{web, App, HttpResponse, HttpServer};
use awc::body::BoxBody;
use awc::error::StatusCode;
use log::{debug, warn};
use rustls::ServerConfig;
use tokio::task::JoinHandle;

pub(super) mod handler;

pub(super) fn start_proxy_http(
    addr: &str,
    versions: Data<Versions>,
    writer: Data<RequestWriter>,
    ssl_challenge: Data<AcmeChallenge>,
) -> JoinHandle<std::io::Result<()>> {
    tokio::spawn(
        HttpServer::new(move || {
            App::new()
                .app_data(versions.clone())
                .app_data(writer.clone())
                .app_data(ssl_challenge.clone())
                .service(web::resource("/.well-known/acme-challenge/{token}").route(web::get().to(serve_challenge)))
                .default_service(web::to(proxy_handler))
        })
        .bind(addr)
        .unwrap()
        .run(),
    )
}

pub(super) async fn start_proxy_https(
    addr: &str,
    versions: Data<Versions>,
    writer: Data<RequestWriter>,
    tls_config: ServerConfig,
) -> JoinHandle<std::io::Result<()>> {
    tokio::spawn(
        HttpServer::new(move || App::new().app_data(versions.clone()).app_data(writer.clone()).default_service(web::to(proxy_handler)))
            .bind_rustls_021(addr, tls_config)
            .unwrap()
            .run(),
    )
}

async fn serve_challenge(token: Path<String>, ssl_challenge: Data<AcmeChallenge>) -> HttpResponse {
    debug!("Checking SSL challenge for token: {}", token);
    let token = token.into_inner();
    if let Some(proof) = ssl_challenge.get_proof(token).await {
        debug!("Found correct proof");
        HttpResponse::with_body(StatusCode::OK, BoxBody::new(proof.to_string()))
    } else {
        warn!("No proof found");
        HttpResponse::new(StatusCode::NOT_FOUND)
    }
}