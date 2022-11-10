use opa_client::{Data, OpenPolicyAgentClient};

use crate::config::policy::PolicyConfig;
use crate::policy::context::Context;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use url::Url;

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

pub struct ExplainedDecision {
    decision: Decision,
    audit: String,
}

#[derive(Deserialize)]
struct Result {
    // intentionally empty, we don't care the content.
}

#[derive(Clone)]
pub struct PolicyEngine<T: OpenPolicyAgentClient> {
    default_decision: Decision,
    opa: T,
    policy: String,
}

impl<T: OpenPolicyAgentClient> PolicyEngine<T>
where
    T: OpenPolicyAgentClient,
{
    pub fn new(config: &PolicyConfig) -> Self {
        Self {
            default_decision: config.default_decision(),
            opa: <T>::new(config.url().to_string().as_bytes()).unwrap(),
            policy: config.policy(),
        }
    }

    pub async fn evaluate<O: DeserializeOwned>(&mut self, context: &Context) -> Decision {
        let dummy = Data { data: b"abcd" };

        if let Ok(result) = self
            .opa
            .query::<_, _, O>(&self.policy, context, &dummy)
            .await
        {
            if result.is_some() {
                return Decision::Allow;
            }
        }

        self.default_decision
    }
}
