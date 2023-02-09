use crate::policy::Decision;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PolicyConfig {
    #[serde(default)]
    decision: Decision,
    url: Url,
}

impl PolicyConfig {
    pub fn decision(&self) -> Decision {
        self.decision
    }

    pub fn url(&self) -> Url {
        self.url.clone()
    }
}
