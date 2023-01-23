pub(crate) mod policy;
pub(crate) mod proxy;
pub(crate) mod repositories;

use crate::config::policy::PolicyConfig;
use crate::config::proxy::ProxyConfig;
use crate::config::repositories::Repositories;
use serde::{Deserialize, Serialize};
use std::io::Error as IoError;
use std::io::Read;
use toml::de::Error as TomlError;

#[derive(Debug)]
pub enum ConfigError {
    Serialization(TomlError),
    Io(IoError),
}

impl From<TomlError> for ConfigError {
    fn from(inner: TomlError) -> Self {
        Self::Serialization(inner)
    }
}

impl From<IoError> for ConfigError {
    fn from(inner: IoError) -> Self {
        Self::Io(inner)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    #[serde(default)]
    proxy: ProxyConfig,
    policy: PolicyConfig,
    #[serde(default)]
    repositories: Repositories,
}

impl Config {
    pub fn new<R: Read>(
        mut input: R,
        bind_override: Option<String>,
        port_override: Option<u16>,
    ) -> Result<Self, ConfigError> {
        let mut vec = Vec::new();
        let len = input.read_to_end(&mut vec)?;
        let mut config: Config = toml::from_slice(&vec[0..len])?;

        if let Some(bind) = bind_override {
            *config.proxy_mut().bind_mut() = bind;
        }

        if let Some(port) = port_override {
            *config.proxy_mut().port_mut() = port;
        }

        Ok(config)
    }

    pub fn proxy(&self) -> &ProxyConfig {
        &self.proxy
    }

    pub(crate) fn proxy_mut(&mut self) -> &mut ProxyConfig {
        &mut self.proxy
    }

    pub fn policy(&self) -> &PolicyConfig {
        &self.policy
    }

    pub fn repositories(&self) -> &Repositories {
        &self.repositories
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::repositories::RepositoryType;
    use crate::policy::Decision;
    use url::Url;

    #[test]
    fn empty_config() {
        let config: Result<Config, _> = toml::from_str(
            r#"
        "#,
        );

        assert!(config.is_err());
    }

    #[test]
    fn basic_config() {
        let config: Config = toml::from_str(
            r#"
            [policy]
            url = 'http://localhost:8080/'
        "#,
        )
        .unwrap();

        println!("{:?}", config);

        assert_eq!(
            Url::parse("http://localhost:8080/").unwrap(),
            config.policy.url()
        );
        assert_eq!(Decision::Deny, config.policy.default_decision());
    }

    #[test]
    fn basic_config_default_allow() {
        let config: Config = toml::from_str(
            r#"
            [policy]
            url = 'http://localhost:8080/'
            default = "allow"
        "#,
        )
        .unwrap();

        println!("{:?}", config);

        assert_eq!(
            Url::parse("http://localhost:8080/").unwrap(),
            config.policy.url()
        );
        assert_eq!(Decision::Allow, config.policy.default_decision());
    }

    #[test]
    fn full_config() {
        let config: Config = toml::from_str(
            r#"
            [proxy]
            bind = "0.0.0.0"
            port = 8181

            [policy]
            url = 'http://localhost:8080/'
            enforce = false

            [repositories.crates-io]
            type = "crates"
            url = "https://crates.io/"

            [repositories.m2]
            type = "m2"
            url = "https://repo.maven.apache.org/maven2"
        "#,
        )
        .unwrap();

        assert_eq!(
            Url::parse("http://localhost:8080/").unwrap(),
            config.policy.url()
        );
        assert_eq!(Decision::Deny, config.policy.default_decision());

        let mut repo_iter = config.repositories.iter();

        let crates_io = repo_iter.next().unwrap();
        assert_eq!("crates-io", crates_io.0);
        assert_eq!(RepositoryType::Crates, crates_io.1.repository_type());

        let m2 = repo_iter.next().unwrap();
        assert_eq!("m2", m2.0);
        assert_eq!(RepositoryType::M2, m2.1.repository_type());

        assert!(repo_iter.next().is_none())
    }
}
