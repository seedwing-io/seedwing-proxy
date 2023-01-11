use crate::repositories::crates::CratesState;
use actix_web::dev::{HttpServiceFactory,PeerAddr};
use actix_web::{error, web, Error, HttpRequest, HttpResponse};

pub mod v1;

// Mostly copied from actix http-proxy example code

async fn forward(
    req: HttpRequest,
    payload: web::Payload,
    peer_addr: Option<PeerAddr>,
    crates: web::Data<CratesState>,
) -> Result<HttpResponse, Error> {
    log::info!("forward url: {}, path: {:?}", crates.url, req.path());

    let mut repo_url = crates.url.clone();
    let req_path = req.uri().path();
    let repo_path = repo_url.path();

    let scope = crates.scope.as_str();
    let new_path = format!("{repo_path}{}", req_path.strip_prefix(scope).unwrap());

    log::info!("Forwarding uri: {req_path} in scope {scope}");
    repo_url.set_path(new_path.as_str());
    repo_url.set_query(req.uri().query());

    log::info!("Forwarding request to: {repo_url}");
    let forwarded_req = crates.awc
        .request_from(repo_url.as_str(), req.head())
        .no_decompress();

    // TODO: This forwarded implementation is incomplete as it only handles the unofficial
    // X-Forwarded-For header but not the official Forwarded one.
    let forwarded_req = match peer_addr {
        Some(PeerAddr(addr)) => {
            forwarded_req.insert_header(("x-forwarded-for", addr.ip().to_string()))
        }
        None => forwarded_req,
    };

    let res = forwarded_req
        .send_stream(payload)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let mut client_resp = HttpResponse::build(res.status());
    // Remove `Connection` as per
    // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Connection#Directives
    for (header_name, header_value) in res.headers().iter().filter(|(h, _)| *h != "connection") {
        client_resp.insert_header((header_name.clone(), header_value.clone()));
    }

    Ok(client_resp.streaming(res))
}

pub fn proxy_service(scope: &str) -> impl HttpServiceFactory {
    log::info!("{scope}");
    web::resource(scope).to(forward)
}
