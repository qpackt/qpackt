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

#![cfg(test)]

use serde::{Deserialize, Serialize};

mod token_tests {
    use crate::panel::auth::token::TokenResponse;
    use crate::run_app;
    use crate::tests::SomePayload;
    use actix_web::dev::Server;
    use actix_web::web::{Data, Json};
    use actix_web::{web, App, HttpResponse, HttpServer};
    use reqwest::{Client, StatusCode};
    use serde_json::json;
    use std::env;
    use std::net::{Ipv4Addr, SocketAddrV4};
    use std::path::{Path, PathBuf};
    use std::time::Duration;
    use tokio::fs;
    use tokio::sync::mpsc::{channel, Sender};
    use tokio::task::JoinHandle;
    use tokio::time::sleep;

    /// Builds a config
    /// Starts backend
    /// Gets admin token
    #[tokio::test]
    async fn test_get_token() {
        let dir = tmpdir::TmpDir::new("qpackt_dir").await.expect("Unable to create temp dir");
        let config = write_config_file(&dir.to_path_buf()).await;
        env::set_var("QPACKT_HTML_DIR", ".");
        let task = run_app(&config).await;
        let _process = ProcessHandler { task };
        let token = get_token().await;
        assert!(!token.is_empty());
    }

    /// Gets token
    /// Creates reverse proxy
    /// Starts proxy server
    /// Calls some endpoint
    #[tokio::test]
    async fn test_reverse_proxy() {
        let dir = tmpdir::TmpDir::new("qpackt_dir").await.expect("Unable to create temp dir");
        let config = write_config_file(&dir.to_path_buf()).await;
        env::set_var("QPACKT_HTML_DIR", ".");
        let task = run_app(&config).await;
        let _process = ProcessHandler { task };
        let token = get_token().await;
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
        let send_message = SomePayload { var1: "var1".to_string(), var2: "var2".to_string() };
        tokio::spawn(server);
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

    async fn write_config_file(dir: &Path) -> PathBuf {
        let content =  format!("domain: qpackt.com\nhttp_proxy: 0.0.0.0:8080\npassword: $scrypt$ln=17,r=8,p=1$H63UY378M+ql3bpQMQ37aQ$XXt3kOaWrW/CQr+/lPIDtPlPTLJSHbaaGBEVo3l3wFY\nrun_directory: {}", dir.to_str().unwrap());
        let config = dir.join("qpackt.yaml");
        fs::write(&config, content).await.expect("Unable to write config");
        config
    }

    async fn get_token() -> String {
        for _ in 0..5 {
            let client = Client::new();
            let request = client.post("http://localhost:9080/token").json(&json!({"password":"admin"}));
            if let Ok(response) = request.send().await {
                if let Ok(token) = response.json::<TokenResponse>().await {
                    return token.token;
                }
            }
            sleep(Duration::from_secs(1)).await;
        }
        panic!("No token after timeout!");
    }

    /// Convenience struct to ensure task aborted when dropped.
    struct ProcessHandler {
        task: JoinHandle<()>,
    }

    impl Drop for ProcessHandler {
        fn drop(&mut self) {
            self.task.abort();
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct SomePayload {
    var1: String,
    var2: String,
}
