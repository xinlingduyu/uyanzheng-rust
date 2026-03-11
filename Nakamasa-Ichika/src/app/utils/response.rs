use serde::Serialize;
use std::borrow::Cow;
use crate::app::plugins::encryption::{EncryptionConfig, Encryption, create_encryption};
use crate::core::md5_optimize::{md5_hex, md5_to_str};

/// 高性能API响应结构 - 使用Cow避免不必要的String分配
#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub code: i32,
    pub msg: Cow<'static, str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T: Serialize> ApiResponse<T> {
    /// 创建成功响应 - 支持&'static str零分配
    #[inline]
    pub fn success(msg: impl Into<Cow<'static, str>>, data: Option<T>) -> Self {
        Self {
            code: 200,
            msg: msg.into(),
            data,
        }
    }

    /// 创建错误响应 - 支持&'static str零分配
    #[inline]
    pub fn error(msg: impl Into<Cow<'static, str>>, code: i32) -> Self {
        Self {
            code,
            msg: msg.into(),
            data: None,
        }
    }
    
    /// 创建成功响应 - 静态字符串零分配
    #[inline]
    pub fn success_static(msg: &'static str, data: Option<T>) -> Self {
        Self::success(msg, data)
    }

    /// 创建错误响应 - 静态字符串零分配
    #[inline]
    pub fn error_static(msg: &'static str, code: i32) -> Self {
        Self::error(msg, code)
    }
}

impl ApiResponse<()> {
    #[inline]
    pub fn success_msg(msg: impl Into<Cow<'static, str>>) -> Self {
        Self {
            code: 200,
            msg: msg.into(),
            data: None,
        }
    }
}

/// 带签名的API响应结构 - 包含sign和time字段
/// 签名算法: md5(code + time + appkey)
#[derive(Serialize)]
pub struct SignedApiResponse<T> {
    pub code: i32,
    pub msg: Cow<'static, str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    pub sign: String,
    pub time: i64,
}

impl<T: Serialize> SignedApiResponse<T> {
    /// 创建带签名的成功响应
    /// code = 0 表示成功
    #[inline]
    pub fn success(app_key: &str, data: Option<T>) -> Self {
        Self::new(0, "成功", app_key, data)
    }

    /// 创建带签名的错误响应
    #[inline]
    pub fn error(msg: impl Into<Cow<'static, str>>, code: i32, app_key: &str) -> Self {
        Self::new(code, msg, app_key, None)
    }

    /// 创建带签名的响应 - 核心方法
    /// 签名算法: md5(code + time + appkey)
    #[inline]
    pub fn new(code: i32, msg: impl Into<Cow<'static, str>>, app_key: &str, data: Option<T>) -> Self {
        let time = chrono::Utc::now().timestamp();
        // 签名算法: md5(code + time + appkey) - 与PHP保持一致
        let sign_data = format!("{}{}{}", code, time, app_key);
        let sign = md5_to_str(&md5_hex(sign_data.as_bytes())).to_string();
        
        Self {
            code,
            msg: msg.into(),
            data,
            sign,
            time,
        }
    }
}

impl SignedApiResponse<()> {
    /// 创建无数据的带签名成功响应
    /// 返回时不会包含data字段
    /// code = 0 表示成功
    #[inline]
    pub fn success_msg(app_key: &str) -> Self {
        Self {
            code: 0,
            msg: "成功".into(),
            data: None,
            sign: Self::calc_sign(0, app_key),
            time: chrono::Utc::now().timestamp(),
        }
    }
    
    /// 创建自定义消息的无数据带签名成功响应
    /// code = 0 表示成功
    #[inline]
    pub fn success_with_msg(msg: impl Into<Cow<'static, str>>, app_key: &str) -> Self {
        let time = chrono::Utc::now().timestamp();
        let sign_data = format!("{}{}{}", 0, time, app_key);
        let sign = md5_to_str(&md5_hex(sign_data.as_bytes())).to_string();
        
        Self {
            code: 0,
            msg: msg.into(),
            data: None,
            sign,
            time,
        }
    }
}

impl<T: Serialize> SignedApiResponse<T> {
    /// 计算签名
    #[inline]
    fn calc_sign(code: i32, app_key: &str) -> String {
        let time = chrono::Utc::now().timestamp();
        let sign_data = format!("{}{}{}", code, time, app_key);
        md5_to_str(&md5_hex(sign_data.as_bytes())).to_string()
    }

    /// 创建带签名、数据和自定义消息的成功响应
    /// code = 0 表示成功
    #[inline]
    pub fn success_msg_data(app_key: &str, data: Option<T>, msg: impl Into<Cow<'static, str>>) -> Self {
        let time = chrono::Utc::now().timestamp();
        let sign_data = format!("{}{}{}", 0, time, app_key);
        let sign = md5_to_str(&md5_hex(sign_data.as_bytes())).to_string();
        
        Self {
            code: 0,
            msg: msg.into(),
            data,
            sign,
            time,
        }
    }
}

// ============================================================================
// 加密响应结构
// ============================================================================

/// 带加密的API响应结构
/// 当应用配置了加密时，data字段会被加密为字符串
/// 参考PHP: Ue/tools/out.php 的加密逻辑
#[derive(Serialize)]
pub struct EncryptedApiResponse {
    pub code: i32,
    pub msg: Cow<'static, str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,  // 加密后的数据字符串
    pub sign: String,
    pub time: i64,
}

impl EncryptedApiResponse {
    /// 创建带加密的成功响应
    /// 
    /// # 参数
    /// - `app_key`: 应用密钥，用于签名
    /// - `data`: 要返回的数据（会被序列化并加密）
    /// - `encryption_config`: 加密配置
    /// - `enc_type`: 加密类型 ("aes", "des", "rc4", "rsa")
    /// 
    /// # 示例
    /// ```ignore
    /// let response = EncryptedApiResponse::success(
    ///     "app_key",
    ///     Some(json!({"uid": 123})),
    ///     &config,
    ///     "aes"
    /// );
    /// ```
    pub fn success<T: Serialize>(
        app_key: &str, 
        data: Option<T>, 
        encryption_config: &EncryptionConfig,
    ) -> Self {
        Self::new(0, "成功", app_key, data, encryption_config)
    }

    /// 创建带加密的错误响应（data不会被加密，因为没有数据）
    pub fn error(msg: impl Into<Cow<'static, str>>, code: i32, app_key: &str) -> Self {
        let time = chrono::Utc::now().timestamp();
        let sign_data = format!("{}{}{}", code, time, app_key);
        let sign = format!("{:x}", md5::compute(sign_data.as_bytes()));
        
        Self {
            code,
            msg: msg.into(),
            data: None,
            sign,
            time,
        }
    }

    /// 创建带加密的响应 - 核心方法
    /// 参考PHP逻辑：
    /// 1. 先将data序列化为JSON
    /// 2. 使用加密器加密
    /// 3. 返回加密后的字符串
    pub fn new<T: Serialize>(
        code: i32, 
        msg: impl Into<Cow<'static, str>>, 
        app_key: &str, 
        data: Option<T>,
        encryption_config: &EncryptionConfig,
    ) -> Self {
        let time = chrono::Utc::now().timestamp();
        let sign_data = format!("{}{}{}", code, time, app_key);
        let sign = format!("{:x}", md5::compute(sign_data.as_bytes()));
        
        // 加密data字段 
        let encrypted_data = if let Some(d) = data {
            match serde_json::to_string(&d) {
                Ok(json_str) => {
                    let encryptor = create_encryption(encryption_config);
                    match encryptor.encode(&json_str) {
                        Ok(encrypted) => Some(encrypted),
                        Err(e) => {
                            tracing::error!("数据加密失败: {}", e);
                            None
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("数据序列化失败: {}", e);
                    None
                }
            }
        } else {
            None
        };
        
        Self {
            code,
            msg: msg.into(),
            data: encrypted_data,
            sign,
            time,
        }
    }
    
    /// 创建无数据的带签名成功响应
    #[inline]
    pub fn success_msg(app_key: &str) -> Self {
        let time = chrono::Utc::now().timestamp();
        let sign_data = format!("{}{}{}", 0, time, app_key);
        let sign = format!("{:x}", md5::compute(sign_data.as_bytes()));
        
        Self {
            code: 0,
            msg: "成功".into(),
            data: None,
            sign,
            time,
        }
    }
    
    /// 创建带自定义消息的无数据成功响应
    #[inline]
    pub fn success_with_msg(msg: impl Into<Cow<'static, str>>, app_key: &str) -> Self {
        let time = chrono::Utc::now().timestamp();
        let sign_data = format!("{}{}{}", 0, time, app_key);
        let sign = format!("{:x}", md5::compute(sign_data.as_bytes()));
        
        Self {
            code: 0,
            msg: msg.into(),
            data: None,
            sign,
            time,
        }
    }
}

// ============================================================================
// 通用响应构建器 - 支持动态加密/不加密
// ============================================================================

/// API响应构建器
/// 支持根据条件自动选择加密或非加密响应
pub struct ApiResponseBuilder<'a> {
    app_key: &'a str,
    encryption_config: Option<&'a EncryptionConfig>,
}

impl<'a> ApiResponseBuilder<'a> {
    /// 创建新的构建器
    #[inline]
    pub fn new(app_key: &'a str) -> Self {
        Self {
            app_key,
            encryption_config: None,
        }
    }
    
    /// 设置加密配置
    #[inline]
    pub fn with_encryption(mut self, config: &'a EncryptionConfig) -> Self {
        self.encryption_config = Some(config);
        self
    }
    
    /// 构建成功响应 - 自动选择加密或非加密
    pub fn build_success<T: Serialize + Clone>(&self, data: Option<T>) -> ResponseType<T> {
        if let Some(config) = self.encryption_config {
            ResponseType::Encrypted(EncryptedApiResponse::success(self.app_key, data, config))
        } else {
            ResponseType::Signed(SignedApiResponse::success(self.app_key, data))
        }
    }
    
    /// 构建错误响应
    pub fn build_error<T: Serialize>(&self, msg: impl Into<Cow<'static, str>>, code: i32) -> ResponseType<T> {
        if let Some(_config) = self.encryption_config {
            ResponseType::Encrypted(EncryptedApiResponse::error(msg, code, self.app_key))
        } else {
            ResponseType::Signed(SignedApiResponse::error(msg, code, self.app_key))
        }
    }
}

/// 响应类型枚举
pub enum ResponseType<T: Serialize> {
    Signed(SignedApiResponse<T>),
    Encrypted(EncryptedApiResponse),
}