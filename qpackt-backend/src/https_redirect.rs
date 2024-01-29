// SPDX-License-Identifier: AGPL-3.0
/*
   qpackt: Web & Analytics Server
   Copyright (C) 2024 Łukasz Wojtów

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

use std::future::{ready, Ready};
use std::sync::atomic::Ordering;

use crate::ssl::FORCE_HTTPS_REDIRECT;
use actix_web::{
    body::EitherBody,
    dev::{self, Service, ServiceRequest, ServiceResponse, Transform},
    http, Error, HttpResponse,
};
use futures_util::future::LocalBoxFuture;

pub(crate) struct CheckHttpsRedirect;

impl<S, B> Transform<S, ServiceRequest> for CheckHttpsRedirect
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = CheckHttpsRedirectMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(CheckHttpsRedirectMiddleware { service }))
    }
}
pub(crate) struct CheckHttpsRedirectMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for CheckHttpsRedirectMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    dev::forward_ready!(service);

    fn call(&self, request: ServiceRequest) -> Self::Future {
        if FORCE_HTTPS_REDIRECT.load(Ordering::Relaxed) && request.connection_info().scheme() == "http" {
            let (request, _) = request.into_parts();
            let host = request.connection_info().host().to_owned();
            let uri = request.uri().to_owned();
            let url = format!("https://{}{}", host, uri);
            let response = HttpResponse::MovedPermanently().insert_header((http::header::LOCATION, url)).finish().map_into_right_body();
            return Box::pin(async { Ok(ServiceResponse::new(request, response)) });
        }
        let res = self.service.call(request);
        Box::pin(async move { res.await.map(ServiceResponse::map_into_left_body) })
    }
}
