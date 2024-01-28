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

use crate::analytics;
use crate::analytics::hash::VisitorHash;
use crate::analytics::writer::RequestWriter;
use crate::dao::requests::Request;
use crate::dao::version::VersionName;
use crate::server::Versions;
use actix_web::cookie::Cookie;
use actix_web::dev::RequestHead;
use actix_web::web::{Data, Payload};
use actix_web::{HttpRequest, HttpResponse};
use awc::cookie::time::{Duration, OffsetDateTime};
use awc::http::StatusCode;
use awc::{Client, ClientRequest};
use log::debug;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::ops::{Add, Deref};
use std::sync::Arc;
use url::Url;

/// A cookie that is used to recognize which version was served to the client in previous requests.
/// If no cookie is set then assume it's the first request and use [Strategy] to decide which version will be served
/// from now on.
const QPACKT_COOKIE_NAME: &str = "QPACKT_VERSION";

/// Basic proxy handler (method agnostic).
/// Finds cookie in client's request and previously set url.
/// If not found, then creates a new cookie and picks url from [Versions]
pub(crate) async fn proxy_handler(
    payload: Payload,
    client_request: HttpRequest,
    versions: Data<Versions>,
    writer: Data<RequestWriter>,
) -> HttpResponse {
    match previous_url(&client_request, &versions).await {
        None => proxy_to_new(payload, client_request, versions, writer).await,
        Some((url, version)) => proxy_to_previous(payload, client_request, url.deref().clone(), writer, version).await,
    }
}

async fn proxy_to_new(
    payload: Payload,
    client_request: HttpRequest,
    versions: Data<Versions>,
    writer: Data<RequestWriter>,
) -> HttpResponse {
    let Ok((url, version)) = versions.pick_upstream(client_request.query_string()).await else {
        return HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR);
    };
    let cookie = create_new_cookie(version.clone());
    let hash = calculate_visitor_hash(&client_request);
    debug!("Proxying request to {} with visitor hash {:?}", url, hash);
    writer.save(Request::new(hash, version, client_request.uri().clone())).await;
    build_response(payload, client_request.head(), url.deref().clone(), Some(cookie)).await
}

fn calculate_visitor_hash(client_request: &HttpRequest) -> VisitorHash {
    let peer = client_request.peer_addr().unwrap_or_else(|| SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 1))).ip();
    let user_agent = client_request.headers().get("User-Agent").map(|v| v.as_bytes().to_vec()).unwrap_or_default();
    analytics::hash::create(peer, user_agent)
}

async fn proxy_to_previous(
    payload: Payload,
    client_request: HttpRequest,
    url: Url,
    writer: Data<RequestWriter>,
    version: VersionName,
) -> HttpResponse {
    let hash = calculate_visitor_hash(&client_request);
    debug!("Proxying request to {} with visitor hash {:?}", url, hash);
    let destination = build_upstream_url(&client_request, url).await;
    writer.save(Request::new(hash, version, client_request.uri().clone())).await;
    build_response(payload, client_request.head(), destination, None).await
}

async fn previous_url(request: &HttpRequest, versions: &Data<Versions>) -> Option<(Arc<Url>, VersionName)> {
    let version = request.cookie(QPACKT_COOKIE_NAME)?;
    versions.get_url_for_cookie(version.value()).await
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

fn create_new_cookie(version: VersionName) -> Cookie<'static> {
    let mut cookie = Cookie::new(QPACKT_COOKIE_NAME, version.to_string());
    cookie.set_expires(OffsetDateTime::now_utc().add(Duration::days(7)));
    cookie
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
