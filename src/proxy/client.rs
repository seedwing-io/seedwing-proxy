use actix_web::{HttpRequest, HttpResponse, HttpResponseBuilder, Responder};

pub async fn proxy(req: HttpRequest) -> impl Responder {
    let client = awc::Client::default();
    let request = client.request_from(req.uri(), req.head());
    match request.send().await {
        Ok(upstream) => {
            let mut response = HttpResponseBuilder::new(upstream.status());
            for header in upstream.headers().iter() {
                response.insert_header(header);
            }
            response.streaming(upstream)
        }
        _ => HttpResponse::NotFound().body("not found"),
    }
}
