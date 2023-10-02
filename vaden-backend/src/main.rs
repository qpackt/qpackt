// SPDX-License-Identifier: AGPL-3.0
/*
   Vaden: Versioned Application Deployment Engine
   Copyright (C) 2023 Łukasz Wojtów

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
use actix_web::web::Payload;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
use awc::Client;
use std::env;
use std::future;
use std::str::FromStr;
use url::Url;

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
        })
        .bind(("0.0.0.0", 8081))
        .unwrap()
        .run(),
    );

    println!("started 1");
    tokio::spawn(
        HttpServer::new(|| App::new().default_service(web::to(proxy)))
            .bind(("0.0.0.0", 8080))
            .unwrap()
            .run(),
    );
    println!("started 2");
}

async fn proxy(payload: Payload, client_request: HttpRequest) -> HttpResponse {
    let mut destination = Url::from_str("http://localhost:1111").unwrap();
    destination.set_path(client_request.uri().path());
    destination.set_query(client_request.uri().query());

    let client = Client::default();
    let proxy_request = client
        .request_from(destination.as_str(), client_request.head())
        .no_decompress();
    let upstream_response = proxy_request.send_stream(payload).await.unwrap();
    let mut proxy_response = HttpResponse::build(upstream_response.status());
    for (header_name, header_value) in upstream_response
        .headers()
        .iter()
        .filter(|(h, _)| *h != "connection")
    {
        proxy_response.insert_header((header_name.clone(), header_value.clone()));
    }
    proxy_response.streaming(upstream_response)
}
