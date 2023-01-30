use crate::policy::PolicyEngine;
use crate::repositories::crates::CratesState;
use crate::sigstore::search;
use actix_web::dev::HttpServiceFactory;
use actix_web::{get, web, HttpResponse, Responder};
use awc::http::header;

#[get("/{version}/download")]
async fn download(
    path: web::Path<(String, String)>,
    crates: web::Data<CratesState>,
    policy: web::Data<PolicyEngine>,
) -> impl Responder {
    let (crate_name, version) = path.into_inner();
    log::info!("download {} {}", crate_name, version);

    let client = &crates.client;

    if let Ok(info) = client.get_crate(&crate_name).await {
        if let Some(crate_version) = info.versions.iter().find(|e| e.num == version) {
            let link = &crate_version.dl_path;

            if let Ok(mut upstream) = policy
                .client
                .get(format!("https://crates.io/{link}"))
                .send()
                .await
            {
                if let Ok(payload) = upstream.body().limit(20_000_000).await {
                    let digest = sha256::digest(payload.as_ref());
                    let uuids = search(digest.clone()).await;
                    println!("{crate_name} {version} = {digest} {uuids:?}");

                    let mut response = HttpResponse::Ok();
                    if let Some(v) = upstream.headers().get(header::CONTENT_TYPE) {
                        response.append_header((header::CONTENT_TYPE, v));
                    }
                    let disposition =
                        format!("attachment; filename=\"{crate_name}-{version}.crate\"");
                    response.append_header((header::CONTENT_DISPOSITION, disposition));
                    return response.body(payload);
                }
            }
        }
    }

    HttpResponse::NotFound().body("not found")
}

pub fn service() -> impl HttpServiceFactory {
    web::scope("/crates/{crate_name}").service(download)
}
