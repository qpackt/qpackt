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

use std::env;

use actix_files::Files;
use actix_web::{App, HttpRequest, HttpResponse, HttpServer, web};
use actix_web::http::header;
use actix_web::web::Data;
use awc::http::StatusCode;
use log::warn;
use rustls::ServerConfig;

use auth::token::get_token;

use crate::config::QpacktConfig;
use crate::dao::Dao;
use crate::error::{QpacktError, Result};
use crate::https_redirect::CheckHttpsRedirect;
use crate::panel::analytics::events::{get_events_csv, get_events_stats};
use crate::panel::analytics::get_analytics;
use crate::panel::auth::token::{invalidate_token, is_token_valid};
use crate::panel::reverse_proxy::{create_proxy, delete_proxy, list_proxies};
use crate::panel::versions::delete::delete_version;
use crate::panel::versions::list::list_versions;
use crate::panel::versions::update::update_versions;
use crate::panel::versions::upload::upload_version;
use crate::reverse_proxy::ReverseProxies;
use crate::server::Versions;

mod analytics;
pub(crate) mod auth;
pub(crate) mod reverse_proxy;
mod versions;

const PANEL_HTTP: &str = "0.0.0.0:9080";
// TODO turn port into constant (for tests)
const PANEL_HTTPS: &str = "0.0.0.0:9443";

pub(super) fn start_panel_http(
    config: Data<QpacktConfig>,
    dao: Data<Dao>,
    versions: Data<Versions>,
    tls_config: Option<ServerConfig>,
    reverse_proxies: Data<ReverseProxies>,
) {
    tokio::spawn({
        let app_config = config.clone();
        let server = HttpServer::new(move || {
            let html_path = env::var("QPACKT_HTML_DIR").unwrap_or("/usr/share/qpackt/html".into());
            App::new()
                .wrap(CheckHttpsRedirect {})
                .app_data(app_config.clone())
                .app_data(versions.clone())
                .app_data(dao.clone())
                .app_data(reverse_proxies.clone())
                .service(web::resource("/analytics").route(web::post().to(get_analytics)))
                .service(web::resource("/proxy").get(list_proxies).post(create_proxy))
                .service(web::resource("/proxy/{id}").delete(delete_proxy))
                .service(web::resource("/token").delete(invalidate_token).post(get_token))
                .service(web::resource("/version").post(upload_version))
                .service(web::resource("/version/{name}").route(web::delete().to(delete_version)))
                .service(web::resource("/versions").get(list_versions).put(update_versions))
                .service(web::resource("/events/csv").get(get_events_csv))
                .service(web::resource("/events/stats").get(get_events_stats))
                // This needs to be at the end of all `service` calls so that backend (api) calls will have a chance to match routes.
                .service(Files::new("/", html_path).index_file("index.html"))
        });
        match tls_config {
            None => server.bind(PANEL_HTTP).unwrap().run(),
            Some(tls_config) => server.bind_rustls_021(PANEL_HTTPS, tls_config).unwrap().run(),
        }
    });
}

/// Gets token from 'Authorization: Bearer ...' header.
/// If token is valid then returns HTTP::OK. If token not valid then returns an error.
/// Designed to be used with question mark `validate_permission(...)?` at the beginning
/// of privileged calls.
fn validate_permission(request: &HttpRequest) -> Result<HttpResponse> {
    let Some(header) = request.headers().get(header::AUTHORIZATION) else {
        return Err(QpacktError::Forbidden);
    };
    let Ok(value) = header.to_str() else {
        return Err(QpacktError::Forbidden);
    };
    let Some(parts) = value.split_once(' ') else {
        return Err(QpacktError::Forbidden);
    };
    if is_token_valid(parts.1) {
        Ok(HttpResponse::new(StatusCode::OK))
    } else {
        warn!("Invalid token ({}) from {:?}", parts.1, request.peer_addr());
        Err(QpacktError::Forbidden)
    }
}
