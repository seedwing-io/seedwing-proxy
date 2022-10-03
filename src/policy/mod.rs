use opa_client::OpenPolicyAgentClient;

use crate::config::policy::PolicyConfig;
use crate::policy::context::Context;
use serde::{Deserialize, Serialize};
use url::Url;

pub mod context;

#[derive(Deserialize, Copy, Clone, Debug, PartialEq, Eq)]
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

pub struct ExplainedDecision {
    decision: Decision,
    audit: String,
}

#[derive(Deserialize)]
struct Result {
    // intentionally empty, we don't care the content.
}

#[derive(Clone)]
pub struct PolicyEngine {
    default_decision: Decision,
    opa: OpenPolicyAgentClient,
    policy: String,
}

impl PolicyEngine {
    pub fn new(config: &PolicyConfig) -> Self {
        Self {
            default_decision: config.default_decision(),
            opa: OpenPolicyAgentClient::new(config.url()),
            policy: config.policy(),
        }
    }

    pub async fn evaluate(&self, context: &Context) -> Decision {
        if let Ok(result) = self.opa.query::<_, Result>(&self.policy, context).await {
            if result.is_some() {
                return Decision::Allow;
            }
        }

        self.default_decision
    }
}
