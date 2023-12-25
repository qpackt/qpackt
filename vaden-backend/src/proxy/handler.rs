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

use crate::server::Versions;
use actix_web::cookie::Cookie;
use actix_web::dev::RequestHead;
use actix_web::web::{Data, Payload};
use actix_web::{HttpRequest, HttpResponse};
use awc::cookie::time::{Duration, OffsetDateTime};
use awc::http::StatusCode;
use awc::{Client, ClientRequest};
use log::{debug, error};
use rand::{thread_rng, RngCore};
use std::fmt::{Display, Formatter};
use std::ops::{Add, Deref};
use std::sync::Arc;
use url::Url;

/// A cookie that is used to recognize which version was served to the client in previous requests.
/// If no cookie is set then assume it's the first request and use [Strategy] to decide which version will be served
/// from now on.
const VADEN_COOKIE_NAME: &str = "VADEN_SERVER";

#[derive(Clone, Eq, Hash, PartialEq)]
pub(crate) struct CookieValue(Arc<str>);

impl From<&str> for CookieValue {
    fn from(value: &str) -> Self {
        Self(Arc::from(value))
    }
}

impl Display for CookieValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Basic proxy handler (method agnostic).
/// Finds cookie in client's request and previously set url.
/// If not found, then creates a new cookie and picks url from [Versions]
pub(crate) async fn proxy_handler(payload: Payload, client_request: HttpRequest, versions: Data<Versions>) -> HttpResponse {
    let Ok(previous_url) = previous_url(&client_request, &versions).await else {
        return HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR);
    };
    match previous_url {
        None => proxy_to_new(payload, client_request, versions).await,
        Some(url) => proxy_to_previous(payload, client_request, url.deref().clone()).await,
    }
}

async fn proxy_to_new(payload: Payload, client_request: HttpRequest, versions: Data<Versions>) -> HttpResponse {
    let Ok((url, version)) = versions.pick_upstream(client_request.query_string()).await else {
        return HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR);
    };
    let (cookie, cookie_value) = create_new_cookie();
    versions.save_cookie(cookie_value, url.clone(), version).await;
    debug!("Proxying request to {}", url);
    build_response(payload, client_request.head(), url.deref().clone(), Some(cookie)).await
}

async fn proxy_to_previous(payload: Payload, client_request: HttpRequest, url: Url) -> HttpResponse {
    let destination = build_upstream_url(&client_request, url).await;
    debug!("Proxying request to {}", destination);
    build_response(payload, client_request.head(), destination, None).await
}

async fn previous_url(request: &HttpRequest, versions: &Data<Versions>) -> crate::error::Result<Option<Arc<Url>>> {
    let Some(cookie) = request.cookie(VADEN_COOKIE_NAME) else {
        return Ok(None);
    };
    match versions.get_url_for_cookie(&cookie.value().into()).await {
        Ok(url) => Ok(url),
        Err(e) => {
            error!("Unable to get url for cookie: {}", e);
            Err(e)
        }
    }
}

async fn build_response(payload: Payload, head: &RequestHead, destination: Url, cookie: Option<Cookie<'_>>) -> HttpResponse {
    let proxy_request = build_request(head, destination);
    let upstream_response = proxy_request.send_stream(payload).await.unwrap();
    let mut proxy_response = HttpResponse::build(upstream_response.status());
    for (header_name, header_value) in upstream_response.headers().iter().filter(|(h, _)| *h != "connection") {
        proxy_response.insert_header((header_name.clone(), header_value.clone()));
    }
    if let Some(cookie) = cookie {
        proxy_response.cookie(cookie);
    }
    // TODO add "no cache"
    proxy_response.streaming(upstream_response)
}

fn create_new_cookie() -> (Cookie<'static>, CookieValue) {
    let s = format!("{}", thread_rng().next_u64());
    let value = CookieValue::from(s.as_str());
    let mut cookie = Cookie::new(VADEN_COOKIE_NAME, s);
    cookie.set_expires(OffsetDateTime::now_utc().add(Duration::days(7)));
    (cookie, value)
}

fn build_request(head: &RequestHead, destination: Url) -> ClientRequest {
    let client = Client::default();
    client.request_from(destination.as_str(), head).no_decompress()
}

async fn build_upstream_url(client_request: &HttpRequest, mut url: Url) -> Url {
    url.set_path(client_request.uri().path());
    url.set_query(client_request.uri().query());
    url
}
