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
use crate::dao::Dao;
use crate::https_redirect::CheckHttpsRedirect;
use crate::panel::analytics::get_analytics;
use crate::panel::versions::delete::delete_version;
use crate::panel::versions::list::list_versions;
use crate::panel::versions::update::update_versions;
use crate::panel::versions::upload::upload_version;
use crate::server::Versions;
use actix_files::Files;
use actix_web::web::{Data, Json};
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use rustls::ServerConfig;
use serde::Serialize;

mod analytics;
mod versions;

const PANEL_HTTP: &str = "0.0.0.0:9080";
const PANEL_HTTPS: &str = "0.0.0.0:9443";

pub(super) fn start_panel_http(config: Data<QpacktConfig>, dao: Data<Dao>, versions: Data<Versions>, tls_config: Option<ServerConfig>) {
    tokio::spawn({
        let app_config = config.clone();
        let server = HttpServer::new(move || {
            App::new()
                .wrap(CheckHttpsRedirect {})
                .app_data(app_config.clone())
                .app_data(versions.clone())
                .app_data(dao.clone())
                .service(web::resource("/update-versions").route(web::post().to(update_versions)))
                .service(web::resource("/upload-version").route(web::post().to(upload_version)))
                .service(web::resource("/list-versions").route(web::get().to(list_versions)))
                .service(web::resource("/analytics").route(web::post().to(get_analytics)))
                .service(web::resource("/delete-version/{name}").route(web::delete().to(delete_version)))
                // This needs to be at the end of all `service` calls so that backend (api) calls will have a chance to match routes.
                .service(Files::new("/", "../qpackt-frontend/dist").index_file("index.html"))
        });
        match tls_config {
            None => server.bind(PANEL_HTTP).unwrap().run(),
            Some(tls_config) => server.bind_rustls_021(PANEL_HTTPS, tls_config).unwrap().run(),
        }
    });
}

fn json_response<T>(request: &HttpRequest, content: T) -> HttpResponse
where
    T: Serialize,
{
    Json(content).respond_to(request).map_into_boxed_body()
}
