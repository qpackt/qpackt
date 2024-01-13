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
use crate::dao::Dao;
use crate::panel::analytics::get_analytics;
use crate::panel::versions::delete::delete_version;
use crate::panel::versions::list::list_versions;
use crate::panel::versions::update::update_versions;
use crate::panel::versions::upload::upload_version;
use crate::server::Versions;
use actix_files::Files;
use actix_web::web::{Data, Json};
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use serde::Serialize;
use tokio::task::JoinHandle;

mod analytics;
mod versions;

pub(super) fn start_panel_http(config: Data<Config>, dao: Data<Dao>, versions: Data<Versions>) -> JoinHandle<std::io::Result<()>> {
    tokio::spawn({
        let app_config = config.clone();
        let versions = versions.clone();
        HttpServer::new(move || {
            App::new()
                .app_data(app_config.clone())
                .app_data(versions.clone())
                .app_data(dao.clone())
                .service(web::resource("/update-versions").route(web::post().to(update_versions)))
                .service(web::resource("/upload-version").route(web::post().to(upload_version)))
                .service(web::resource("/list-versions").route(web::get().to(list_versions)))
                .service(web::resource("/analytics").route(web::post().to(get_analytics)))
                .service(web::resource("/delete-version/{name}").route(web::delete().to(delete_version)))
                // This needs to be at the end of all `service` calls so that backend (api) calls will have a chance to match routes.
                .service(Files::new("/", "../vaden-frontend/dist").index_file("index.html"))
        })
        .bind(config.panel_addr())
        .unwrap()
        .run()
    })
}

fn json_response<T>(request: &HttpRequest, content: T) -> HttpResponse
where
    T: Serialize,
{
    Json(content).respond_to(request).map_into_boxed_body()
}
