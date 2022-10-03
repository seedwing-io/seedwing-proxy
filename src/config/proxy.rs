use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProxyConfig {
    #[serde(default = "default_bind")]
    bind: String,
    #[serde(default = "default_port")]
    port: u16,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            bind: default_bind(),
            port: default_port(),
        }
    }
}

impl ProxyConfig {
    pub fn bind(&self) -> String {
        self.bind.clone()
    }

    pub(crate) fn bind_mut(&mut self) -> &mut String {
        &mut self.bind
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub(crate) fn port_mut(&mut self) -> &mut u16 {
        &mut self.port
    }
}

impl From<&ProxyConfig> for (String, u16) {
    fn from(config: &ProxyConfig) -> Self {
        (config.bind.clone(), config.port)
    }
}

fn default_bind() -> String {
    String::from("0.0.0.0")
}

const fn default_port() -> u16 {
    8675
}
