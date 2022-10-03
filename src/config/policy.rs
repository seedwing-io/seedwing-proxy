use crate::policy::Decision;
use serde::Deserialize;
use url::Url;

#[derive(Deserialize, Clone, Debug)]
pub struct PolicyConfig {
    #[serde(rename = "default", default)]
    default_decision: Decision,
    url: Url,
    policy: String,
}

impl PolicyConfig {
    pub fn default_decision(&self) -> Decision {
        self.default_decision
    }

    pub fn url(&self) -> Url {
        self.url.clone()
    }

    pub fn policy(&self) -> String {
        self.policy.clone()
    }
}
