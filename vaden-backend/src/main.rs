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

mod config;
mod error;
mod password;

use crate::config::Config;
use actix_files::Files;
use actix_web::{web, App, HttpRequest, HttpServer, Responder};
use std::env;
use std::future;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if let Some(config_path) = args.get(1) {
        let config = Config::read(config_path).await.unwrap();
        println!("config: {:?}", config);
        start_http().await;
        future::pending::<()>().await;
    } else {
        let config = Config::new().unwrap();
        let path = "vaden.yaml";
        config.save(path).await.unwrap();
        println!("Config file saved in {}", path);
    }
}

async fn start_http() {
    tokio::spawn(
        HttpServer::new(|| {
            App::new()
                .service(Files::new("/static", "../vaden-frontend/dist").index_file("index.html"))
                .default_service(web::to(default))
        })
        .bind(("0.0.0.0", 8081))
        .unwrap()
        .run(),
    );

    println!("started 1");
    tokio::spawn(
        HttpServer::new(|| App::new().default_service(web::to(default)))
            .bind(("0.0.0.0", 8080))
            .unwrap()
            .run(),
    );
    println!("started 2");
}

async fn default(request: HttpRequest) -> impl Responder {
    println!("req: {:?}", request.headers());
    "OK"
}
