use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct SecurityConfig {
    #[serde(default = "default_true")]
    pub admin_token_verify_enabled: bool,
    #[serde(default = "default_true")]
    pub user_token_verify_enabled: bool,
    #[serde(default = "default_true")]
    pub admin_ip_bind_enabled: bool,
    #[serde(default)]
    pub trust_proxy_headers: bool,
    #[serde(default = "default_trusted_proxies")]
    pub trusted_proxies: Vec<String>,
}

fn default_true() -> bool {
    true
}

fn default_trusted_proxies() -> Vec<String> {
    vec!["127.0.0.1".to_string(), "::1".to_string()]
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            admin_token_verify_enabled: true,
            user_token_verify_enabled: true,
            admin_ip_bind_enabled: true,
            trust_proxy_headers: false,
            trusted_proxies: default_trusted_proxies(),
        }
    }
}

impl SecurityConfig {
    pub fn admin_token_verify_enabled(&self) -> bool {
        self.admin_token_verify_enabled
    }
    pub fn user_token_verify_enabled(&self) -> bool {
        self.user_token_verify_enabled
    }
    pub fn admin_ip_bind_enabled(&self) -> bool {
        self.admin_ip_bind_enabled
    }
    pub fn trust_proxy_headers(&self) -> bool {
        self.trust_proxy_headers
    }
    pub fn trusted_proxies(&self) -> &[String] {
        &self.trusted_proxies
    }
}
