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

use crate::proxy::handler::proxy_handler;
use crate::proxy::upstream::Upstreams;
use actix_files::Files;
use actix_web::web::Data;
use actix_web::{web, App, HttpServer};
use tokio::task::JoinHandle;

pub(super) async fn start_http() -> (
    JoinHandle<std::io::Result<()>>,
    JoinHandle<std::io::Result<()>>,
) {
    let upstreams = Data::new(Upstreams::default());

    let panel_handle = start_panel_http(upstreams.clone());
    let proxy_handle = start_proxy_http(upstreams);

    (panel_handle, proxy_handle)
}

fn start_proxy_http(upstreams: Data<Upstreams>) -> JoinHandle<std::io::Result<()>> {
    tokio::spawn(
        HttpServer::new(move || {
            App::new()
                .app_data(upstreams.clone())
                .default_service(web::to(proxy_handler))
        })
        .bind(("0.0.0.0", 8080))
        .unwrap()
        .run(),
    )
}

fn start_panel_http(upstreams: Data<Upstreams>) -> JoinHandle<std::io::Result<()>> {
    tokio::spawn(
        HttpServer::new(move || {
            App::new()
                .app_data(upstreams.clone())
                .service(Files::new("/static", "../vaden-frontend/dist").index_file("index.html"))
        })
        .bind(("0.0.0.0", 8081))
        .unwrap()
        .run(),
    )
}
