use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProxyConfig {
    #[serde(default = "default_bind")]
    bind: String,
    #[serde(default = "default_port")]
    port: u16,
    #[serde(default = "default_cache_dir")]
    cache_dir: String,
    #[serde(default = "default_git_cmd")]
    git_cmd: String,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            bind: default_bind(),
            port: default_port(),
            cache_dir: default_cache_dir(),
            git_cmd: default_git_cmd(),
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

    pub fn cache_dir(&self) -> String {
        self.cache_dir.clone()
    }

    pub fn cache_dir_mut(&mut self) -> &mut String {
        &mut self.cache_dir
    }

    pub fn expanded_cache_dir(&self) -> PathBuf {
        let mut cache_dir = self.cache_dir.trim();
        let mut tilde = false;
        cache_dir = match cache_dir.strip_prefix("~/") {
            Some(cache_dir) => {
                tilde = true;
                cache_dir
            }
            None => cache_dir,
        };
        while let Some(stripped) = cache_dir.strip_suffix('/') {
            cache_dir = stripped
        }
        if tilde {
            PathBuf::from(format!("{}/{}", env!("HOME"), cache_dir))
        } else {
            PathBuf::from(cache_dir)
        }
    }

    pub fn git_cmd(&self) -> String {
        self.git_cmd.clone()
    }
}

// Used to create address binding information for the HTTP server
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

fn default_cache_dir() -> String {
    String::from("~/.seedwing_proxy/cache")
}

fn default_git_cmd() -> String {
    String::from("git")
}
