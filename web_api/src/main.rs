use actix_web::{HttpServer, App, web};
use actix_web_opentelemetry::RequestTracing;
use actix_web::middleware::Logger;
use log::info;
use opentelemetry_api::trace::{TracerProvider};
use service::{telemetry::init_telemetry, config::CONFIG};


mod handlers;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_telemetry();

    info!("Starting");
    let channel = tonic::transport::Channel::from_static("http://[::1]:50051")
        .connect_lazy();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(channel.clone()))
            .wrap(Logger::default())
            .wrap(RequestTracing::new())
            .route("/query", web::post().to(handlers::query))
    })
        .bind(("127.0.0.1", CONFIG.port))?
        .run()
        .await
}

