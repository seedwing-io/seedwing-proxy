use indexmap::IndexMap;
use serde::Deserialize;
use url::Url;

#[derive(Deserialize, Clone, Debug)]
pub struct Repositories(IndexMap<String, RepositoryConfig>);

impl Repositories {
    pub fn iter(&self) -> impl Iterator<Item = (&String, &RepositoryConfig)> {
        self.0.iter()
    }
}

impl Default for Repositories {
    fn default() -> Self {
        Self(Default::default())
    }
}

#[derive(Deserialize, Copy, Clone, Debug, PartialEq, Eq)]
pub enum RepositoryType {
    #[serde(rename = "crates")]
    Crates,
    #[serde(rename = "m2")]
    M2,
}

#[derive(Deserialize, Clone, Debug)]
pub struct RepositoryConfig {
    #[serde(rename = "type")]
    repository_type: RepositoryType,
    url: Url,
}

impl RepositoryConfig {
    pub fn repository_type(&self) -> RepositoryType {
        self.repository_type
    }

    pub fn url(&self) -> Url {
        self.url.clone()
    }
}
