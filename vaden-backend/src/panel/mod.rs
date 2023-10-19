use crate::config::Config;
use crate::dao::Dao;
use crate::panel::upload_version::upload_version;
use crate::proxy::upstream::Upstreams;
use actix_files::Files;
use actix_web::web::Data;
use actix_web::{web, App, HttpServer};
use tokio::task::JoinHandle;
pub(super) mod upload_version;

pub(super) fn start_panel_http(
    upstreams: Data<Upstreams>,
    config: Data<Config>,
    dao: Data<Dao>,
) -> JoinHandle<std::io::Result<()>> {
    tokio::spawn({
        let app_config = config.clone();
        HttpServer::new(move || {
            App::new()
                .app_data(upstreams.clone())
                .app_data(app_config.clone())
                .app_data(dao.clone())
                .service(Files::new("/static", "../vaden-frontend/dist").index_file("index.html"))
                .service(web::resource("/upload-version").route(web::post().to(upload_version)))
        })
        .bind(config.panel_addr())
        .unwrap()
        .run()
    })
}
