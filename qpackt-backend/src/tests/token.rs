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

use crate::panel::auth::token::TokenResponse;
use crate::tests::build_config_and_run_app;
use reqwest::Client;
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

/// Builds a config
/// Starts backend
/// Gets admin token
#[tokio::test]
async fn test_get_token() {
    let _dir = build_config_and_run_app().await;
    let token = get_token().await;
    assert!(!token.is_empty());
}

pub(super) async fn get_token() -> String {
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
