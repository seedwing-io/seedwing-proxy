use crate::Config;
use actix_web::dev::HttpServiceFactory;
use actix_web::{get, web, HttpResponse, Responder};
use handlebars::Handlebars;
use serde_json::json;

pub struct UiState {
    config: Config,
}

const INDEX: &str = include_str!("index.html");

#[get("/")]
async fn index(config: web::Data<UiState>) -> impl Responder {
    let index = Handlebars::new();

    let result = index.render_template(
        INDEX,
        &json!( {
        "config": config.config,
            } ),
    );

    match result {
        Ok(rendered) => HttpResponse::Ok().body(rendered),
        Err(err) => {
            log::error!("{:?}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

const STYLE: &str = include_str!("style.css");

#[get("/style.css")]
async fn style() -> impl Responder {
    HttpResponse::Ok().content_type("text/css").body(STYLE)
}

pub fn service(config: Config) -> impl HttpServiceFactory {
    web::scope("")
        .app_data(web::Data::new(UiState { config }))
        .service(style)
        .service(index)
}
