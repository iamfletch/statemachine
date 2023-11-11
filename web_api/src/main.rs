use actix_web::{HttpServer, App, web};
use actix_web::middleware::Logger;
use log::info;
use tracing_actix_web::TracingLogger;
use service::config::CONFIG;
use service::telemetry::init_tracing;

mod handlers;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let _ = init_tracing();

    info!("Starting");

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(TracingLogger::default())
            .route("/query", web::post().to(handlers::query))
    })
        .bind(("127.0.0.1", CONFIG.port))?
        .run()
        .await
}

