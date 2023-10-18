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
use crate::panel::upload_version;
use crate::proxy::handler::proxy_handler;
use crate::proxy::upstream::Upstreams;
use actix_files::Files;
use actix_web::web::Data;
use actix_web::{web, App, HttpServer};
use tokio::task::JoinHandle;

/// Starts two actix processes to serve admin's panel and http proxy.
/// Returns two join handles so later then can be awaited.
pub(super) async fn start_http(
    config: Config,
    dao: Dao,
) -> (
    JoinHandle<std::io::Result<()>>,
    JoinHandle<std::io::Result<()>>,
) {
    let upstreams = Data::new(Upstreams::default());
    let config = Data::new(config);
    let dao = Data::new(dao);
    let panel_handle = start_panel_http(upstreams.clone(), config.clone(), dao);
    let proxy_handle = start_proxy_http(upstreams, config.proxy_addr());

    (panel_handle, proxy_handle)
}

fn start_proxy_http(upstreams: Data<Upstreams>, addr: &str) -> JoinHandle<std::io::Result<()>> {
    tokio::spawn(
        HttpServer::new(move || {
            App::new()
                .app_data(upstreams.clone())
                .default_service(web::to(proxy_handler))
        })
        .bind(addr)
        .unwrap()
        .run(),
    )
}

fn start_panel_http(
    upstreams: Data<Upstreams>,
    config: Data<Config>,
    dao: Data<Dao>,
) -> JoinHandle<std::io::Result<()>> {
    tokio::spawn({
        let app_config = config.clone();
        HttpServer::new(move || {
            App::new()
                .app_data(upstreams.clone())
                .app_data(app_config.clone())
                .app_data(dao.clone())
                .service(Files::new("/static", "../vaden-frontend/dist").index_file("index.html"))
                .service(web::resource("/upload-version").route(web::post().to(upload_version)))
        })
        .bind(config.panel_addr())
        .unwrap()
        .run()
    })
}
