use opa_client::OpenPolicyAgentClient;

use crate::config::repositories::RepositoryType;
use crate::config::Config;
use crate::policy::PolicyEngine;
use crate::{repositories, ui};
use actix_web::middleware::Logger;
use actix_web::{get, web, App, HttpServer, Responder, HttpRequest, HttpResponse, ResponseError};
use std::marker::{PhantomData, Send, Sync};
use std::sync::Arc;

use actix_web::dev::HttpServiceFactory;

//use actix_web::{get, web, App, HttpServer, Responder};

#[derive(Clone)]
pub struct ProxyState<T: OpenPolicyAgentClient> {
    _policy: Arc<PolicyEngine<T>>,
}

impl<T: OpenPolicyAgentClient> ProxyState<T> {
    pub fn new(policy: PolicyEngine<T>) -> Self {
        Self {
            _policy: Arc::new(policy),
        }
    }
}

pub struct Proxy<T: OpenPolicyAgentClient> {
    config: Config,
    phantom: PhantomData<T>,
}

impl<T: 'static> Proxy<T>
where
    T: OpenPolicyAgentClient + Clone + Send + Sync,
{
    pub fn new(config: Config) -> Self {
        Self {
            config,
            phantom: PhantomData,
        }
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        let bind_args: (String, u16) = self.config.proxy().into();

        let policy_engine: PolicyEngine<T> = PolicyEngine::new(self.config.policy());

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
            let mut app = App::new()
                .wrap(Logger::default())
                .app_data(web::Data::new(proxy_state.clone()));

            for service in self.config.repositories().iter().map(|(scope, config)| {
                match config.repository_type() {
                    RepositoryType::Crates => repositories::crates::service(scope, config.url()), // this
                                                                                                  // sends
                                                                                                  // to
                                                                                                  // proxy
                                                                                                  // service
                    RepositoryType::M2 => repositories::maven::service(scope),
                }
            }) {
                app = app.service(service)
            }

            app.service(ui::service(self.config.clone()))
                //.default_service(web::to(repositories::crates::service(scope, config.url())))
                .default_service(web::to( | req: HttpRequest | async move {
                                    log::info!("i am handling the request: {:?}\n", req);
                                    
                                    let req_path = req.uri().path();
                                    log::info!("req path is {:?}\n", req_path);

                                    let mut extended_path = String::new();
                                    extended_path += "https://crates.io";
                                    extended_path += req_path;

                                    log::info!("new path is {:?}\n", extended_path);

                                    let body = reqwest::get(extended_path)
                                        .await.expect("request failed")
                                        .text()
                                        .await.expect("getting response text failed");

                                    //log::info!("body = {:?}", body);

                                    HttpResponse::Ok()
                                    }
                                         ))
        });

        log::info!("seedwing at http://{}:{}/", bind_args.0, bind_args.1);
        log::info!("========================================================================");
        server.workers(1).bind(bind_args)?.run().await
    }
}
