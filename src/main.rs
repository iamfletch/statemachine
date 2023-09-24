use actix_web::{HttpServer, App, web};
use actix_web_opentelemetry::RequestTracing;
use actix_web::middleware::Logger;
use log::info;

use crate::{telemetry::init_telemetry, config::CONFIG};

mod telemetry;
mod config;
mod handlers;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_telemetry();

    info!("Starting");

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .wrap(RequestTracing::new())
            .route("/query", web::post().to(handlers::query))
    })
        .bind(("127.0.0.1", CONFIG.port))?
        .run()
        .await
}
