use actix_web::{Responder, web, Result};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Request {
    pub first_name: String,
    pub last_name: String,
    pub ip_address: Option<String>
}

#[derive(Serialize)]
pub struct Response {
    pub name: String
}

pub async fn query(request: web::Json<Request>) -> Result<impl Responder> {
    Ok(web::Json(
        Response {
            name: format!("{} {}", request.first_name, request.last_name)
        })
    )
}