use url::Url;

use super::repositories::crates::git;
use crate::config::repositories::RepositoryType;
use crate::config::Config;
use crate::policy::PolicyEngine;
use crate::repositories::crates::git::IndexRepository;
use crate::{repositories, ui};
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer, HttpResponse};
use std::collections::HashMap;
use std::path::PathBuf;

pub struct Proxy {
    config: Config,
    crate_repositories: HashMap<String, IndexRepository>,
}

impl Proxy {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            crate_repositories: HashMap::new(),
        }
    }

    pub async fn run(mut self) -> Result<(), std::io::Error> {
        let bind_args: (String, u16,) = self.config.proxy().into();

        let policy_engine: PolicyEngine = PolicyEngine::new(self.config.policy().clone());

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
            if RepositoryType::Crates == config.repository_type() {
                log::info!("------------------------------------------------------------------------");
                log::info!("Initialising Crate Repository for scope {scope}");
                let index_repository = git::IndexRepository::new(config.url().clone(),
                    self.get_cache_dir(&base_cache_dir, scope, &bind_args.0, bind_args.1),
                    self.get_url(scope, &bind_args.0, bind_args.1, Some("/api/v1/crates")),
                    self.get_url(scope, &bind_args.0, bind_args.1, None));
                log::info!("    Crate repository       : {}", index_repository.get_repo());
                log::info!("    local repository cache : {}", index_repository.get_local_repository_cache().display());
                log::info!("    download URL           : {}", index_repository.get_dl_url());
                log::info!("    API URL                : {}", index_repository.get_api_url());
                if let Err(error) = index_repository.prepare_local_cache() {
                    log::info!("    Failed to initialize   : {error}");
                } else {
                    self.crate_repositories.insert(scope.to_string(), index_repository);
                }
                log::info!("------------------------------------------------------------------------");
            }
        }

        let server = HttpServer::new(move || {
            let mut app = App::new()
                .wrap(Logger::default())
                .app_data(web::Data::new(policy_engine.clone()));

            for service in self.config.repositories().iter().map(|(scope, config)| {
                match config.repository_type() {
                    RepositoryType::Crates => {
                        if let Some(index_repository) = self.crate_repositories.get(scope) {
                            let git_cmd = &self.config.proxy().git_cmd();
                            repositories::crates::service(scope,  git_cmd, index_repository.clone())
                        } else {
                            log::info!("Ignoring scope {scope} because of earlier initialisation failures");
                            web::scope(&scope)
                                .default_service(web::to(|| HttpResponse::NotFound()))
                        }
                    },
                    RepositoryType::M2 => repositories::maven::service(scope, config.url()),
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

    fn get_cache_dir(&self, base_cache_dir: &PathBuf, name: &String, addr: &String, port: u16) -> PathBuf {
        PathBuf::from(format!("{}/{}_{}_{}", base_cache_dir.display(), name, addr, port))
    }

    fn get_url(&self, name: &String, addr: &String, port: u16, path: Option<&str>) -> Url {
        let addr = if "0.0.0.0" == addr {
            "127.0.0.1"
        } else {
            addr
        } ;
        match path {
            Some(path) => Url::parse(&format!("http://{}:{}/{}{}", addr, port, name, path)).unwrap(),
            None => Url::parse(&format!("http://{}:{}/{}", addr, port, name)).unwrap()
        }
    }

}
