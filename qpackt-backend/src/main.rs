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

use std::{env, fs};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

use actix_web::web::Data;
use log::info;
use rustls::ServerConfig;
use tokio::select;
use tokio::signal::unix::{signal, SignalKind};
use tokio::task::JoinHandle;

use crate::analytics::event_writer::EventWriter;
use crate::analytics::http_request_log_writer::HttpRequestLogWriter;
use crate::config::QpacktConfig;
use crate::dao::Dao;
use crate::dao::version::Version;
use crate::error::QpacktError;
use crate::error::Result;
use crate::panel::start_panel_http;
use crate::proxy::{start_proxy_http, start_proxy_https};
use crate::reverse_proxy::ReverseProxies;
use crate::server::Versions;
use crate::ssl::{FORCE_HTTPS_REDIRECT, get_certificate};
use crate::ssl::challenge::AcmeChallenge;
use crate::ssl::resolver::{read_intermediate_cert, try_build_resolver};

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

mod reverse_proxy;

#[cfg(test)]
mod tests;

#[tokio::main]
async fn main() {
    env::set_var("RUST_LOG", "debug");
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    run_with_args(args).await;
}

pub(crate) async fn run_with_args(args: Vec<String>) {
    if let Some(config_path) = args.get(1) {
        let handler = run_app(&PathBuf::from(config_path)).await;
        wait(handler).await;
    } else {
        QpacktConfig::create().await;
    }
}

pub(crate) async fn run_app(config_path: &PathBuf) -> JoinHandle<()> {
    let config = QpacktConfig::read(config_path).await.unwrap();
    ensure_app_dir_exists(config.app_run_directory()).unwrap();
    let dao = Dao::init(config.app_run_directory()).await.unwrap();
    let http_request_log_writer = HttpRequestLogWriter::new(dao.clone());
    let event_writer = EventWriter::new(dao.clone());
    analytics::hash::init(dao.clone()).await.unwrap();
    let versions = dao.list_versions().await.unwrap();
    tokio::spawn(start_http(config, dao, versions, http_request_log_writer, event_writer))
}

/// Waits for signal and exits
async fn wait(handler: JoinHandle<()>) -> ! {
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
    handler.abort();
    std::process::exit(0);
}

/// Starts actix processes to serve admin's panel and http proxy.
async fn start_http(qpackt_config: QpacktConfig, dao: Dao, versions: Vec<Version>, http_request_log_writer: HttpRequestLogWriter, event_writer: EventWriter) {
    let qpackt_config = Data::new(qpackt_config);
    let servers = Versions::start(versions, qpackt_config.app_run_directory()).await;
    let dao = Data::new(dao);
    let servers = Data::new(servers);
    let http_request_log_writer = Data::new(http_request_log_writer);
    let event_writer = Data::new(event_writer);
    let reverse_proxies = ReverseProxies::default();
    reverse_proxies.set(dao.list_reverse_proxies().await.unwrap()).await;
    let reverse_proxies = Data::new(reverse_proxies);
    let ssl_challenge = AcmeChallenge::new().await;
    start_proxy_http(
        qpackt_config.http_proxy_addr(),
        dao.clone(),
        servers.clone(),
        http_request_log_writer.clone(),
        Data::new(ssl_challenge.clone()),
        reverse_proxies.clone(),
        event_writer.clone(),
    );
    start_panel_http(qpackt_config.clone(), dao.clone(), servers.clone(), None, reverse_proxies.clone());

    if let Some(https_proxy_addr) = qpackt_config.https_proxy_addr() {
        let certificate = get_certificate(qpackt_config.domain(), qpackt_config.app_run_directory(), ssl_challenge.clone()).await;
        ssl_challenge.clear().await;
        let intermediate_cert = read_intermediate_cert(qpackt_config.app_run_directory());
        let resolver = try_build_resolver(certificate, intermediate_cert);
        let tls_config = ServerConfig::builder().with_safe_defaults().with_no_client_auth().with_cert_resolver(Arc::new(resolver));
        FORCE_HTTPS_REDIRECT.store(true, Ordering::Relaxed);
        start_proxy_https(https_proxy_addr, dao.clone(), servers.clone(), http_request_log_writer.clone(), tls_config.clone(), reverse_proxies.clone(), event_writer.clone());
        start_panel_http(qpackt_config, dao, servers.clone(), Some(tls_config), reverse_proxies);
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
