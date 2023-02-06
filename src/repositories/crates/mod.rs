use actix_web::{web, Scope};
use awc::Client;
use crates_io_api::AsyncClient;

use self::{git::IndexRepository, sparse::SparseRepository};

pub mod api;

pub mod git;

pub mod sparse;

pub struct CratesDownloadConfig {
    scope: String,
    client: AsyncClient,
}

impl CratesDownloadConfig {
    pub fn new(scope: &str) -> Self {
        let scope = String::from(scope);
        let client = AsyncClient::new(
            "seedwing-io (seedwing@example.com)",
            std::time::Duration::from_millis(1000),
        )
        .expect("Unable to construct crates.io async client");

        Self { scope, client }
    }
}

pub struct CratesConfig {
    scope: String,
    index_repository: IndexRepository,
    git_cmd: String,
}

impl CratesConfig {
    pub fn new(scope: &str, git_cmd: &str, index_repository: IndexRepository) -> Self {
        let scope = String::from(scope);
        let git_cmd = String::from(git_cmd);
        Self {
            scope,
            index_repository,
            git_cmd,
        }
    }
}

pub struct CratesSparseConfig {
    scope: String,
    sparse_repository: SparseRepository,
    awc: Client,
}

impl CratesSparseConfig {
    pub fn new(scope: &str, sparse_repository: SparseRepository) -> Self {
        let scope = String::from(scope);
        let awc = Client::default();
        Self {
            scope,
            sparse_repository,
            awc,
        }
    }
}

pub fn service(
    scope: &str,
    git_cmd: &str,
    index_repository: IndexRepository,
    api_path: &str,
) -> Scope {
    let scope = format!("/{scope}");
    log::info!("Creating cargo service with scope {scope}");
    web::scope(&scope)
        .app_data(web::Data::new(CratesDownloadConfig::new(&scope)))
        .app_data(web::Data::new(CratesConfig::new(
            &scope,
            git_cmd,
            index_repository,
        )))
        .service(web::scope(api_path).service(api::v1::service()))
        .service(git::git_backend_service("/info/refs"))
        .service(git::git_backend_service("/git-upload-pack"))
}

pub fn service_sparse(
    scope: &str,
    sparse_repository: SparseRepository,
    index_path: &str,
    api_path: &str,
) -> Scope {
    let scope = format!("/{scope}");
    log::info!("Creating cargo sparse service with scope {scope}");
    web::scope(&scope)
        .app_data(web::Data::new(CratesDownloadConfig::new(&scope)))
        .app_data(web::Data::new(CratesSparseConfig::new(
            &scope,
            sparse_repository,
        )))
        .service(
            web::scope(index_path)
                .service(sparse::config_service("/config.json"))
                .default_service(sparse::proxy_service()),
        )
        .service(web::scope(api_path).service(api::v1::service()))
}
