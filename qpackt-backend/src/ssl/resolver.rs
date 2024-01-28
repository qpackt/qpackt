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

use acme_lib::Certificate;
use log::debug;
use rustls::internal::msgs::codec;
use rustls::internal::msgs::codec::Codec;
use rustls::server::{ClientHello, ResolvesServerCert};
use rustls::sign::{any_supported_type, CertifiedKey};
use rustls::PrivateKey;
use std::sync::Arc;

/// Resolver that can be used to bind tls with actix.
pub(crate) fn try_build_resolver(certificate: Certificate) -> CertResolver {
    debug!("Building TLS resolver");
    let der = certificate.certificate_der();
    let mut bytes = Vec::with_capacity(der.len() + 3);
    codec::u24(der.len() as u32).encode(&mut bytes);
    bytes.extend_from_slice(&der);
    let cert = rustls::Certificate::read_bytes(&bytes).unwrap();
    let pk = PrivateKey(certificate.private_key_der());
    let key = any_supported_type(&pk).unwrap();
    let key = Arc::new(CertifiedKey::new(vec![cert], key));
    CertResolver { key }
}

pub(crate) struct CertResolver {
    key: Arc<CertifiedKey>,
}

impl ResolvesServerCert for CertResolver {
    fn resolve(&self, _: ClientHello<'_>) -> Option<Arc<CertifiedKey>> {
        Some(self.key.clone())
    }
}
