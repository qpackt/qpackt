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

use crate::dao::Dao;
use crate::error::{QpacktError, Result};
use crate::panel::validate_permission;
use crate::reverse_proxy::ReverseProxies;
use actix_web::web::{Data, Json, Path};
use actix_web::{HttpRequest, Responder};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use url::Url;

#[derive(Serialize)]
pub(crate) struct ReverseProxyDTO {
    id: i32,
    prefix: String,
    target: String,
}

#[derive(Deserialize)]
pub(crate) struct CreateReverseProxyRequest {
    prefix: String,
    target: String,
}

pub(crate) async fn list_proxies(request: HttpRequest, dao: Data<Dao>) -> Result<impl Responder> {
    validate_permission(&request)?;
    debug!("Listing proxies");
    let proxies = dao.list_reverse_proxies().await?;
    let proxies =
        proxies.into_iter().map(|p| ReverseProxyDTO { id: p.id, prefix: p.prefix, target: p.target.to_string() }).collect::<Vec<_>>();
    debug!("Got {} proxies", proxies.len());
    Ok(Json(proxies))
}

pub(crate) async fn delete_proxy(
    request: HttpRequest,
    dao: Data<Dao>,
    reverse_proxies: Data<ReverseProxies>,
    id: Path<i32>,
) -> Result<impl Responder> {
    let id = id.into_inner();
    debug!("Deleting proxy {}", id);
    validate_permission(&request)?;
    dao.delete_reverse_proxy(id).await?;
    let current = dao.list_reverse_proxies().await?;
    reverse_proxies.set(current).await;
    info!("Deleted proxy {}", id);
    Ok("OK".to_string())
}

pub(crate) async fn create_proxy(
    request: HttpRequest,
    dao: Data<Dao>,
    reverse_proxies: Data<ReverseProxies>,
    Json(create_reverse_proxy_request): Json<CreateReverseProxyRequest>,
) -> Result<impl Responder> {
    debug!("Creating reverse proxy: {} -> {}", create_reverse_proxy_request.prefix, create_reverse_proxy_request.target);
    validate_permission(&request)?;
    if let Err(e) = Url::from_str(&create_reverse_proxy_request.target) {
        warn!("Invalid URL when attempting to create proxy `{}`: {}", create_reverse_proxy_request.target, e);
        return Err(QpacktError::ProxyError);
    }
    dao.create_reverse_proxy(&create_reverse_proxy_request.prefix, &create_reverse_proxy_request.target).await?;
    let current = dao.list_reverse_proxies().await?;
    reverse_proxies.set(current).await;
    debug!("Created reverse proxy: {} -> {}", create_reverse_proxy_request.prefix, create_reverse_proxy_request.target);
    Ok("OK".to_string())
}
