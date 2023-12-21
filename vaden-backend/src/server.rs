use crate::constants::VERSIONS_SUBDIRECTORY;
use crate::dao::Version;
use actix_files::Files;
use actix_web::{App, HttpServer};
use log::{debug, info, warn};
use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::path::Path;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use url::Url;

/// All servers for versions will be started on localhost:port where starting port is the value below.
const START_PORT: u16 = 9000;

/// Contains details for various versions' servers.  
pub(crate) struct Versions {
    versions: RwLock<Vec<VersionServer>>,
    cookie_map: RwLock<HashMap<String, Url>>,
}

pub(crate) struct VersionServer {
    port: u16,
    pub(crate) upstream: Url,
    // TODO hide
    pub(crate) version: Version,
    task: Option<JoinHandle<std::io::Result<()>>>,
}

impl Versions {
    pub(super) async fn start(versions: Vec<Version>, run_dir: &Path) -> Self {
        let mut versions = build_version_servers(versions);
        start_version_servers(&mut versions, run_dir).await;
        Self { versions: RwLock::new(versions), cookie_map: RwLock::new(Default::default()) }
    }

    pub(crate) async fn get_upstream(&self, query: &str) -> Option<Url> {
        let versions = self.versions.read().await;
        versions.get(0).map(|v| v.upstream.clone())
    }

    pub(super) async fn update_strategies(&self, new: &[Version]) {
        let mut versions = self.versions.write().await;
        for current_version in versions.iter_mut() {
            for updated in new {
                if updated.name == current_version.version.name {
                    current_version.version.strategy = updated.strategy.clone();
                    break;
                }
            }
        }
    }

    pub(super) async fn delete_version(&self, name: &str) {
        let mut versions = self.versions.write().await;
        for (i, v) in versions.iter().enumerate() {
            if v.version.name == name {
                let version = versions.remove(i);
                if let Some(task) = version.task {
                    task.abort();
                } else {
                    warn!("No running task for version `{}` that's being removed", name);
                }
                break;
            }
        }
    }

    pub(super) async fn add_version(&self, version: Version, run_dir: &Path) {
        let mut versions = self.versions.write().await;
        let next_port = versions.iter().map(|v| v.port).max().unwrap_or(START_PORT) + 1;
        let mut server = build_version_server(version, next_port);
        start_version_server(&mut server, run_dir).await;
        versions.push(server);
    }

    pub(super) async fn get_url_for_cookie(&self, cookie: &str) -> Option<Url> {
        let lock = self.cookie_map.read().await;
        lock.get(cookie).cloned()
    }

    pub(super) async fn save_cookie_url(&self, value: &str, url: Url) {}
}

/// Starts a normal 'Files' server to serve a version.
/// Doesn't check for 'Inactive' strategy because there may be sessions using this version and started before
/// this version became inactive.
async fn start_version_servers(handlers: &mut Vec<VersionServer>, run_dir: &Path) {
    for handler in handlers {
        info!("Starting version {} on port {}", handler.version.name, handler.port);
        start_version_server(handler, run_dir).await;
    }
}

async fn start_version_server(version_handler: &mut VersionServer, run_dir: &Path) {
    let path = run_dir.join(VERSIONS_SUBDIRECTORY).join(&version_handler.version.web_root);
    let task = tokio::spawn({
        HttpServer::new(move || App::new().service(Files::new("/", path.clone()).index_file("index.html")))
            .bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, version_handler.port))
            .unwrap()
            .run()
    });
    version_handler.task = Some(task);
}

fn build_version_servers(versions: Vec<Version>) -> Vec<VersionServer> {
    let mut handlers = Vec::with_capacity(versions.len());
    for (id, version) in versions.into_iter().enumerate() {
        let port = START_PORT + id as u16;
        handlers.push(build_version_server(version, port))
    }
    handlers
}

fn build_version_server(version: Version, port: u16) -> VersionServer {
    VersionServer { port, upstream: Url::parse(format!("http://localhost:{}", port).as_str()).unwrap(), version, task: None }
}
