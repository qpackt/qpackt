// SPDX-License-Identifier: AGPL-3.0
/*
   Primeclue: Machine Learning and Data Mining
   Copyright (C) 2020 Łukasz Wojtów

   This program is free software: you can redistribute it and/or modify
   it under the terms of the GNU Affero General Public License as
   published by the Free Software Foundation, either version 3 of the
   License, or (at your option) any later version.

   This program is distributed in the hope that it will be useful,
   but WITHOUT ANY WARRANTY; without even the implied warranty of
   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
   GNU Affero General Public License for more details.

   You should have received a copy of the GNU Affero General Public License
   along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

use actix_files::Files;
use actix_web::{web, App, HttpRequest, HttpServer, Responder};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    start_http().await
}

async fn start_http() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(Files::new("/static", "../vaden-frontend/dist").index_file("index.html"))
            .default_service(web::to(default))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

async fn default(request: HttpRequest) -> impl Responder {
    println!("req: {:?}", request.headers());
    "OK"
}
