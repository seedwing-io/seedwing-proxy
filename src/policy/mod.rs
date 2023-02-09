use crate::config::policy::PolicyConfig;
use crate::policy::context::Context;
use actix_web::{error::ErrorInternalServerError, HttpResponse, HttpResponseBuilder};
use serde::{Deserialize, Serialize};

pub mod context;

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq)]
pub enum Decision {
    #[serde(rename = "disable")]
    Disable,
    #[serde(rename = "warn")]
    Warn,
    #[serde(rename = "enforce")]
    Enforce,
}

impl Default for Decision {
    fn default() -> Self {
        Self::Disable
    }
}

#[derive(Clone)]
pub struct PolicyEngine {
    config: PolicyConfig,
    pub client: awc::Client,
}

impl PolicyEngine {
    pub fn new(config: PolicyConfig) -> Self {
        let client = awc::Client::default();
        Self { config, client }
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
        if let Decision::Disable = self.config.decision() {
            // short-circuit if policy checking is disabled
            return Ok(None);
        }
        match self
            .client
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
                            if let Decision::Enforce = self.config.decision() {
                                let mut result = HttpResponseBuilder::new(response.status());
                                for header in response.headers().iter() {
                                    result.insert_header(header);
                                }
                                Ok(Some(result.body(payload)))
                            } else {
                                Ok(None)
                            }
                        }
                        Err(e) => Err(actix_web::Error::from(e)),
                    }
                }
            }
            Err(e) => {
                log::warn!("Unable to query policy server: {e}");
                Err(ErrorInternalServerError(e))
            }
        }
    }
}
