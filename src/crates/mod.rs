use actix_web::dev::HttpServiceFactory;
use actix_web::web;

pub mod api;

pub fn service() -> impl HttpServiceFactory {
    web::scope("/api/v1")
        .service(api::v1::service())
}