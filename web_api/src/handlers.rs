use actix_web::{Responder, web, Result};
use log::info;
use serde::{Deserialize, Serialize};
use statemachine_grpc::say_client::SayClient;
use statemachine_grpc::SayRequest;
use tonic::transport::Channel;
use tower::ServiceBuilder;

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

pub async fn query(request: web::Json<WebRequest>) -> Result<impl Responder> {
    info!("Received query, forwarding to statemachine");

    let channel = Channel::from_static("http://[::1]:50051").connect_lazy();

    let channel = ServiceBuilder::new()
        .layer(tower_http::trace::TraceLayer::new_for_grpc())
        .service(channel);

    let mut client = SayClient::new(channel);

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
