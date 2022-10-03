use crate::config::repositories::RepositoryType;
use crate::config::Config;
use crate::repositories;
use actix_web::{web, App, HttpServer};

pub struct ProxyState {}

impl ProxyState {
    pub fn new() -> Self {
        Self {}
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
        let server = HttpServer::new(move || {
            let mut app = App::new().app_data(web::Data::new(ProxyState::new()));

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
        server.bind(bind_args)?.run().await
    }
}
