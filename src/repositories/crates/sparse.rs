use actix_web::dev::HttpServiceFactory;
use actix_web::http::header::{CONNECTION, HOST, UPGRADE};
use actix_web::http::StatusCode;
use actix_web::{error, guard, web, Error, HttpRequest, HttpResponse, Route};
use url::Url;

use super::CratesSparseConfig;

#[derive(Clone)]
pub struct SparseRepository {
    repo: Url,
    index_prefix: String,
    dl: Url,
    api: Url,
}

impl SparseRepository {
    pub fn new(repo: Url, index_prefix: String, dl: Url, api: Url) -> Self {
        Self {
            repo,
            index_prefix,
            dl,
            api,
        }
    }

    pub fn get_repo(&self) -> &Url {
        &self.repo
    }

    pub fn get_index_prefix(&self) -> &str {
        &self.index_prefix
    }

    pub fn get_dl_url(&self) -> &Url {
        &self.dl
    }

    pub fn get_api_url(&self) -> &Url {
        &self.api
    }
}

macro_rules! remove_headers {
    ( $h:expr, $($n:expr),+ ) => {{
            let mut headers = $h.clone();
            $(
                headers.remove($n);
            )*
            headers
        }
    };
}

async fn forward(
    req: HttpRequest,
    crates: web::Data<CratesSparseConfig>,
) -> Result<HttpResponse, Error> {
    let sparse_repository = &crates.sparse_repository;
    let mut repo_url = sparse_repository.repo.clone();
    let req_path = req.uri().path();

    let index_prefix = format!("{}/", &sparse_repository.index_prefix);
    let new_path = format!(
        "{}{}",
        repo_url.path(),
        req_path.strip_prefix(&index_prefix).unwrap()
    );

    repo_url.set_path(new_path.as_str());
    repo_url.set_query(req.uri().query());

    log::info!(
        "Forwarding uri: {req_path} in scope {} to: {repo_url}",
        &crates.scope
    );
    let mut forwarded_req = crates.awc.request(req.method().clone(), repo_url.as_str());

    forwarded_req.headers_mut().clone_from(&remove_headers!(
        req.headers(),
        CONNECTION,
        HOST,
        UPGRADE
    ));
    forwarded_req = forwarded_req.no_decompress();

    let res = forwarded_req
        .send()
        .await
        .map_err(error::ErrorInternalServerError)?;

    let headers = remove_headers!(res.headers(), CONNECTION);

    let mut client_resp = HttpResponse::build(res.status()).streaming(res);
    client_resp.headers_mut().clone_from(&headers);

    Ok(client_resp)
}

pub fn proxy_service() -> Route {
    web::get().to(forward)
}

async fn generate_config(crates: web::Data<CratesSparseConfig>) -> Result<HttpResponse, Error> {
    let sparse_repository = &crates.sparse_repository;
    let body = format!(
        "{{\n  \"dl\": \"{}\",\n  \"api\": \"{}\"\n}}\n",
        sparse_repository.dl, sparse_repository.api
    );

    let response = HttpResponse::build(StatusCode::OK)
        .content_type("application/json")
        .body(body);

    Ok(response)
}

pub fn config_service(scope: &str) -> impl HttpServiceFactory {
    web::resource(scope).guard(guard::Get()).to(generate_config)
}
