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

use crate::config::QpacktConfig;
use crate::error::QpacktError;
use crate::error::Result;
use crate::panel::auth::password::password_matches;
use crate::panel::validate_permission;
use actix_web::web::{Data, Json};
use actix_web::{HttpRequest, HttpResponse, Responder};
use awc::http::StatusCode;
use log::{info, warn};
use rand::{thread_rng, RngCore};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};

static ADMIN_TOKEN: AtomicU64 = AtomicU64::new(0);

#[derive(Deserialize)]
pub(crate) struct TokenRequest {
    password: String,
}

#[derive(Serialize)]
struct TokenResponse {
    token: String,
}

/// Gets token if supplied correct password.
pub(crate) async fn get_token(
    http: HttpRequest,
    Json(token_request): Json<TokenRequest>,
    config: Data<QpacktConfig>,
) -> Result<impl Responder> {
    let correct = password_matches(token_request.password, config.password())?;
    if correct {
        let token = create_token();
        Ok(Json(TokenResponse { token }))
    } else {
        warn!("Invalid admin password from {:?}", http.peer_addr());
        Err(QpacktError::Forbidden)
    }
}

/// Removes token from the state. This makes [is_token_valid] below to always return false.
pub(crate) async fn invalidate_token(request: HttpRequest) -> Result<impl Responder> {
    validate_permission(&request)?;
    ADMIN_TOKEN.store(0, Ordering::SeqCst);
    info!("Cleared admin token");
    Ok(HttpResponse::new(StatusCode::OK))
}

/// Checks for token validity. Token in state cannot be 0.
pub(crate) fn is_token_valid(token: &str) -> bool {
    match token.parse::<u64>() {
        Ok(v) => {
            let current = ADMIN_TOKEN.load(Ordering::SeqCst);
            current != 0 && current == v
        }
        Err(_) => false,
    }
}

/// Creates a random token and saves it in state. Later token is compared in [is_token_valid].
pub(crate) fn create_token() -> String {
    let v = thread_rng().next_u64();
    ADMIN_TOKEN.store(v, Ordering::SeqCst);
    info!("Created admin token");
    format!("{v}")
}
