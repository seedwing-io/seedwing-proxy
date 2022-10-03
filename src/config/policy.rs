use serde::Deserialize;
use url::Url;

#[derive(Deserialize, Clone, Debug)]
pub struct PolicyConfig {
    url: Url,
    policy: String,
    #[serde(default = "default_enforce")]
    enforce: bool,
}

impl PolicyConfig {
    pub fn url(&self) -> Url {
        self.url.clone()
    }

    pub fn policy(&self) -> String {
        self.policy.clone()
    }

    pub fn enforce(&self) -> bool {
        self.enforce
    }
}

fn default_enforce() -> bool {
    true
}
