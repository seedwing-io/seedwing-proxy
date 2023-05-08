use actix_web::{route, web, HttpRequest, HttpResponse, HttpResponseBuilder, Responder, Scope};
use url::Url;

use crate::policy::PolicyEngine;

pub struct PipConfig {
    url: Url,
}

impl Default for PipConfig {
    fn default() -> Self {
        Self::new("https://pypi.org/simple".try_into().unwrap())
    }
}

impl PipConfig {
    pub fn new(url: Url) -> Self {
        Self { url }
    }
}

pub fn service(scope: &str, url: Url) -> Scope {
    web::scope(scope)
        .app_data(web::Data::new(PipConfig::new(url)))
        .service(pass_through)
}

#[route("{any:.*}", method = "GET", method = "HEAD", method = "POST")]
async fn pass_through(
    req: HttpRequest,
    payload: web::Payload,
    config: web::Data<PipConfig>,
    policy: web::Data<PolicyEngine>,
    path: web::Path<String>,
) -> impl Responder {
    let path = path.into_inner();
    let uri = format!("{}{path}", config.url);
    log::debug!("pass: {uri}");
    let request = policy.client.request_from(uri, req.head()).no_decompress();
    log::debug!("request: {request:?}");
    match request.send_stream(payload).await {
        Ok(upstream) => {
            let mut response = HttpResponseBuilder::new(upstream.status());
            for header in upstream.headers().iter() {
                response.insert_header(header);
            }
            response.streaming(upstream)
        }
        Err(e) => {
            log::error!("proxy error: {}", e);
            HttpResponse::NotFound().body("not found")
        }
    }
}
