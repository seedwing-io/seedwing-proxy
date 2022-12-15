use actix_web::{get, web, Responder, Scope};

pub fn service(scope: &str) -> Scope {
    web::scope(&format!("{scope}/")).service(nothing)
}

#[get("/")]
async fn nothing() -> impl Responder {
    actix_web::HttpResponse::NotFound().body("not found")
}
