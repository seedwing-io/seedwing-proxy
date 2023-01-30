use actix_web::{web, Scope};
use crates_io_api::AsyncClient;

use self::git::IndexRepository;

pub mod api;

pub mod git;

pub struct CratesState {
    client: AsyncClient,
    scope: String,
    index_repository: IndexRepository,
    git_cmd: String,
}

impl CratesState {
    pub fn new(scope: &str, git_cmd: &str, index_repository: IndexRepository) -> Self {
        let client = AsyncClient::new(
            "seedwing-io (bmcwhirt@redhat.com)",
            std::time::Duration::from_millis(1000),
        )
        .expect("Unable to construct crates.io async client");

        let scope = String::from(scope);
        let git_cmd = String::from(git_cmd);
        Self {
            client,
            scope,
            index_repository,
            git_cmd,
        }
    }
}

pub fn service(scope: &str, git_cmd: &str, index_repository: git::IndexRepository) -> Scope {
    let scope = format!("/{scope}");
    log::info!("Creating cargo service with scope {scope}");
    web::scope(&scope)
        .app_data(web::Data::new(CratesState::new(
            &scope,
            git_cmd,
            index_repository,
        )))
        .service(web::scope("/api/v1").service(api::v1::service()))
        .service(git::git_backend_service("/info/refs"))
        .service(git::git_backend_service("/git-upload-pack"))
}
