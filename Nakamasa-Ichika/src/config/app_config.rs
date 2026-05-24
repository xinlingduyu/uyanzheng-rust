use serde::Deserialize;

#[allow(dead_code)]
#[derive(Debug, Deserialize, Default)]
pub struct AppConfig {
    pub host: String,
    pub code: String,
    pub upload_dir: String,
    pub upload_size: u32,
    /// 每个 token 每天允许上传的最大文件数量，0 表示不限制
    #[serde(default)]
    pub upload_daily_limit: u32,
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

    /// 获取加密密钥（app.code）
    pub fn code(&self) -> &str {
        &self.code
    }

    #[allow(dead_code)]
    pub fn wx_appid(&self) -> &str {
        &self.wx_appid
    }

    pub fn admin(&self) -> &AdminConfig {
        &self.admin
    }
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Default)]
pub struct AdminConfig {
    pub path: String,
    pub keys: String,
    pub token_exp: u64,
    pub token_key: String,
}

impl AdminConfig {
    /// 获取原始 keys（可能加密）
    pub fn keys(&self) -> &str {
        &self.keys
    }
}
