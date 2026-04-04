//! 修改密码
//! 
//! 功能说明：
//! 已登录用户修改登录密码，需要验证当前密码。
//!
//! 处理流程：
//! 1. 验证token、当前密码、新密码参数
//! 2. 验证当前密码是否正确
//! 3. 更新用户password字段（MD5加密）
//! 4. 更新Redis中token关联的密码
//! 5. 返回成功

use salvo::prelude::*;
use std::sync::Arc;
use chrono::Utc;

use crate::core::AppState;
use crate::core::middleware::get_client_ip;
use crate::core::md5_optimize::{md5_hex, md5_to_str};
use crate::app::utils::response::SignedApiResponse;
use crate::app::utils::validator::Validator;
use crate::app::models::requests::ModifyPwdRequest;
use crate::app::middleware::user_auth::UserInfo;
use crate::app::middleware::app_context::AppInfo;

#[handler]
pub async fn modify_pwd(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let app_state = depot.obtain::<Arc<AppState>>().unwrap();
    
    // 获取 app_key 用于签名
    let app_key = match depot.get::<AppInfo>("app_info") {
        Ok(info) => info.app_key.as_str(),
        Err(_) => {
            res.render(Json(SignedApiResponse::<()>::error("应用信息不存在", 201, "")));
            return;
        }
    };

    let modify_req = match req.parse_json::<ModifyPwdRequest>().await {
        Ok(data) => data,
        Err(_) => {
            res.render(Json(SignedApiResponse::<()>::error("参数解析失败", 201, app_key)));
            return;
        }
    };

    // PHP: $checkRules = ['token' => ['wordnum','32,32','TOKEN有误'], 'password' => ['Password','6,18','当前密码有误'], 'newPassword' => ['Password','6,18','新密码长度需要满足6-18位数,不支持中文以及.-*_以外特殊字符']]
    // 验证参数
    let mut validator = Validator::new();
    validator
        .wordnum("token", &modify_req.token, 32, 32)
        .password("password", &modify_req.password, 6, 18)
        .password("new_password", &modify_req.new_password, 6, 18);
    
    if let Err(msg) = validator.validate() {
        res.render(Json(SignedApiResponse::<()>::error(msg, 201, app_key)));
        return;
    }

    // 从 depot 获取用户信息（由 UserAuth 中间件提供）
    let user_info = match depot.get::<UserInfo>("user_info") {
        Ok(info) => info,
        Err(_) => {
            res.render(Json(SignedApiResponse::<()>::error("未授权", 201, app_key)));
            return;
        }
    };

    let (uid, appid) = (user_info.uid, user_info.appid);
    let user_type = user_info.user_type.as_str();
    let current_time = Utc::now().timestamp();
    let ip = get_client_ip(req);
    let redis_util = &app_state.redis_util;

    // 计算密码 hash - 使用优化的 MD5 计算
    let current_hash_bytes = md5_hex(modify_req.password.as_bytes());
    let current_hash = md5_to_str(&current_hash_bytes);
    let new_hash_bytes = md5_hex(modify_req.new_password.as_bytes());
    let new_hash = md5_to_str(&new_hash_bytes);

    // PHP: if($this->user['password'] != md5($_POST['password']))$this->out->e(132);
    // 验证当前密码
    if current_hash != user_info.password {
        res.render(Json(SignedApiResponse::<()>::error("当前密码错误", 132, app_key)));
        return;
    }

    // PHP: if(md5($_POST['newPassword']) == md5($_POST['password']))$this->out->e(133);
    // 验证新旧密码不能相同
    if new_hash == current_hash {
        res.render(Json(SignedApiResponse::<()>::error("新旧密码不能相同", 133, app_key)));
        return;
    }

    // PHP: $res = $this->db->where('id = ? and appid = ?',[$this->user['id'],$this->app['id']])->update(['password'=>md5($_POST['newPassword'])]);
    // 更新密码 - 根据用户类型选择表
    let result = if user_type == "kami" {
        sqlx::query("UPDATE u_cdk_kami SET password = ? WHERE id = ? AND appid = ?")
            .bind(new_hash)
            .bind(uid as i64)
            .bind(appid as i64)
            .execute(app_state.get_db())
            .await
    } else {
        sqlx::query("UPDATE u_user SET password = ? WHERE id = ? AND appid = ?")
            .bind(new_hash)
            .bind(uid as i64)
            .bind(appid as i64)
            .execute(app_state.get_db())
            .await
    };

    match result {
        Ok(r) => {
            if r.rows_affected() > 0 {
                // PHP: $this->log->u($this->app['app_type'],$this->user['id'])->add($res);
                // 记录日志
                let _ = sqlx::query(
                    "INSERT INTO u_logs (ug, uid, type, time, ip, ip_address, appid) VALUES (?, ?, ?, ?, ?, NULL, ?)"
                )
                .bind(user_type)
                .bind(uid as i64)
                .bind("modifyPwd")
                .bind(current_time)
                .bind(ip)
                .bind(appid as i64)
                .execute(app_state.get_db())
                .await;

                // PHP: $this->__delToken($this->user['id']);
                // 删除Redis中该用户的所有token（踢下线）
                if let Some(redis_pool) = app_state.redis_pool.as_ref() {
                    delete_all_user_tokens(redis_util, redis_pool, appid, uid, user_type).await;
                }

                res.render(Json(SignedApiResponse::success(app_key, None::<()>)));
            } else {
                res.render(Json(SignedApiResponse::<()>::error("修改失败", 201, app_key)));
            }
        }
        Err(e) => {
            tracing::error!("修改密码失败: {}", e);
            res.render(Json(SignedApiResponse::<()>::error("修改失败", 201, app_key)));
        }
    }
}

/// 删除用户的所有token（踢下线）- 优化版
/// PHP: __delToken($uid)
async fn delete_all_user_tokens(
    redis_util: &crate::core::redis::RedisUtil,
    redis_pool: &deadpool_redis::Pool,
    appid: u64,
    uid: u64,
    user_type: &str,
) {
    // 查找所有匹配的online key
    // 格式: {user_type}_{appid}_online_{uid}_{udid_hash}
    let pattern = format!("{}_{}_online_{}_*", user_type, appid, uid);
    
    tracing::debug!("清除用户 {} 的所有token, pattern: {}", uid, pattern);
    
    // 使用scan_keys查找所有匹配的键
    match redis_util.scan_keys(redis_pool, &pattern, Some(100)).await {
        Ok(keys) => {
            let key_count = keys.len();
            for key in &keys {
                // 获取token值
                if let Ok(Some(token)) = redis_util.get(redis_pool, key).await {
                    // 删除token键
                    let token_key = format!("{}_{}__{}", user_type, appid, token);
                    let _ = redis_util.del(redis_pool, &token_key).await;
                }
                
                // 删除online key
                let _ = redis_util.del(redis_pool, key).await;
            }
            tracing::debug!("成功清除用户 {} 的 {} 个token", uid, key_count);
        }
        Err(e) => {
            tracing::debug!("查找token失败: {}", e);
        }
    }
}
