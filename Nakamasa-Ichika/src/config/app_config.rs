use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub host: String,
    pub code: String,
    pub upload_dir: String,
    pub upload_size: u32,
    pub cache: bool,
    pub user_api_rewrite: bool,
    pub output_msg: bool,
    pub ver: String,
    pub wx_appid: String,
    pub wx_secret: String,
    pub qq_appid: String,
    pub qq_appkey: String,
    pub admin: AdminConfig,
}

impl AppConfig {
    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn token_key(&self) -> &str {
        &self.admin.token_key
    }

    pub fn wx_appid(&self) -> &str {
        &self.wx_appid
    }

    pub fn wx_secret(&self) -> &str {
        &self.wx_secret
    }

    pub fn qq_appid(&self) -> &str {
        &self.qq_appid
    }

    pub fn qq_appkey(&self) -> &str {
        &self.qq_appkey
    }

    pub fn admin(&self) -> &AdminConfig {
        &self.admin
    }
}

#[derive(Debug, Deserialize)]
pub struct AdminConfig {
    pub path: String,
    pub keys: String,
    pub token_exp: u64,
    pub token_key: String,
}

impl AdminConfig {
    pub fn keys(&self) -> &str {
        &self.keys
    }

    pub fn token_key(&self) -> &str {
        &self.token_key
    }
}