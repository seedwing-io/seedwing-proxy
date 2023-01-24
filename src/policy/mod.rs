use crate::config::policy::PolicyConfig;
use crate::policy::context::Context;
use serde::{Deserialize, Serialize};

pub mod context;

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq)]
pub enum Decision {
    #[serde(rename = "allow")]
    Allow,
    #[serde(rename = "deny")]
    Deny,
}

impl Default for Decision {
    fn default() -> Self {
        Self::Deny
    }
}

#[derive(Clone)]
pub struct PolicyEngine {
    config: PolicyConfig,
}

impl PolicyEngine {
    pub fn new(config: PolicyConfig) -> Self {
        Self { config }
    }

    pub async fn evaluate(&self, context: &Context) -> Decision {
        let client = awc::Client::default();
        match client
            .post(self.config.url().as_str())
            .send_json(context)
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    Decision::Allow
                } else {
                    Decision::Deny
                }
            }
            Err(_) => self.config.default_decision(),
        }
    }
}
