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
pub mod dao;
mod error;
mod password;
pub mod proxy;

mod http;

use crate::config::Config;
use crate::dao::Dao;

use crate::http::start_http;
use log::info;
use std::env;
use std::time::Duration;
use tokio::select;
use tokio::signal::unix::{signal, SignalKind};
use tokio::task::JoinHandle;

#[tokio::main]
async fn main() {
    env::set_var("RUST_LOG", "debug");
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    if let Some(config_path) = args.get(1) {
        let config = Config::read(config_path).await.unwrap();
        Dao::init(config.app_run_directory()).await.unwrap();
        let (panel_handle, proxy_handle) = start_http(&config).await;
        wait(panel_handle, proxy_handle).await;
    } else {
        Config::create().await;
    }
}

/// Waits on things like http servers and signals.
/// This function should never exit, when waiting is done that
/// means either there was some problem with one of http server
/// or some signal was received.
/// SIGHUP is not supported at this time.
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
            println!("Proxy exited");
        },
        _ = panel_handle => {
            println!("Panel exited");
        }
    }
    std::process::exit(0);
}
