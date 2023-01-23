use crate::policy::Decision;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PolicyConfig {
    #[serde(rename = "default", default)]
    default_decision: Decision,
    url: Url,
}

impl PolicyConfig {
    pub fn default_decision(&self) -> Decision {
        self.default_decision
    }

    pub fn url(&self) -> Url {
        self.url.clone()
    }
}
