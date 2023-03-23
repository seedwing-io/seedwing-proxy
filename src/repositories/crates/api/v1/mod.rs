use crate::errors::Result;
use crate::policy::{context::Context, PolicyEngine};
use crate::repositories::crates::CratesDownloadConfig;
use actix_web::dev::HttpServiceFactory;
use actix_web::{get, web, HttpResponse, HttpResponseBuilder, Responder};

#[get("/{version}/download")]
async fn download(
    path: web::Path<(String, String)>,
    crates: web::Data<CratesDownloadConfig>,
    policy: web::Data<PolicyEngine>,
) -> Result<impl Responder> {
    let (crate_name, version) = path.into_inner();
    log::info!("download {} {}", crate_name, version);

    let client = &crates.client;
    let info = client.get_crate(&crate_name).await?;

    match info.versions.iter().find(|e| e.num == version) {
        None => {
            let msg = format!(
                "Error encountered finding version {version} of crate {crate_name} in crate info"
            );
            log::error!("{msg}");
            Ok(HttpResponse::NotFound().body("{msg}"))
        }
        Some(crate_version) => {
            let link = &crate_version.dl_path;
            let url = format!("https://crates.io/{link}");

            let mut upstream = policy.client.get(url.clone()).send().await?;
            let payload = upstream.body().limit(20_000_000).await?;

            let context = Context::new(
                format!("pkg:cargo/{crate_name}@{version}"),
                url,
                sha256::digest(payload.as_ref()), // todo: double check this
            );

            match policy.evaluate(&context).await? {
                None => {
                    log::info!("Policy evaluation success");
                    let mut response = HttpResponseBuilder::new(upstream.status());
                    for header in upstream.headers().iter() {
                        response.insert_header(header);
                    }
                    Ok(response.body(payload))
                }
                Some(err_response) => {
                    log::info!(
                        "Policy evaluation returned failure: {}",
                        err_response.status()
                    );
                    Ok(err_response)
                }
            }
        }
    }
}

pub fn service() -> impl HttpServiceFactory {
    web::scope("/crates/{crate_name}").service(download)
}
