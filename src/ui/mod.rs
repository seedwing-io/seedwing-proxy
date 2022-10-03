use actix_web::dev::HttpServiceFactory;
use actix_web::{get, web, App, HttpResponse, Responder};
use awc::http::StatusCode;

const INDEX: &str = include_str!("index.html");

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body(INDEX)
}

pub fn service() -> impl HttpServiceFactory {
    web::scope("").service(index)
}
