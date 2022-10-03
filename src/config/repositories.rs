use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use url::Url;

#[derive(Serialize, Deserialize, Clone, Debug)]
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

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq)]
pub enum RepositoryType {
    #[serde(rename = "crates")]
    Crates,
    #[serde(rename = "m2")]
    M2,
}

impl Display for RepositoryType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RepositoryType::Crates => {
                write!(f, "crates")
            }
            RepositoryType::M2 => {
                write!(f, "m2")
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
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
