//! 心跳接口
//! 
//! 功能说明：
//! 客户端定期调用的保活接口，用于维持登录状态。
//! 成功调用会延长token有效期。
//!
//! 处理流程：
//! 1. 验证token参数（32位字母数字）
//! 2. 检查应用登录开关状态
//! 3. 从Redis获取token数据
//! 4. 延长token过期时间
//! 5. 返回成功状态

use salvo::prelude::*;
use std::sync::Arc;
use std::fmt::Write;
use serde::Deserialize;

use crate::core::AppState;
use crate::core::md5_optimize::{md5_hex, md5_to_str};
use crate::app::utils::response::{SignedApiResponse, render_success, render_success_msg, render_success_with_msg, render_error};
use crate::app::utils::validator::Validator;
use crate::app::middleware::app_context::AppInfo;

/// 心跳请求参数
#[derive(Deserialize)]
struct HeartbeatRequest {
    token: String,
}

/// 心跳接口处理器
/// 
/// PHP原始逻辑:
/// 1. 验证token参数（32位字母数字）
/// 2. 检查logon_state是否为off
/// 3. 从Redis获取token数据
/// 4. 调用__setToken延长token过期时间
/// 5. 返回成功（code=200 → API文档code=1000）
#[handler]
pub async fn heartbeat(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let app_state = match depot.obtain::<Arc<AppState>>() {
        Ok(s) => s,
        Err(_) => {
            render_error(res, "服务器错误", 201, "");
            return;
        }
    };
    
    // 获取应用信息（避免 clone，直接使用引用）
    let app_info = match depot.get::<AppInfo>("app_info") {
        Ok(info) => info,
        Err(_) => {
            render_error(res, "应用信息不存在", 201, "");
            return;
        }
    };

    let appid = app_info.id;
    let app_type = &app_info.app_type;
    let app_key = &app_info.app_key;

    // 解析JSON请求体
    let heartbeat_req = match req.parse_json::<HeartbeatRequest>().await {
        Ok(data) => data,
        Err(_) => {
            render_error(res, "参数解析失败", 201, app_key);
            return;
        }
    };

    // PHP: 'token' => ['wordnum','32,32','TOKEN有误']
    // 验证token参数：32位字母数字
    {
        let mut validator = Validator::new();
        validator.wordnum("TOKEN", &heartbeat_req.token, 32, 32);
        if validator.validate().is_err() {
            render_error(res, "TOKEN有误", 201, app_key);
            return;
        }
    }
    let token = heartbeat_req.token;

    // PHP: if($this->app['logon_state'] == 'off')$this->out->e(103,$this->app['logon_off_msg']);
    // 检查登录状态
    if app_info.logon_state == "off" {
        let msg = app_info.logon_off_msg.clone().unwrap_or_else(|| "登录功能已关闭".to_string());
        render_error(res, msg, 103, app_key);
        return;
    }

    let redis_util = &app_state.redis_util;
    let redis_pool = match app_state.redis_pool.as_ref() {
        Some(pool) => pool,
        None => {
            render_error(res, "Redis未初始化", 201, app_key);
            return;
        }
    };

    // PHP: $this->tokenPre = $this->appConfig['USER_TOKEN_PRE'].$this->app['app_type'].'_'.$this->app['id'].'_';
    // token前缀格式: {app_type}_{appid}_（预分配容量）
    let mut token_pre = String::with_capacity(16);
    let _ = write!(&mut token_pre, "{}_{}_", app_type, appid);
    
    // token_key（预分配容量）
    let mut token_key = String::with_capacity(48);
    let _ = write!(&mut token_key, "{}{}", token_pre, token);

    // PHP: $this->redis->select(1);
    // PHP: $token = $this->redis->get($this->tokenPre.$_POST['token']);
    // PHP: if(!$token)$this->out->e(128);
    let token_str = match redis_util.get(redis_pool, &token_key).await {
        Ok(Some(data)) => data,
        Ok(None) => {
            #[cfg(debug_assertions)]
            {
                tracing::warn!("[心跳] Token不存在: token_key={}", token_key);
                // 调试：尝试模糊匹配查找token
                let pattern = format!("{}*", token_pre);
                if let Ok(keys) = redis_util.keys(redis_pool, &pattern).await {
                    tracing::warn!("[心跳] 找到 {} 个匹配前缀的key", keys.len());
                }
            }
            render_error(res, "Token不存在", 128, app_key);
            return;
        }
        Err(e) => {
            tracing::error!("[心跳] Redis查询失败: {}", e);
            render_error(res, "服务器错误", 201, app_key);
            return;
        }
    };

    // PHP: $res = $this->__setToken($_POST['token'],json_decode($token,true));
    // 延长token过期时间
    let token_exp = app_info.logon_token_exp as u64;
    let set_result = set_token(
        redis_util, 
        redis_pool, 
        &token_pre, 
        &token, 
        &token_str, 
        token_exp
    ).await;

    if !set_result {
        // PHP: if(!$res)$this->out->e(201,'心跳失败，token记录失败');
        render_error(res, "心跳失败，token记录失败", 201, app_key);
        return;
    }

    // PHP: $this->out->e(200);
    // API文档要求返回code=1000表示成功，无data字段
    render_success_msg(res, app_key);
}

/// 保存token - 一比一还原PHP的__setToken方法
/// 
/// PHP原始代码:
/// ```php
/// protected function __setToken($token,$data){
///     $this->redis->select(1);
///     $res = $this->redis->setex($this->tokenPre.$token,$this->app['logon_token_exp'],json_encode($data));
///     $this->redis->setex($this->tokenPre."online_{$data['uid']}_".md5($data['udid']),$this->app['logon_token_exp'],$token);
///     return $res;
/// }
/// ```
async fn set_token(
    redis_util: &crate::core::redis::RedisUtil,
    redis_pool: &deadpool_redis::Pool,
    token_pre: &str,
    token: &str,
    token_data_str: &str,
    token_exp: u64,
) -> bool {
    // 解析token数据获取uid和udid
    let token_data: serde_json::Value = match serde_json::from_str(token_data_str) {
        Ok(data) => data,
        Err(_) => return false,
    };

    let uid = match token_data.get("uid").and_then(|v| v.as_i64()) {
        Some(id) => id,
        None => return false,
    };

    let udid = match token_data.get("udid").and_then(|v| v.as_str()) {
        Some(u) => u,
        None => return false,
    };

    // PHP: $res = $this->redis->setex($this->tokenPre.$token,$this->app['logon_token_exp'],json_encode($data));
    let mut token_key = String::with_capacity(48);
    let _ = write!(&mut token_key, "{}{}", token_pre, token);
    
    if redis_util.set(redis_pool, &token_key, token_data_str, Some(token_exp)).await.is_err() {
        return false;
    }

    // PHP: $this->redis->setex($this->tokenPre."online_{$data['uid']}_".md5($data['udid']),$this->app['logon_token_exp'],$token);
    // 使用优化的MD5计算
    let udid_hash_bytes = md5_hex(udid.as_bytes());
    let udid_hash = md5_to_str(&udid_hash_bytes);
    
    let mut online_key = String::with_capacity(64);
    let _ = write!(&mut online_key, "{}online_{}_{}", token_pre, uid, udid_hash);
    
    if redis_util.set(redis_pool, &online_key, token, Some(token_exp)).await.is_err() {
        return false;
    }

    true
}
