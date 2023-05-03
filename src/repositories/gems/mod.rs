use actix_web::{
    get, route, web, Error, HttpRequest, HttpResponse, HttpResponseBuilder, Responder, Scope,
};
use url::Url;

use crate::policy::{context::Context, PolicyEngine};

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
        .service(proxy)
        .service(pass_through)
}

#[get(r"{pkg}/{name:.*}-{version:[0-9]+\.[0-9]+.*}.{ext}")]
async fn proxy(
    req: HttpRequest,
    config: web::Data<GemsConfig>,
    policy: web::Data<PolicyEngine>,
    path: web::Path<(String, String, String, String)>,
) -> impl Responder {
    let (pkg, name, version, ext) = path.into_inner();
    let uri = format!("{}{pkg}/{name}-{version}.{ext}", config.url);
    log::debug!("upstream: {uri}");
    let mut request = policy.client.request_from(&uri, req.head()).no_decompress();
    request.headers_mut().remove("keep-alive");
    log::debug!("request: {request:?}");
    match request.send().await {
        Ok(mut upstream) => match upstream.body().limit(20_000_000).await {
            Ok(payload) => {
                let context = Context::new(
                    format!("pkg:gem/{name}@{version}"),
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

#[route("{any:.*}", method = "GET", method = "HEAD")]
async fn pass_through(
    req: HttpRequest,
    payload: web::Payload,
    config: web::Data<GemsConfig>,
    policy: web::Data<PolicyEngine>,
    path: web::Path<String>,
) -> impl Responder {
    let path = path.into_inner();
    let uri = format!("{}{path}", config.url);
    log::debug!("pass: {uri}");
    let mut request = policy.client.request_from(&uri, req.head()).no_decompress();
    request.headers_mut().remove("keep-alive");
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
