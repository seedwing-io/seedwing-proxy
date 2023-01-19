use crate::repositories::crates::CratesState;
use actix_web::dev::HttpServiceFactory;
use actix_web::{get, web, HttpResponse, Responder};
use awc::http::header;
use sigstore::rekor::apis::configuration::Configuration;
use sigstore::rekor::apis::{entries_api, index_api};
use sigstore::rekor::models::SearchIndex;

use std::io::prelude::*;

#[get("/{version}/download")]
async fn download(
    path: web::Path<(String, String)>,
    crates: web::Data<CratesState>,
) -> impl Responder {
    let (crate_name, version) = path.into_inner();
    log::info!("download {} {}", crate_name, version);

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

pub fn modify_index() {
    log::info!("inside modify index");

    // set up modded git index; potentially better with git2 library;
    // download every time to get fresh version?
    let output = std::process::Command::new("git")
        .arg("clone")
        .arg("https://github.com/rust-lang/crates.io-index.git")
        .output()
        .expect("unable to clone crates io index");

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    log::info!("manifest dir: {:?}", manifest_dir);

    let path = std::path::PathBuf::from(manifest_dir);
    log::info!("path: {:?}", path);

    let path = path.join("crates.io-index/config.json");
    log::info!("file path: {:?}", path);

    //let mut file = std::fs::File::create(file_path)
    //    .expect("could not create config.json");
    
    std::fs::write(path, "{\n\"dl\": \"http://localhost:8181/api/v1/crates\",\n\"api\": \"https://crates.io\"\n}")
        .expect("could not write to config.json");

    log::info!("succeeded modifying index");
}

pub fn service() -> impl HttpServiceFactory {
    log::info!("inside crates service");

    modify_index();

    web::scope("/crates/{crate_name}").service(download)
}
