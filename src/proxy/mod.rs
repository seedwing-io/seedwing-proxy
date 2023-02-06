use url::Url;

use crate::config::repositories::RepositoryType;
use crate::config::Config;
use crate::policy::PolicyEngine;
use crate::repositories::crates::git::IndexRepository;
use crate::repositories::crates::sparse::SparseRepository;
use crate::{repositories, ui};
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

const INDEX_PATH: &str = "/index";
const API_PATH: &str = "/api/v1";

pub struct Proxy {
    config: Config,
    crate_repositories: HashMap<String, IndexRepository>,
    crate_sparse_repositories: HashMap<String, SparseRepository>,
}

impl Proxy {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            crate_repositories: HashMap::new(),
            crate_sparse_repositories: HashMap::new(),
        }
    }

    pub async fn run(mut self) -> Result<(), std::io::Error> {
        let bind_args: (String, u16) = self.config.proxy().into();

        log::info!("========================================================================");
        log::info!("Policy server {}", self.config.policy().url());
        log::info!("------------------------------------------------------------------------");

        for (scope, config) in self.config.repositories().iter() {
            log::info!(
                "{} endpoint at http://{}:{}/{scope}/",
                config.repository_type(),
                bind_args.0,
                bind_args.1
            );
        }

        let base_cache_dir = self.config.proxy().expanded_cache_dir();

        for (scope, config) in self.config.repositories().iter() {
            match config.repository_type() {
                RepositoryType::Crates => {
                    log::info!(
                        "------------------------------------------------------------------------"
                    );
                    log::info!("Initialising Crate Repository for scope {scope}");
                    let index_repository = IndexRepository::new(
                        config.url(),
                        self.get_cache_dir(&base_cache_dir, scope, &bind_args.0, bind_args.1),
                        self.get_url(
                            scope,
                            &bind_args.0,
                            bind_args.1,
                            Some(&format!("{API_PATH}/crates")),
                        ),
                        self.get_url(scope, &bind_args.0, bind_args.1, None),
                        config.periodic_update(),
                    );
                    log::info!(
                        "    Crate repository       : {}",
                        index_repository.get_repo()
                    );
                    log::info!(
                        "    local repository cache : {}",
                        index_repository.get_local_repository_cache().display()
                    );
                    log::info!(
                        "    download URL           : {}",
                        index_repository.get_dl_url()
                    );
                    log::info!(
                        "    API URL                : {}",
                        index_repository.get_api_url()
                    );
                    log::info!(
                        "    Periodic Update        : {}",
                        index_repository.get_periodic_update()
                    );
                    self.crate_repositories
                        .insert(scope.to_string(), index_repository);
                    log::info!(
                        "------------------------------------------------------------------------"
                    );
                }
                RepositoryType::SparseCrates => {
                    log::info!("Initialising Crate Sparse Repository for scope {scope}");
                    let sparse_repository = SparseRepository::new(
                        config.url(),
                        format!("/{scope}{INDEX_PATH}"),
                        self.get_url(
                            scope,
                            &bind_args.0,
                            bind_args.1,
                            Some(&format!("{API_PATH}/crates")),
                        ),
                        self.get_url(scope, &bind_args.0, bind_args.1, None),
                    );
                    log::info!(
                        "    Crate sparse repository: {}",
                        sparse_repository.get_repo()
                    );
                    log::info!(
                        "    download URL           : {}",
                        sparse_repository.get_dl_url()
                    );
                    log::info!(
                        "    API URL                : {}",
                        sparse_repository.get_api_url()
                    );
                    log::info!(
                        "    Index Prefix           : {}",
                        sparse_repository.get_index_prefix()
                    );
                    self.crate_sparse_repositories
                        .insert(scope.to_string(), sparse_repository);
                    log::info!(
                        "------------------------------------------------------------------------"
                    );
                }
                RepositoryType::M2 => {}
            }
        }

        let server = HttpServer::new(move || {
            let mut app =
                App::new()
                    .wrap(Logger::default())
                    .app_data(web::Data::new(PolicyEngine::new(
                        self.config.policy().clone(),
                    )));

            for service in self.config.repositories().iter().map(|(scope, config)| {
                match config.repository_type() {
                    RepositoryType::Crates => {
                        let index_repository = self.crate_repositories.get(scope).unwrap();
                        let git_cmd = &self.config.proxy().git_cmd();
                        repositories::crates::service(
                            scope,
                            git_cmd,
                            index_repository.clone(),
                            API_PATH,
                        )
                    }
                    RepositoryType::M2 => repositories::maven::service(scope, config.url()),
                    RepositoryType::SparseCrates => {
                        let sparse_repository = self.crate_sparse_repositories.get(scope).unwrap();
                        repositories::crates::service_sparse(
                            scope,
                            sparse_repository.clone(),
                            INDEX_PATH,
                            API_PATH,
                        )
                    }
                }
            }) {
                app = app.service(service)
            }

            app.service(ui::service(self.config.clone()))
        });

        log::info!("seedwing at http://{}:{}/", bind_args.0, bind_args.1);
        log::info!("========================================================================");
        server.bind(bind_args)?.run().await
    }

    fn get_cache_dir(
        &self,
        base_cache_dir: &Path,
        name: &String,
        addr: &String,
        port: u16,
    ) -> PathBuf {
        PathBuf::from(format!(
            "{}/{}_{}_{}",
            base_cache_dir.display(),
            name,
            addr,
            port
        ))
    }

    fn get_url(&self, name: &String, addr: &String, port: u16, path: Option<&str>) -> Url {
        let addr = if "0.0.0.0" == addr { "127.0.0.1" } else { addr };
        match path {
            Some(path) => Url::parse(&format!("http://{addr}:{port}/{name}{path}")).unwrap(),
            None => Url::parse(&format!("http://{addr}:{port}/{name}")).unwrap(),
        }
    }
}
