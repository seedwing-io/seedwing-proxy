use actix_web::dev::HttpServiceFactory;
use actix_web::web;
use crates_io_api::AsyncClient;

pub mod api;

pub struct CratesState {
    client: AsyncClient,
}

impl CratesState {
    pub fn new() -> Self {
        let client = AsyncClient::new(
            "seedwing-io (bmcwhirt@redhat.com)",
            std::time::Duration::from_millis(1000),
        )
        .expect("Unable to construct crates.io async client");

        Self { client }
    }
}

pub fn service(scope: &str) -> impl HttpServiceFactory {
    web::scope(&format!("{scope}/api/v1"))
        .app_data(web::Data::new(CratesState::new()))
        .service(api::v1::service())
}
