use actix_web::{web, HttpRequest, HttpResponse, HttpResponseBuilder, Responder};

pub async fn proxy(req: HttpRequest, payload: web::Payload) -> impl Responder {
    log::debug!("incoming {:?}", req);
    let client = awc::Client::default();
    let request = client.request_from(
        // TODO: un-hardcode
        format!(
            "https://github.com/rust-lang/crates.io-index{}",
            req.uri()
                .path_and_query()
                .unwrap()
                .as_str()
                .strip_prefix("/crates-io")
                .unwrap()
        ),
        req.head(),
    );
    log::debug!("outgoing {:?}", request);
    match request.send_stream(payload).await {
        Ok(upstream) => {
            let mut response = HttpResponseBuilder::new(upstream.status());
            for header in upstream.headers().iter() {
                response.insert_header(header);
            }
            let result = response.streaming(upstream);
            log::debug!("returning {:?}", result);
            result
        }
        Err(e) => {
            log::error!("proxy error: {}", e);
            HttpResponse::NotFound().body("not found")
        }
    }
}
