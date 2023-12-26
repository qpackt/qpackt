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
mod server;

use crate::config::Config;
use crate::dao::version::Version;
use crate::dao::Dao;
use crate::error::Result;
use crate::error::VadenError;
use crate::panel::start_panel_http;
use crate::proxy::start_proxy_http;
use crate::server::Versions;
use actix_web::web::Data;
use log::{error, info};
use std::path::Path;
use std::time::Duration;
use std::{env, fs};
use tokio::select;
use tokio::signal::unix::{signal, SignalKind};
use tokio::task::JoinHandle;

#[tokio::main]
async fn main() {
    env::set_var("RUST_LOG", "info");
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
async fn wait(panel_handle: JoinHandle<std::io::Result<()>>, proxy_handle: JoinHandle<std::io::Result<()>>) -> ! {
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
) -> (JoinHandle<std::io::Result<()>>, JoinHandle<std::io::Result<()>>) {
    let config = Data::new(config);
    let servers = Versions::start(versions, config.app_run_directory()).await;
    let dao = Data::new(dao);
    let servers = Data::new(servers);
    let panel_handle = start_panel_http(config.clone(), dao, servers.clone());
    let proxy_handle = start_proxy_http(config.proxy_addr(), servers);
    (panel_handle, proxy_handle)
}

fn ensure_app_dir_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        create_app_dir(path)
    } else if !path.is_dir() {
        Err(VadenError::InvalidConfig(format!("App dir is not a directory: {path:?}")))
    } else {
        Ok(())
    }
}

fn create_app_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path)
        .map_err(|e| VadenError::InvalidConfig(format!("Unable to create app directory {}: {}", path.to_string_lossy(), e)))
}
