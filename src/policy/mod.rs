use async_trait::async_trait;

use serde::Serialize;

pub mod opa;

#[derive(Serialize)]
pub struct Context {}

#[derive(Copy, Clone, Debug)]
pub enum Decision {
    Allow,
    Deny,
}

pub struct ExplainedDecision {
    decision: Decision,
    audit: String,
}

#[async_trait]
pub trait Policy {
    async fn evaluate(&self, context: &Context) -> Decision;
}

pub struct Policies {
    default_decision: Decision,
    policies: Vec<Box<dyn Policy>>,
}

impl Policies {
    pub fn new(default_decision: Decision) -> Self {
        Self {
            default_decision,
            policies: Vec::new(),
        }
    }

    pub async fn evaluate(&self, context: &Context) -> Decision {
        if self.policies.is_empty() {
            return self.default_decision;
        }

        for policy in self.policies.iter() {
            if let Decision::Deny = policy.evaluate(context).await {
                return Decision::Deny;
            }
        }

        Decision::Allow
    }
}
