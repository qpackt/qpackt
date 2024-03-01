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

use crate::tests::build_config_and_run_app;
use crate::tests::token::get_token;
use actix_web::dev::Server;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json};
use actix_web::{web, App, HttpResponse, HttpServer};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::net::{Ipv4Addr, SocketAddrV4};
use tokio::sync::mpsc::{channel, Sender};

/// Gets token
/// Creates reverse proxy
/// Starts proxy server
/// Calls some endpoint
#[tokio::test]
async fn test_reverse_proxy() {
    let _dir = build_config_and_run_app().await;
    let token = get_token().await;
    assert!(!token.is_empty());
    let client = Client::new();
    let request = client
        .post("http://localhost:9080/proxy")
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({"prefix":"/reverse", "target":"http://localhost:9999/api"}));
    let result = request.send().await.unwrap();
    assert_eq!(result.status(), StatusCode::OK);
    let (sender, mut receiver) = channel(1);
    let server = build_app_server(sender);
    let handle = server.handle();
    tokio::spawn(server);
    let send_message = SomePayload { var1: "var1".to_string(), var2: "var2".to_string() };
    let request = client.post("http://localhost:8080/reverse/test/123").json(&send_message);
    request.send().await.unwrap();
    let received_message = receiver.recv().await.unwrap();
    assert_eq!(received_message, send_message);
    handle.stop(true).await;
}

async fn test_endpoint(Json(payload): Json<SomePayload>, sender: Data<Sender<SomePayload>>) -> HttpResponse {
    let sender = sender.into_inner();
    sender.send(payload).await.unwrap();
    HttpResponse::new(StatusCode::OK)
}

fn build_app_server(sender: Sender<SomePayload>) -> Server {
    HttpServer::new(move || App::new().app_data(Data::new(sender.clone())).service(web::resource("/api/test/{id}").post(test_endpoint)))
        .bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 9999))
        .unwrap()
        .run()
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct SomePayload {
    var1: String,
    var2: String,
}
