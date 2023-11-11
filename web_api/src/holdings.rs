pub struct TraceInterceptor;

impl Interceptor for TraceInterceptor {
    fn call(&mut self, mut request: Request<()>) -> std::result::Result<Request<()>, Status> {
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
            .span_builder((default_span_namer)(&request))
            .with_kind(SpanKind::Client)
            .with_attributes(mem::take(&mut attr))
            .start_with_context(&tracer, &parent_cx);

        let cx = parent_cx.with_span(span);

        global::get_text_map_propagator(|injector| {
            injector.inject_context(&cx, &mut TonicClientCarrier::new(&mut request));
        });

        Ok(request)
    }
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



#[derive(Default, Debug, Clone)]
pub struct TraceLayer;

impl<S> Layer<S> for TraceLayer {
    type Service = TraceService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        TraceService { inner }
    }
}

#[derive(Debug, Clone)]
pub struct TraceService<S> {
    inner: S
}

impl<T, ReqBody, ResBody> GrpcService<ReqBody> for TraceService<T>
    where
        T: GrpcService<ReqBody, ResponseBody = ResBody>,
        T::Error: std::error::Error,
        ResBody: http_body::Body
{
    type ResponseBody = ResBody;
    type Error = T::Error;
    type Future = T::Future;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> Poll<std::result::Result<(), Self::Error>> {
        info!("layer poll_ready");
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: tonic::codegen::http::Request<ReqBody>) -> Self::Future {
        info!("layer call");
        self.inner.call(request)
    }
}

