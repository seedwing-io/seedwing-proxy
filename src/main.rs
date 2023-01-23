use crate::cli::cli;
use crate::config::Config;
use crate::proxy::Proxy;
use std::fs::File;
use std::path::PathBuf;

pub mod cli;
pub mod config;
pub mod errors;
pub mod policy;
pub mod proxy;
pub mod repositories;
pub mod sigstore;
pub mod ui;

pub const DEFAULT_CONFIG: &str = "seedwing.toml";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let command = cli();

    let matches = command.get_matches();

    let config_toml: PathBuf = matches.get_one("config").cloned().unwrap_or_else(|| {
        if let Ok(pwd) = std::env::current_dir() {
            pwd.join(DEFAULT_CONFIG)
        } else {
            PathBuf::new().join(DEFAULT_CONFIG)
        }
    });

    let bind = matches.get_one("bind").cloned();
    let port = matches.get_one("port").cloned();

    if let Ok(config_toml) = File::open(config_toml.clone()) {
        match Config::new(config_toml, bind, port) {
            Ok(config) => {
                let proxy: Proxy = Proxy::new(config);
                proxy.run().await
            }
            Err(err) => {
                eprintln!("Unable to read the configuration file {:?}", err);
                std::process::exit(-1);
            }
        }
    } else {
        eprintln!(
            "Unable to locate configuration file {}",
            config_toml.to_str().unwrap()
        );
        std::process::exit(-2);
    }
}
