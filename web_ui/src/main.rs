use std::path::PathBuf;
use actix_files::NamedFile;
use actix_web::dev::PeerAddr;
use actix_web::{HttpServer, App, Result, web, HttpResponse, HttpRequest};
use actix_web::error::ErrorNotFound;
use actix_web::middleware::Logger;
use actix_web_opentelemetry::RequestTracing;
use actix_web_opentelemetry::ClientExt;
use awc::Client;
use log::{debug, info};

use service::{telemetry::init_telemetry, config::CONFIG};
use url::Url;

async fn index() -> Result<NamedFile> {
    let path: PathBuf = "web_ui/static/index.html".parse().unwrap();
    Ok(NamedFile::open(path)?)
}

// TODO - think of a better location for this bundle
async fn bundle() -> Result<NamedFile> {
    let path: PathBuf = "web_ui/static/editor.bundle.js".parse().unwrap();
    Ok(NamedFile::open(path)?)
}

async fn forward(
    req: HttpRequest,
    payload: web::Payload,
    peer_addr: Option<PeerAddr>,
    url: web::Data<Url>,
    client: web::Data<Client>,
) -> Result<HttpResponse, actix_web::Error> {
    debug!("forwarding request to api");
    let mut new_url = (**url).clone();
    if let Some(path) = req.uri().path().strip_prefix("/api") {
        new_url.set_path(path);
    } else {
        return Err(ErrorNotFound("bad path"))
    }
    new_url.set_query(req.uri().query());

    let forwarded_req = client
        .request_from(new_url.as_str(), req.head())
        .no_decompress();

    let forwarded_req = match peer_addr {
        Some(PeerAddr(addr)) => {
            forwarded_req.insert_header(("x-forwarded-for", addr.ip().to_string()))
        }
        None => forwarded_req,
    };

    let res = forwarded_req
        .trace_request()
        .send_stream(payload)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let mut client_resp = HttpResponse::build(res.status());
    // Remove `Connection` as per
    // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Connection#Directives
    for (header_name, header_value) in res.headers().iter().filter(|(h, _)| *h != "connection") {
        client_resp.insert_header((header_name.clone(), header_value.clone()));
    }

    Ok(client_resp.streaming(res))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_telemetry();

    info!("Starting");

    let forward_url = Url::parse("http://localhost:8081").unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(Client::default()))
            .app_data(web::Data::new(forward_url.clone()))
            .wrap(Logger::default())
            .wrap(RequestTracing::new())
            .route("/", web::get().to( index))
            .route("/editor.bundle.js", web::get().to( bundle))
            .service(web::scope("/api").default_service(web::to(forward)))
    })
        .bind(("127.0.0.1", CONFIG.ui_port))?
        .run()
        .await
}
