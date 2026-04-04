//! 管理员认证中间件
//! 参考PHP base/admin.php 的 tokenVerify 方法

use salvo::prelude::*;
use crate::core::AppState;
use crate::core::md5_optimize::{md5_hex, md5_to_str};
use crate::app::utils::response::ApiResponse;
use std::sync::Arc;
use Nakamasa_utils::jwt::JwtBuilder;
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

// 预分配错误消息
static ERR_TOKEN_EMPTY: &str = "Token不能为空";
static ERR_TOKEN_VERIFY_FAIL: &str = "Token验证失败";
static ERR_TOKEN_INVALID: &str = "Token失效";
static ERR_TOKEN_EXPIRED: &str = "Token已过期或不存在";
static ERR_DB_ERROR: &str = "数据库错误";

/// 快速获取当前时间戳
#[inline]
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// 常量时间比较 - 防止时序攻击
/// 无论字符串是否匹配，都比较所有字符，避免通过响应时间推断信息
#[inline]
fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    let mut result: u8 = 0;
    
    for i in 0..a.len() {
        result |= a_bytes[i] ^ b_bytes[i];
    }
    
    result == 0
}

/// 管理员信息（从Token中解析）
#[derive(Debug, Clone, Serialize)]
pub struct AdminInfo {
    pub id: u64,
    pub user: String,
    pub password: String,
    pub notes: Option<String>,
    pub avatars: String,
    pub lockin: bool,
    pub auth: Option<serde_json::Value>,
    pub state: String,
    pub appid: Option<u64>,
}

/// Token验证结果
#[derive(Debug, Clone, Serialize)]
pub struct TokenVerifyResult {
    pub info: AdminInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<TokenRenew>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TokenRenew {
    pub new: String,
    pub exp: i64,
}

/// 管理员认证中间件
pub struct AdminAuth {
    pub skip_token_verify: bool,
}

impl AdminAuth {
    pub fn new() -> Self {
        Self {
            skip_token_verify: false,
        }
    }

    pub fn skip_verify(mut self) -> Self {
        self.skip_token_verify = true;
        self
    }
}

impl Default for AdminAuth {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Handler for AdminAuth {
    async fn handle(&self, req: &mut Request, depot: &mut Depot, res: &mut Response, ctrl: &mut FlowCtrl) {
        let app_state = depot.obtain::<Arc<AppState>>().unwrap();
        let app_conf = app_state.config();

        // 获取Token - 支持 "Token" 和 "HTTP_TOKEN" 两种 header
        let token = req.headers().get("Token")
            .or_else(|| req.headers().get("HTTP_TOKEN"));
        
        let token_str = match token.and_then(|t| t.to_str().ok()).filter(|s| !s.is_empty()) {
            Some(s) => s,
            None => {
                res.render(Json(ApiResponse::<()>::error(ERR_TOKEN_EMPTY, 201)));
                ctrl.skip_rest();
                return;
            }
        };
        
        // 获取客户端IP
        let ip = get_client_ip(req);
        let ip_str: &str = &ip;

        // 验证Token
        let jwt_key = app_conf.app().admin().keys();
        let jwt_builder = JwtBuilder::new(jwt_key, 3);
        
        let claims = match jwt_builder.verify(token_str) {
            Ok(c) => c,
            Err(_) => {
                res.render(Json(ApiResponse::<()>::error(ERR_TOKEN_VERIFY_FAIL, -1)));
                ctrl.skip_rest();
                return;
            }
        };

        // 验证Claims - 使用短路求值
        let id = match claims.custom.get("id").and_then(|v| v.as_u64()) {
            Some(id) => id,
            None => {
                res.render(Json(ApiResponse::<()>::error(ERR_TOKEN_INVALID, -1)));
                ctrl.skip_rest();
                return;
            }
        };

        let pwd = match claims.custom.get("pwd").and_then(|v| v.as_str()) {
            Some(p) => p,
            None => {
                res.render(Json(ApiResponse::<()>::error(ERR_TOKEN_INVALID, -1)));
                ctrl.skip_rest();
                return;
            }
        };

        let claim_ip = match claims.custom.get("ip").and_then(|v| v.as_str()) {
            Some(ip) => ip,
            None => {
                res.render(Json(ApiResponse::<()>::error(ERR_TOKEN_INVALID, -1)));
                ctrl.skip_rest();
                return;
            }
        };

        // IP验证
        if claim_ip != ip_str {
            res.render(Json(ApiResponse::<()>::error(ERR_TOKEN_INVALID, -1)));
            ctrl.skip_rest();
            return;
        }

        // 查询管理员信息
        let admin_result = sqlx::query_as::<_, (u64, String, String, Option<String>, String, Option<String>, Option<String>, bool, Option<u64>)>(
            "SELECT id, user, password, notes, state, avatars, auth, lockin, appid FROM u_admin WHERE id = ? AND state = ?"
        )
        .bind(id)
        .bind("y")
        .fetch_optional(app_state.get_db())
        .await;

        let admin = match admin_result {
            Ok(Some(a)) => a,
            Ok(None) => {
                res.render(Json(ApiResponse::<()>::error(ERR_TOKEN_EXPIRED, -1)));
                ctrl.skip_rest();
                return;
            }
            Err(_) => {
                res.render(Json(ApiResponse::<()>::error(ERR_DB_ERROR, 201)));
                ctrl.skip_rest();
                return;
            }
        };

        // 验证密码 - 栈上MD5计算 + 常量时间比较防止时序攻击
        let password_hash_bytes = md5_hex(admin.2.as_bytes());
        let password_hash_str = md5_to_str(&password_hash_bytes);
        if !constant_time_eq(password_hash_str, pwd) {
            res.render(Json(ApiResponse::<()>::error(ERR_TOKEN_EXPIRED, -1)));
            ctrl.skip_rest();
            return;
        }

        // 构建管理员信息
        let auth = admin.6.as_ref().and_then(|v| serde_json::from_str(v).ok());
        
        // 存储到Depot供后续使用 - 在move之前
        depot.insert("admin_id", admin.0);
        depot.insert("admin_user", admin.1.clone());
        
        let admin_info = AdminInfo {
            id: admin.0,
            user: admin.1,
            password: admin.2,
            notes: admin.3,
            avatars: admin.5.unwrap_or_default(),
            lockin: admin.7,
            auth,
            state: admin.4,
            appid: admin.8,
        };

        depot.insert("admin_info", admin_info.clone());

        // 如果是tokenVerify接口，返回验证结果
        if self.skip_token_verify {
            let mut result = TokenVerifyResult {
                info: admin_info,
                token: None,
            };

            // 检查是否需要续期（剩余时间小于24小时）
            let exp = claims.exp;
            let now = current_timestamp();
            if (exp - now) < 86400
                && let Ok(new_token) = jwt_builder
                    .set_iss("admin")
                    .add_claim("id", admin.0)
                    .add_claim("ip", ip_str)
                    .add_claim("pwd", password_hash_str)
                    .build()
                {
                    result.token = Some(TokenRenew {
                        new: new_token,
                        exp: exp as i64,
                    });
                }

            res.render(Json(ApiResponse::success("成功", Some(result))));
            ctrl.skip_rest();
            return;
        }

        // 继续执行下一个处理器
        ctrl.call_next(req, depot, res).await;
    }
}

/// 获取客户端IP - 返回 String 避免内存泄漏
/// 注意：在生产环境中，应配置可信代理列表来验证 X-Real-IP 和 X-Forwarded-For
#[inline]
fn get_client_ip(req: &Request) -> String {
    // 尝试从 X-Real-IP 获取
    if let Some(x_real_ip) = req.headers().get("X-Real-IP")
        && let Ok(ip) = x_real_ip.to_str()
            && is_valid_ip(ip) {
                return ip.to_string();
            }
    
    // 尝试从 X-Forwarded-For 获取
    if let Some(x_forwarded_for) = req.headers().get("X-Forwarded-For")
        && let Ok(ip_list) = x_forwarded_for.to_str()
            && let Some(ip) = ip_list.split(',').next() {
                let ip = ip.trim();
                if is_valid_ip(ip) {
                    return ip.to_string();
                }
            }

    // 默认返回本地IP
    "127.0.0.1".to_string()
}

/// 验证 IP 地址格式
#[inline]
fn is_valid_ip(ip: &str) -> bool {
    // 简单验证：检查是否只包含有效的 IP 字符
    if ip.is_empty() || ip.len() > 45 { // IPv6 最大长度
        return false;
    }
    
    // 检查字符是否有效
    let mut dot_count = 0;
    let mut colon_count = 0;
    
    for c in ip.chars() {
        match c {
            '0'..='9' => {}
            'a'..='f' | 'A'..='F' => {} // IPv6 十六进制
            '.' => dot_count += 1,
            ':' => colon_count += 1,
            _ => return false,
        }
    }
    
    // IPv4 最多 3 个点，IPv6 最多 7 个冒号
    dot_count <= 3 || colon_count <= 7
}