use serde::Deserialize;
use Nakamasa_utils::{decrypt_if_needed, is_encrypted};

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

    /// 获取解密后的 JWT token_key
    pub fn token_key(&self) -> String {
        self.admin.decrypted_token_key(&self.code)
    }
    
    /// 获取原始 token_key（可能加密）
    pub fn raw_token_key(&self) -> &str {
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
    
    /// 获取解密后的 admin keys
    pub fn admin_keys(&self) -> String {
        self.admin.decrypted_keys(&self.code)
    }
}

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
    
    /// 获取解密后的 keys
    pub fn decrypted_keys(&self, secret: &str) -> String {
        decrypt_if_needed(&self.keys, secret).unwrap_or_else(|_| self.keys.clone())
    }

    /// 获取原始 token_key（可能加密）
    pub fn token_key(&self) -> &str {
        &self.token_key
    }
    
    /// 获取解密后的 token_key
    pub fn decrypted_token_key(&self, secret: &str) -> String {
        decrypt_if_needed(&self.token_key, secret).unwrap_or_else(|_| self.token_key.clone())
    }
    
    /// 检查 keys 是否已加密
    pub fn is_keys_encrypted(&self) -> bool {
        is_encrypted(&self.keys)
    }
    
    /// 检查 token_key 是否已加密
    pub fn is_token_key_encrypted(&self) -> bool {
        is_encrypted(&self.token_key)
    }
}