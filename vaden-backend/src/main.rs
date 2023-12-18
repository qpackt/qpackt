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
mod config;
pub mod constants;
pub mod dao;
mod error;
mod manager;
mod panel;
mod password;
mod proxy;

use crate::config::Config;
use crate::constants::VERSIONS_SUBDIRECTORY;
use crate::dao::{Dao, Version};
use crate::error::Result;
use crate::error::VadenError;
use crate::panel::start_panel_http;
use crate::proxy::start_proxy_http;
use actix_files::Files;
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use log::{error, info};
use std::net::{Ipv4Addr, SocketAddrV4};
use std::path::Path;
use std::time::Duration;
use std::{env, fs};
use tokio::select;
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use url::Url;

#[tokio::main]
async fn main() {
    env::set_var("RUST_LOG", "debug");
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    if let Some(config_path) = args.get(1) {
        let config = Config::read(config_path).await.unwrap();
        ensure_app_dir_exists(config.app_run_directory()).unwrap();
        let dao = Dao::init(config.app_run_directory()).await.unwrap();
        let versions = dao.list_versions().await.unwrap();
        let (panel_handle, proxy_handle) = start_http(config, dao, versions).await;
        wait(panel_handle, proxy_handle).await;
    } else {
        Config::create().await;
    }
}

/// Waits on things like http servers and signals.
/// This function should never exit. When waiting is done that
/// means either there was some problem with one of http server
/// or some signal was received.
/// SIGHUP is not supported yet.
async fn wait(
    panel_handle: JoinHandle<std::io::Result<()>>,
    proxy_handle: JoinHandle<std::io::Result<()>>,
) -> ! {
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
        _ = proxy_handle => {
            error!("Proxy exited");
        },
        _ = panel_handle => {
            error!("Panel exited");
        }
    }
    std::process::exit(0);
}

/// Starts two actix processes to serve admin's panel and http proxy.
/// Returns two join handles so later then can be awaited.
async fn start_http(
    config: Config,
    dao: Dao,
    versions: Vec<Version>,
) -> (
    JoinHandle<std::io::Result<()>>,
    JoinHandle<std::io::Result<()>>,
) {
    let config = Data::new(config);
    let dao = Data::new(dao);
    let mut versions = build_version_handlers(versions);
    start_version_servers(&mut versions, config.app_run_directory()).await;
    let versions = Data::new(RwLock::new(versions));
    let panel_handle = start_panel_http(config.clone(), dao, versions.clone());
    let proxy_handle = start_proxy_http(config.proxy_addr(), versions);
    (panel_handle, proxy_handle)
}

/// Starts a normal 'Files' server to serve a version.
/// Doesn't check for 'Inactive' strategy because there may be sessions using this version and started before
/// this version became inactive.
async fn start_version_servers(handlers: &mut Vec<VersionHandler>, run_dir: &Path) {
    for handler in handlers {
        info!(
            "Starting version {} on port {}",
            handler.version.name, handler.port
        );
        start_version_server(handler, run_dir).await;
    }
}

async fn start_version_server(version_handler: &mut VersionHandler, run_dir: &Path) {
    let path = run_dir
        .join(VERSIONS_SUBDIRECTORY)
        .join(&version_handler.version.web_root);
    let task = tokio::spawn({
        HttpServer::new(move || {
            App::new().service(Files::new("/", path.clone()).index_file("index.html"))
        })
        .bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, version_handler.port))
        .unwrap()
        .run()
    });
    version_handler.task = Some(task);
}

pub(crate) struct VersionHandler {
    port: u16,
    upstream: Url,
    version: Version,
    task: Option<JoinHandle<std::io::Result<()>>>,
}

fn build_version_handlers(versions: Vec<Version>) -> Vec<VersionHandler> {
    let mut handlers = Vec::with_capacity(versions.len());
    for (id, version) in versions.into_iter().enumerate() {
        let port = 9000 + id as u16;
        handlers.push(VersionHandler {
            port,
            upstream: Url::parse(format!("http://localhost:{}", port).as_str()).unwrap(),
            version,
            task: None,
        })
    }
    handlers
}

fn ensure_app_dir_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        create_app_dir(path)
    } else if !path.is_dir() {
        Err(VadenError::InvalidConfig(format!(
            "App dir is not a directory: {path:?}"
        )))
    } else {
        Ok(())
    }
}

fn create_app_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path).map_err(|e| {
        VadenError::InvalidConfig(format!(
            "Unable to create app directory {}: {}",
            path.to_string_lossy(),
            e
        ))
    })
}
