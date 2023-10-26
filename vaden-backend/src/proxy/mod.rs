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
use actix_web::web::Data;
use actix_web::{web, App, HttpServer};
use tokio::task::JoinHandle;
pub(super) mod handler;
pub(super) mod upstream;

pub(super) fn start_proxy_http(
    upstreams: Data<Upstreams>,
    addr: &str,
) -> JoinHandle<std::io::Result<()>> {
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
