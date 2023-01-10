use actix_web::{http::header, HttpRequest, HttpResponse, HttpResponseBuilder, Responder};

pub async fn proxy(req: HttpRequest) -> impl Responder {
    log::debug!("incoming {:?}", req);
    let client = awc::Client::default();
    let mut request = client.request_from(format!("https://crates.io{}", req.uri()), req.head());
    request = request.insert_header((header::HOST, "crates.io"));
    log::debug!("outgoing {:?}", request);
    match request.send().await {
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
