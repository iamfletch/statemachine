use std::mem;
use std::str::FromStr;
use actix_web::{Responder, web, Result};
use log::info;
use serde::{Deserialize, Serialize};
use statemachine_grpc::say_client::SayClient;
use statemachine_grpc::SayRequest;
use opentelemetry_api::{Context, global, KeyValue, propagation::Injector, trace::TracerProvider};
use opentelemetry_api::trace::{SpanKind, TraceContextExt, Tracer};
use tonic::metadata::{MetadataKey, MetadataValue};
use tonic::{Request, Status};
use tonic::transport::Channel;

#[derive(Deserialize)]
pub struct WebRequest {
    pub first_name: String,
    pub last_name: String,
    pub ip_address: Option<String>
}

#[derive(Serialize)]
pub struct WebResponse {
    pub message: String
}

pub async fn query(request: web::Json<WebRequest>, channel: web::Data<Channel>) -> Result<impl Responder> {
    info!("Received query, forwarding to statemachine");

    // TODO - understand this better
    let mut client = SayClient::with_interceptor((**channel).clone(), interceptor);

    let req = tonic::Request::new(
        SayRequest {
           name: format!("{} {}", request.first_name, request.last_name)
        },
    );

    if let Ok(response) = client.send(req).await {
        return Ok(web::Json(WebResponse {
            message: format!("{}", response.into_inner().message)
        }));
    }

    Ok(web::Json(
        WebResponse {
            message: "failed".to_string()
        })
    )
}



fn interceptor(mut req: Request<()>) -> Result<Request<()>, Status> {
    let parent_cx = &Context::current();

    let tracer = global::tracer_provider().versioned_tracer(
        "tonic-opentelemetry-request",
        Some(env!("CARGO_PKG_VERSION")),
        Some(opentelemetry_semantic_conventions::SCHEMA_URL),
        None,
    );

    let mut attr: Vec<KeyValue> = Vec::with_capacity(1);
    attr.push(KeyValue::new("Testing", "hello world"));

    let span = tracer
        .span_builder((default_span_namer)(&req))
        .with_kind(SpanKind::Client)
        .with_attributes(mem::take(&mut attr))
        .start_with_context(&tracer, &parent_cx);

    let cx = parent_cx.with_span(span);

    global::get_text_map_propagator(|injector| {
        injector.inject_context(&cx, &mut TonicClientCarrier::new(&mut req));
    });

    Ok(req)
}


struct TonicClientCarrier<'a> {
    request: &'a mut Request<()>
}

impl<'a> TonicClientCarrier<'a> {
    fn new(request: &'a mut Request<()>) -> Self {
        TonicClientCarrier { request }
    }
}

impl<'a> Injector for TonicClientCarrier<'a> {
    fn set(&mut self, key: &str, value: String) {
        if let Ok(key) = MetadataKey::from_bytes(key.as_bytes()) {
            if let Ok(val) = MetadataValue::from_str(&value.as_str()) {
                self.request.metadata_mut().insert(key, val);
            }
        }
    }
}

fn default_span_namer(request: &Request<()>) -> String {
    info!("default_span_namer {:?}", request);
    format!(
        "{}",
        "testing"
    )
}