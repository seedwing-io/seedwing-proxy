use crate::policy::{Context, Decision, Policy};
use async_trait::async_trait;
use url::Url;

use opa_client::OpenPolicyAgentClient;
use serde::Deserialize;

pub struct OpenPolicyAgent {
    client: OpenPolicyAgentClient,
    policy: String,
}

#[derive(Deserialize)]
pub struct Result {
    // intentionally empty, we don't care the content.
}

impl OpenPolicyAgent {
    pub fn new(url: Url, policy: String) -> Self {
        Self {
            client: OpenPolicyAgentClient::new(url),
            policy,
        }
    }
}

#[async_trait]
impl Policy for OpenPolicyAgent {
    async fn evaluate(&self, context: &Context) -> Decision {
        if let Ok(result) = self.client.query::<_, Result>(&self.policy, context).await {
            if result.is_some() {
                return Decision::Allow;
            }
        }
        Decision::Deny
    }
}
