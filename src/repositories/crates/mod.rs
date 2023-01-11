use actix_web::{web, Scope};
use crates_io_api::AsyncClient;
use url::Url;
use awc::Client;

pub mod api;

pub struct CratesState {
    client: AsyncClient,
    scope: String,
    url: Url,
    awc: Client,
}

impl Default for CratesState {
    fn default() -> Self {
        Self::new("/crates-io", Url::parse("https://crates.io/").expect("Could not parse default URL"))
    }
}

impl CratesState {
    pub fn new(scope: &str, url: Url) -> Self {
        let client = AsyncClient::new(
            "seedwing-io (bmcwhirt@redhat.com)",
            std::time::Duration::from_millis(1000),
        )
        .expect("Unable to construct crates.io async client");

        let awc = Client::default();
        let scope = String::from(scope);
        Self { client, scope, url, awc }
    }
}

pub fn service(scope: &str, url: Url) -> Scope {
    let scope = format!("/{scope}");
    log::info!("Creating cargo service with scope {scope} and url {url}");
    web::scope(&scope)
        .app_data(web::Data::new(CratesState::new(&scope, url)))
        .service(web::scope("/api/v1").service(api::v1::service()))
        .service(api::proxy_service("/info/refs"))
        .service(api::proxy_service("/git-upload-pack"))
}
