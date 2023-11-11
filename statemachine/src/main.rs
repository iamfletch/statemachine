use std::any::Any;
use log::info;
use service::telemetry::{init_telemetry, init_tracing};
use tonic::{transport::Server, Request, Response, Status};
use statemachine_grpc::say_server::{Say, SayServer};
use statemachine_grpc::{SayResponse, SayRequest};
use opentelemetry_api::{Context, global, KeyValue, propagation::Extractor, trace::TracerProvider};
use opentelemetry_api::trace::{SpanKind, TraceContextExt, Tracer};
use tonic::codegen::tokio_stream::StreamExt;
use tonic::metadata::{KeyRef, MetadataMap};
use tonic_tracing_opentelemetry::middleware::filters;
use tonic_tracing_opentelemetry::middleware::server::OtelGrpcLayer;

#[derive(Default)]
pub struct MySay {}

// implementing rpc for service defined in .proto
#[tonic::async_trait]
impl Say for MySay {
// our rpc impelemented as function
    async fn send(&self,request:Request<SayRequest>)->Result<Response<SayResponse>,Status>{
// returning a response as SayResponse message as defined in .proto
        Ok(Response::new(SayResponse{
// reading data from request which is a wrapper around our SayRequest message defined in .proto
                message:format!("hello {}",request.get_ref().name),
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    // defining address for our service
    let addr = "[::1]:50051".parse().unwrap();
    // creating a service
    let say = MySay::default();
    info!("Server listening on {}", addr);
    // adding our service to our server.
    Server::builder()
        .layer(OtelGrpcLayer::default().filter(filters::reject_healthcheck))
        .add_service(SayServer::new(say))
        .serve(addr)
        .await?;
    Ok(())
}


fn intercept(req: Request<()>) -> Result<Request<()>, Status> {
    println!("Intercepting request: {:?}", req);
    let parent_context = global::get_text_map_propagator(|propagator| {
        propagator.extract(&TonicClientHeaderCarrier::new(req.metadata()))
    });
    let tracer = global::tracer_provider().versioned_tracer(
        "tonic-opentelemetry-request",
        Some(env!("CARGO_PKG_VERSION")),
        Some(opentelemetry_semantic_conventions::SCHEMA_URL),
        None,
    );
    let mut span_builder = tracer.span_builder("test");
    span_builder.span_kind = Some(SpanKind::Server);

    let span = tracer.build_with_context(span_builder, &parent_context);
    let cx = parent_context.with_span(span);

    Ok(req)
}

struct TonicClientHeaderCarrier<'a> {
    map: &'a MetadataMap
}

impl<'a> TonicClientHeaderCarrier<'a> {
    fn new(map: &'a MetadataMap) -> Self {
        TonicClientHeaderCarrier { map }
    }
}

impl<'a> Extractor for TonicClientHeaderCarrier<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.map.get(key).and_then(|v| v.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.map.keys()
            .map(|k| match k {
                KeyRef::Ascii(k) => k.as_str(),
                KeyRef::Binary(k) => k.as_str()
            })
            .collect()
    }
}