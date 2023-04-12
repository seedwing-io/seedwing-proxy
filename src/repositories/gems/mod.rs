use actix_web::{
    route, web, Error, HttpRequest, HttpResponse, HttpResponseBuilder, Responder, Scope,
};
use url::Url;

use crate::policy::PolicyEngine;

pub struct GemsConfig {
    url: Url,
}

impl Default for GemsConfig {
    fn default() -> Self {
        Self::new("https://rubygems.org".try_into().unwrap())
    }
}

impl GemsConfig {
    pub fn new(url: Url) -> Self {
        Self { url }
    }
}

pub fn service(scope: &str, url: Url) -> Scope {
    web::scope(scope)
        .app_data(web::Data::new(GemsConfig::new(url)))
        .service(pass_through)
}

#[route("{any:.*}", method = "GET", method = "HEAD")]
async fn pass_through(
    req: HttpRequest,
    config: web::Data<GemsConfig>,
    policy: web::Data<PolicyEngine>,
    path: web::Path<String>,
) -> impl Responder {
    let path = path.into_inner();
    let uri = format!("{}{path}", config.url,);
    log::info!("pass: {uri} from: {req:?}");
    let request = policy.client.request_from(&uri, req.head());
    match request.send().await {
        Ok(mut upstream) => match upstream.body().limit(20_000_000).await {
            Ok(payload) => {
                let mut response = HttpResponseBuilder::new(upstream.status());
                for header in upstream.headers().iter() {
                    response.insert_header(header);
                }
                response.body(payload)
            }
            Err(e) => Error::from(e).into(),
        },
        Err(e) => {
            let msg = format!("Error encountered proxying {uri} -> {e}");
            log::error!("{msg}");
            HttpResponse::InternalServerError().body(msg)
        }
    }
}
