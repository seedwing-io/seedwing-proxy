use crate::policy::{context::Context, PolicyEngine};
use actix_web::{
    route, web, Error, HttpRequest, HttpResponse, HttpResponseBuilder, Responder, Scope,
};
use url::Url;
use urlencoding::encode;

pub struct MavenConfig {
    url: Url,
}

impl Default for MavenConfig {
    fn default() -> Self {
        Self::new("https://repo.maven.apache.org/maven2".try_into().unwrap())
    }
}

impl MavenConfig {
    pub fn new(url: Url) -> Self {
        Self { url }
    }
}

pub fn service(scope: &str, url: Url) -> Scope {
    web::scope(scope)
        .app_data(web::Data::new(MavenConfig::new(url)))
        .service(proxy)
}

#[route(
    "{group:.*}/{artifact}/{version}/{file}",
    method = "GET",
    method = "HEAD"
)]
async fn proxy(
    req: HttpRequest,
    config: web::Data<MavenConfig>,
    policy: web::Data<PolicyEngine>,
    path: web::Path<(String, String, String, String)>,
) -> impl Responder {
    let (group, artifact, version, file) = path.into_inner();
    let uri = format!("{}/{}/{}/{}/{}", config.url, group, artifact, version, file);
    log::debug!("upstream -> {uri}");
    let request = policy.client.request_from(&uri, req.head());
    match request.send().await {
        Ok(mut upstream) => match upstream.body().limit(20_000_000).await {
            Ok(payload) => {
                let context = Context::new(
                    format!(
                        "pkg:maven/{}/{artifact}@{version}?type={}&repository_url={}",
                        group.replace('/', "."),
                        file.rsplit_once('.').unwrap_or(("", "unknown")).1,
                        encode(config.url.as_str())
                    ),
                    uri,
                    sha256::digest(payload.as_ref()),
                );
                match policy.evaluate(&context, Some("jar")).await {
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
