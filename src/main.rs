pub mod config;
pub mod errors;
pub mod policy;
pub mod proxy;
pub mod repositories;
pub mod sigstore;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    /*
    HttpServer::new(|| {
        App::new()
            .app_data(
                web::Data::new(ProxyState::new())
            )
            .service(repositories::crates::service("crates") )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
     */
    Ok(())
}
