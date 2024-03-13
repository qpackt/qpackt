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

use actix_web::{App, HttpResponse, HttpServer, web};
use actix_web::web::{Data, Path};
use awc::body::BoxBody;
use awc::error::StatusCode;
use log::{debug, warn};
use rustls::ServerConfig;

use crate::analytics::event_writer::EventWriter;
use crate::analytics::http_request_log_writer::HttpRequestLogWriter;
use crate::dao::Dao;
use crate::https_redirect::CheckHttpsRedirect;
use crate::proxy::event::{collect_event, QPACKT_EVENT_URI, send_event_script};
use crate::proxy::handler::proxy_handler;
use crate::reverse_proxy::ReverseProxies;
use crate::server::Versions;
use crate::ssl::challenge::AcmeChallenge;

pub(super) mod handler;
pub(super) mod event;

pub(super) fn start_proxy_http(
    addr: &str,
    dao: Data<Dao>,
    versions: Data<Versions>,
    writer: Data<HttpRequestLogWriter>,
    ssl_challenge: Data<AcmeChallenge>,
    reverse_proxies: Data<ReverseProxies>,
    event_writer: Data<EventWriter>,
) {
    tokio::spawn(
        HttpServer::new(move || {
            App::new()
                .wrap(CheckHttpsRedirect {})
                .app_data(dao.clone())
                .app_data(versions.clone())
                .app_data(writer.clone())
                .app_data(ssl_challenge.clone())
                .app_data(reverse_proxies.clone())
                .app_data(event_writer.clone())
                .service(web::resource("/.well-known/acme-challenge/{token}").route(web::get().to(serve_challenge)))
                .service(web::resource(QPACKT_EVENT_URI).post(collect_event))
                .service(web::resource("/qpackt/event/send_event.js").get(send_event_script))
                .default_service(web::to(proxy_handler))
        })
            .bind(addr)
            .unwrap()
            .run(),
    );
}

pub(super) fn start_proxy_https(addr: &str, dao: Data<Dao>, versions: Data<Versions>, writer: Data<HttpRequestLogWriter>, tls_config: ServerConfig, reverse_proxies: Data<ReverseProxies>, event_writer: Data<EventWriter>) {
    tokio::spawn(
        HttpServer::new(move || App::new()
            .app_data(versions.clone())
            .app_data(dao.clone())
            .app_data(writer.clone())
            .app_data(reverse_proxies.clone())
            .app_data(event_writer.clone())
            .service(web::resource(QPACKT_EVENT_URI).post(collect_event))
            .service(web::resource("/qpackt/event/send_event.js").get(send_event_script))
            .default_service(web::to(proxy_handler)))
            .bind_rustls_021(addr, tls_config)
            .unwrap()
            .run(),
    );
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
