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

#![forbid(unsafe_code)]
#![warn(
    anonymous_parameters,
    missing_copy_implementations,
    missing_debug_implementations,
    nonstandard_style,
    rust_2018_idioms,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unused_extern_crates,
    unused_qualifications,
    variant_size_differences
)]
mod analytics;
mod config;
pub mod constants;
pub mod dao;
mod error;
mod https_redirect;
mod manager;
mod panel;
mod proxy;
mod server;
mod ssl;

use crate::analytics::writer::RequestWriter;
use crate::config::QpacktConfig;
use crate::dao::version::Version;
use crate::dao::Dao;
use crate::error::QpacktError;
use crate::error::Result;
use crate::panel::start_panel_http;
use crate::proxy::{start_proxy_http, start_proxy_https};
use crate::server::Versions;
use crate::ssl::challenge::AcmeChallenge;
use crate::ssl::resolver::try_build_resolver;
use crate::ssl::{get_certificate, FORCE_HTTPS_REDIRECT};
use actix_web::web::Data;
use log::info;
use rustls::ServerConfig;
use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use std::{env, fs};
use tokio::select;
use tokio::signal::unix::{signal, SignalKind};

#[tokio::main]
async fn main() {
    env::set_var("RUST_LOG", "debug");
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    if let Some(config_path) = args.get(1) {
        let config = QpacktConfig::read(config_path).await.unwrap();
        ensure_app_dir_exists(config.app_run_directory()).unwrap();
        let dao = Dao::init(config.app_run_directory()).await.unwrap();
        let writer = RequestWriter::new(dao.clone());
        analytics::hash::init(dao.clone()).await.unwrap();
        let versions = dao.list_versions().await.unwrap();
        start_http(config, dao, versions, writer).await;
        wait().await;
    } else {
        QpacktConfig::create().await;
    }
}

/// Waits for signal and exits
async fn wait() -> ! {
    let mut signal_interrupt = signal(SignalKind::interrupt()).unwrap();
    let mut signal_terminate = signal(SignalKind::terminate()).unwrap();

    select! {
        _ = signal_interrupt.recv() => {
            info!("received SIGINT, exiting...");
            tokio::time::sleep(Duration::from_millis(100)).await;
        },
        _ = signal_terminate.recv() => {
            info!("received SIGTERM, exiting...");
            tokio::time::sleep(Duration::from_millis(100)).await;
        },
    }
    std::process::exit(0);
}

/// Starts actix processes to serve admin's panel and http proxy.
async fn start_http(qpackt_config: QpacktConfig, dao: Dao, versions: Vec<Version>, writer: RequestWriter) {
    let qpackt_config = Data::new(qpackt_config);
    let servers = Versions::start(versions, qpackt_config.app_run_directory()).await;
    let dao = Data::new(dao);
    let servers = Data::new(servers);
    let writer = Data::new(writer);
    let ssl_challenge = AcmeChallenge::new().await;
    start_proxy_http(qpackt_config.http_proxy_addr(), servers.clone(), writer.clone(), Data::new(ssl_challenge.clone()));
    start_panel_http(qpackt_config.clone(), dao.clone(), servers.clone(), None);

    if let Some(https_proxy_addr) = qpackt_config.https_proxy_addr() {
        let certificate = get_certificate(qpackt_config.domain(), qpackt_config.app_run_directory(), ssl_challenge.clone()).await;
        ssl_challenge.clear().await;
        let resolver = try_build_resolver(certificate);
        let tls_config = ServerConfig::builder().with_safe_defaults().with_no_client_auth().with_cert_resolver(Arc::new(resolver));
        FORCE_HTTPS_REDIRECT.store(true, Ordering::Relaxed);
        start_proxy_https(https_proxy_addr, servers.clone(), writer.clone(), tls_config.clone());
        start_panel_http(qpackt_config, dao, servers.clone(), Some(tls_config));
    }
}

fn ensure_app_dir_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        create_app_dir(path)
    } else if !path.is_dir() {
        Err(QpacktError::InvalidConfig(format!("App dir is not a directory: {path:?}")))
    } else {
        Ok(())
    }
}

fn create_app_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path)
        .map_err(|e| QpacktError::InvalidConfig(format!("Unable to create app directory {}: {}", path.to_string_lossy(), e)))
}
