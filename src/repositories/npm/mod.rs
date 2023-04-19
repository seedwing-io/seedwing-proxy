use actix_web::{
    get, route, web, Error, HttpRequest, HttpResponse, HttpResponseBuilder, Responder, Scope,
};
use url::Url;

use crate::policy::{context::Context, PolicyEngine};

pub struct NpmConfig {
    url: Url,
}

impl Default for NpmConfig {
    fn default() -> Self {
        Self::new("https://registry.npmjs.org".try_into().unwrap())
    }
}

impl NpmConfig {
    pub fn new(url: Url) -> Self {
        Self { url }
    }
}

pub fn service(scope: &str, url: Url) -> Scope {
    web::scope(scope)
        .app_data(web::Data::new(NpmConfig::new(url)))
        .service(proxy)
        .service(pass_through)
}

#[get("{pkg:.*}/-/{name:.*}-{version}.{ext}")]
async fn proxy(
    req: HttpRequest,
    config: web::Data<NpmConfig>,
    policy: web::Data<PolicyEngine>,
    path: web::Path<(String, String, String, String)>,
) -> impl Responder {
    let (pkg, name, version, ext) = path.into_inner();
    let uri = format!("{}{pkg}/-/{name}-{version}.{ext}", config.url);
    log::debug!("upstream -> {uri}");
    let request = policy.client.request_from(&uri, req.head());
    match request.send().await {
        Ok(mut upstream) => match upstream.body().limit(20_000_000).await {
            Ok(payload) => {
                let context = Context::new(
                    format!("pkg:npm/{}@{version}", pkg.replace('@', "%40")),
                    uri,
                    sha256::digest(payload.as_ref()),
                );
                match policy.evaluate(&context, Some(&ext)).await {
                    Ok(None) => {
                        let mut response = HttpResponseBuilder::new(upstream.status());
                        for header in upstream.headers().iter() {
                            response.insert_header(header);
                        }
                        response.body(payload)
                    }
                    Ok(Some(response)) => response,
                    Err(e) => e.into(),
                }
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

#[route("{any:.*}", method = "GET", method = "HEAD", method = "POST")]
async fn pass_through(
    req: HttpRequest,
    payload: web::Payload,
    config: web::Data<NpmConfig>,
    policy: web::Data<PolicyEngine>,
    path: web::Path<String>,
) -> impl Responder {
    let path = path.into_inner();
    let uri = format!("{}{path}", config.url,);
    log::debug!("pass: {uri}");
    let request = policy.client.request_from(uri, req.head());
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
