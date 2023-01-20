use actix_web::{route, web, HttpRequest, HttpResponse, HttpResponseBuilder, Responder, Scope};
use awc::Client;
use url::Url;

pub struct MavenState {
    client: Client,
    url: Url,
    scope: String,
}

impl Default for MavenState {
    fn default() -> Self {
        Self::new(
            "https://repo.maven.apache.org/maven2".try_into().unwrap(),
            "/m2",
        )
    }
}

impl MavenState {
    pub fn new(url: Url, scope: &str) -> Self {
        let client = Client::default();
        let scope = scope.to_string();
        Self { client, url, scope }
    }
    // Strips the scope out of the path, leaving the query in
    // tact. Now deprecated in favor of extracting GAV path segments
    // in the proxy handler
    pub fn upstream_uri(&self, req: &HttpRequest) -> String {
        format!(
            "{}{}",
            self.url,
            req.uri()
                .path_and_query()
                .unwrap()
                .as_str()
                .strip_prefix(&format!("/{}", self.scope))
                .unwrap()
        )
    }
}

pub fn service(scope: &str, url: Url) -> Scope {
    web::scope(scope)
        .app_data(web::Data::new(MavenState::new(url, scope)))
        .service(proxy)
}

#[route(
    "{group:.*}/{artifact}/{version}/{file}",
    method = "GET",
    method = "HEAD"
)]
async fn proxy(
    req: HttpRequest,
    state: web::Data<MavenState>,
    path: web::Path<(String, String, String, String)>,
) -> impl Responder {
    let (group, artifact, version, file) = path.into_inner();
    let uri = format!("{}/{}/{}/{}/{}", state.url, group, artifact, version, file);
    log::debug!("upstream -> {uri}");
    let request = state.client.request_from(&uri, req.head());
    match request.send().await {
        Ok(upstream) => {
            let mut response = HttpResponseBuilder::new(upstream.status());
            for header in upstream.headers().iter() {
                response.insert_header(header);
            }
            response.streaming(upstream)
        }
        Err(e) => {
            let msg = format!("Error encountered proxying {uri} -> {e}");
            log::error!("{msg}");
            HttpResponse::NotFound().body(msg)
        }
    }
}
