use crate::config::policy::PolicyConfig;
use crate::policy::context::Context;
use actix_web::{error::ErrorInternalServerError, HttpResponse, HttpResponseBuilder};
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

    /// Query the policy server
    ///
    /// Returns Ok(None) when the Context successfully matches the
    /// policy.
    ///
    /// A response is returned only if the policy match fails
    ///
    pub async fn evaluate(
        &self,
        context: &Context,
    ) -> Result<Option<HttpResponse>, actix_web::Error> {
        let client = awc::Client::default(); // TODO: better place for this?
        match client
            .post(self.config.url().as_str())
            .send_json(context)
            .await
        {
            Ok(mut response) => {
                if response.status().is_success() {
                    Ok(None)
                } else {
                    match response.body().await {
                        Ok(payload) => {
                            let reason = String::from_utf8(payload.to_vec()).unwrap();
                            log::warn!(
                                "Access Denied!\n status: {}\n reason: {}",
                                response.status(),
                                reason,
                            );
                            let mut result = HttpResponseBuilder::new(response.status());
                            for header in response.headers().iter() {
                                result.insert_header(header);
                            }
                            Ok(Some(result.body(payload)))
                        }
                        Err(e) => Err(actix_web::Error::from(e)),
                    }
                }
            }
            Err(e) => match self.config.default_decision() {
                Decision::Allow => Ok(None),
                Decision::Deny => Err(ErrorInternalServerError(e)),
            },
        }
    }
}
