use crate::config::repositories::RepositoryType;
use crate::config::Config;
use crate::policy::PolicyEngine;
use crate::{repositories, ui};
use actix_web::{web, App, HttpServer};
use std::sync::Arc;

#[derive(Clone)]
pub struct ProxyState {
    policy: Arc<PolicyEngine>,
}

impl ProxyState {
    pub fn new(policy: PolicyEngine) -> Self {
        Self {
            policy: Arc::new(policy),
        }
    }
}

pub struct Proxy {
    config: Config,
}

impl Proxy {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        let bind_args: (String, u16) = self.config.proxy().into();

        let policy_engine = PolicyEngine::new(self.config.policy());

        log::info!("========================================================================");
        log::info!("OPA server {}", self.config.policy().url());
        log::info!("    policy {}", self.config.policy().policy());
        log::info!("------------------------------------------------------------------------");

        for (scope, config) in self.config.repositories().iter() {
            log::info!(
                "{} endpoint at http://{}:{}/{scope}/",
                config.repository_type(),
                bind_args.0,
                bind_args.1
            );
        }
        log::info!("------------------------------------------------------------------------");

        let proxy_state = ProxyState::new(policy_engine);

        let server = HttpServer::new(move || {
            let mut app = App::new().app_data(web::Data::new(proxy_state.clone()));

            app = app.service(ui::service());

            for service in self.config.repositories().iter().map(|(scope, config)| {
                match config.repository_type() {
                    RepositoryType::Crates => repositories::crates::service(scope),
                    RepositoryType::M2 => {
                        panic!("Maven m2 repositories not yet supported");
                    }
                }
            }) {
                app = app.service(service)
            }

            app
        });

        log::info!("seedwing at http://{}:{}/", bind_args.0, bind_args.1);
        log::info!("========================================================================");
        server.bind(bind_args)?.run().await
    }
}
