use actix_web::{web, App, HttpServer};

pub mod crates;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service( web::scope("/crates").service(crates::service()))
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
