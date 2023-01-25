use crate::config::policy::PolicyConfig;
use crate::policy::context::Context;
use actix_web::error::{ErrorInternalServerError, ErrorNotAcceptable};
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

    pub async fn evaluate(&self, context: &Context) -> Result<(), actix_web::Error> {
        let client = awc::Client::default(); // TODO: better place for this?
        match client
            .post(self.config.url().as_str())
            .send_json(context)
            .await
        {
            Ok(mut response) => {
                if response.status().is_success() {
                    Ok(())
                } else {
                    let reason =
                        String::from_utf8(response.body().await.unwrap().to_vec()).unwrap();
                    log::warn!(
                        "Access Denied!\n status: {}\n reason: {}",
                        response.status(),
                        reason,
                    );
                    Err(ErrorNotAcceptable(reason).into())
                }
            }
            Err(e) => match self.config.default_decision() {
                Decision::Allow => Ok(()),
                Decision::Deny => Err(ErrorInternalServerError(e)),
            },
        }
    }
}
