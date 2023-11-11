use std::path::PathBuf;
use actix_files::NamedFile;
use actix_web::dev::PeerAddr;
use actix_web::{HttpServer, App, Result, web, HttpResponse, HttpRequest};
use actix_web::error::ErrorNotFound;
use actix_web::middleware::Logger;
use awc::Client;
use log::{error, info, trace};
use tracing_actix_web::TracingLogger;
use tracing_awc::Tracing;
use telemetry::{init_tracing};
use tracing::info_span;

use url::Url;

async fn index() -> Result<NamedFile> {
    let path: PathBuf = "static/index.html".parse().unwrap();
    Ok(NamedFile::open(path)?)
}

// TODO - think of a better location for this bundle
async fn bundle() -> Result<NamedFile> {
    let path: PathBuf = "static/editor.bundle.js".parse().unwrap();
    Ok(NamedFile::open(path)?)
}

async fn forward_handler(
    req: HttpRequest,
    payload: web::Payload,
    peer_addr: Option<PeerAddr>,
    url: web::Data<Url>,
    client: web::Data<Client>,
) -> Result<HttpResponse, actix_web::Error> {
    let span = info_span!("forward_handler");
    let _e = span.enter();

    let mut new_url = (**url).clone();
    let old_path = req.uri().path();
    if let Some(path) = old_path.strip_prefix("/api") {
        if path.len() == 0 {
            error!("missing target api in path {old_path}");
            return Err(ErrorNotFound("bad path"))
        }
        new_url.set_path(path);
    } else {
        error!("path must start with api {old_path}");
        return Err(ErrorNotFound("bad path"))
    }
    new_url.set_query(req.uri().query());
    info!("redirecting {} -> {}", old_path, new_url.as_str());

    let forwarded_req = client
        .request_from(new_url.as_str(), req.head())
        .no_decompress();
    trace!("build new client request");

    let forwarded_req = match peer_addr {
        Some(PeerAddr(addr)) => {
            forwarded_req.insert_header(("x-forwarded-for", addr.ip().to_string()))
        }
        None => forwarded_req,
    };
    trace!("added x-forwarded-for");

    info!("send request and await");
    let res = forwarded_req
        .send_stream(payload)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    trace!("generate response stream");
    let mut client_resp = HttpResponse::build(res.status());

    trace!("copy headers (except `Connection`)"); // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Connection#Directives
    for (header_name, header_value) in res.headers().iter().filter(|(h, _)| *h != "connection") {
        client_resp.insert_header((header_name.clone(), header_value.clone()));
    }

    Ok(client_resp.streaming(res))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_tracing();

    let span = info_span!("init").entered();

    tracing::info!("Create forward URL for api");
    let forward_url = Url::parse("http://localhost:8081").unwrap();

    span.exit();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(Client::builder()
                .wrap(Tracing)
                .finish()))
            .app_data(web::Data::new(forward_url.clone()))
            .wrap(Logger::default())
            .wrap(TracingLogger::default())
            .route("/", web::get().to( index))
            .route("/editor.bundle.js", web::get().to( bundle))
            .service(web::scope("/api").default_service(web::to(forward_handler)))
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
