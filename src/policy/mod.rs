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

pub struct ExplainedDecision {
    _decision: Decision,
    _audit: String,
}

#[derive(Deserialize)]
struct Result {
    // intentionally empty, we don't care the content.
}

#[derive(Clone)]
pub struct PolicyEngine {
    default_decision: Decision,
}

impl PolicyEngine {
    pub fn new(config: &PolicyConfig) -> Self {
        Self {
            default_decision: config.default_decision(),
        }
    }

    pub async fn evaluate(&mut self, _context: &Context) -> Decision {
        self.default_decision
    }
}
