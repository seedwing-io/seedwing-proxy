use crate::repositories::crates::CratesState;
use actix_web::dev::HttpServiceFactory;
use actix_web::{get, web, HttpResponse, Responder};
use awc::http::header;
use sigstore::rekor::apis::configuration::Configuration;
use sigstore::rekor::apis::{entries_api, index_api};
use sigstore::rekor::models::SearchIndex;

#[get("/{version}/download")]
async fn download(
    path: web::Path<(String, String)>,
    crates: web::Data<CratesState>,
) -> impl Responder {
    let (crate_name, version) = path.into_inner();

    let client = &crates.client;

    if let Ok(info) = client.get_crate(&*crate_name).await {
        if let Some(crate_version) = info.versions.iter().find(|e| e.num == version) {
            let link = &crate_version.dl_path;

            let client = awc::Client::default();
            if let Ok(mut response) = client.get(format!("https://crates.io/{link}")).send().await {
                if let Ok(payload) = response.body().limit(20_000_000).await {
                    let digest = sha256::digest(
                        std::str::from_utf8(&payload).expect("could not parse Bytes"),
                    );

                    let query = SearchIndex {
                        email: None,
                        public_key: None,
                        hash: Some(digest.clone()),
                    };

                    let configuration = Configuration::default();

                    let uuid_vec = index_api::search_index(&configuration, query).await;

                    println!("{crate_name} {version} = {digest} {:?}", uuid_vec);

                    if let Ok(uuid_vec) = uuid_vec {
                        for uuid in uuid_vec.iter() {
                            let entry =
                                entries_api::get_log_entry_by_uuid(&configuration, uuid).await;
                            if let Ok(entry) = entry {
                                println!("{:?}", entry);
                            }
                        }
                    }

                    let content_type = response.headers().get(header::CONTENT_TYPE);

                    let mut response = HttpResponse::Ok();
                    if let Some(content_type) = content_type {
                        response.append_header((header::CONTENT_TYPE, content_type));
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
