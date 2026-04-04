//! 微信扫码登录查询
//! 
//! 功能说明：
//! 轮询检查微信扫码登录状态，返回登录结果。
//!
//! 处理流程：
//! 1. 验证uuid参数（state标识）
//! 2. 从Redis获取登录信息
//! 3. 检查登录状态（是否已扫码完成）
//! 4. 如已登录，生成token并返回用户信息
//! 5. 如未登录，返回待扫码状态

use salvo::prelude::*;
use std::sync::Arc;
use chrono::Utc;
use rand::Rng;
use sqlx::Row;

use crate::core::AppState;
use crate::core::md5_optimize::{md5_hex, md5_to_str};
use crate::app::utils::response::SignedApiResponse;
use crate::app::utils::validator::Validator;
use crate::app::models::requests::WxQueryRequest;
use crate::app::middleware::app_context::AppInfo;
use crate::app::middleware::user_auth::{set_token, TokenData};
use serde_json::json;
use serde::{Serialize, Deserialize};

/// 微信登录信息 - 存储在Redis中
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WxLogonInfo {
    appid: u64,
    udid: String,
    ip: String,
    invid: Option<i64>,
    wx_config: serde_json::Value,
    create_time: i64,
}

#[handler]
pub async fn wx_logon_query(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let app_state = depot.obtain::<Arc<AppState>>().unwrap();
    
    // 获取 app_key
    let app_key = depot.get::<AppInfo>("app_info").map(|i| i.app_key.as_str()).unwrap_or("");
    
    let query_req = match req.parse_json::<WxQueryRequest>().await {
        Ok(data) => data,
        Err(_) => {
            res.render(Json(SignedApiResponse::<()>::error("参数解析失败", 201, app_key)));
            return;
        }
    };

    // PHP: $checkRules = ['uuid' => ['wordnum','32,32','二维码参数有误']];
    let mut validator = Validator::new();
    validator.wordnum("uuid", &query_req.uuid, 32, 32);
    
    if let Err(msg) = validator.validate() {
        res.render(Json(SignedApiResponse::<()>::error(msg, 201, app_key)));
        return;
    }

    // 获取应用信息
    let app_info = match depot.get::<AppInfo>("app_info") {
        Ok(info) => info.clone(),
        Err(_) => {
            res.render(Json(SignedApiResponse::<()>::error("应用信息不存在", 201, app_key)));
            return;
        }
    };

    let redis_util = &app_state.redis_util;
    let redis_pool = match app_state.redis_pool.as_ref() {
        Some(pool) => pool,
        None => {
            res.render(Json(SignedApiResponse::<()>::error("Redis未初始化", 201, app_key)));
            return;
        }
    };

    // PHP: $wxlogon_info = $this->redis->get('wxlogon_info_'.$_POST['uuid']);
    let info_key = format!("wxlogon_info_{}", query_req.uuid);
    let info_str = match redis_util.get(redis_pool, &info_key).await {
        Ok(Some(s)) => s,
        Ok(None) => {
            res.render(Json(SignedApiResponse::<()>::error("二维码参数已过期", 201, app_key)));
            return;
        }
        Err(e) => {
            tracing::error!("Redis查询失败: {}", e);
            res.render(Json(SignedApiResponse::<()>::error("Redis错误", 201, app_key)));
            return;
        }
    };

    // PHP: $wxlogonInfo = json_decode($wxlogon_info,true);
    let wxlogon_info: WxLogonInfo = match serde_json::from_str(&info_str) {
        Ok(info) => info,
        Err(_) => {
            res.render(Json(SignedApiResponse::<()>::error("二维码参数有误", 201, app_key)));
            return;
        }
    };

    // PHP: $uid = $this->redis->get('logon_'.$_POST['uuid']);
    let logon_key = format!("logon_{}", query_req.uuid);
    let uid_str = redis_util.get(redis_pool, &logon_key).await;
    
    // PHP: if(!$uid)$this->out->e(0,'待扫码');
    let uid = match uid_str {
        Ok(Some(s)) => match s.parse::<u64>() {
            Ok(id) => id,
            Err(_) => {
                res.render(Json(SignedApiResponse::<()>::error("待扫码", 0, app_key)));
                return;
            }
        },
        Ok(None) => {
            res.render(Json(SignedApiResponse::<()>::error("待扫码", 0, app_key)));
            return;
        }
        Err(_) => {
            res.render(Json(SignedApiResponse::<()>::error("待扫码", 0, app_key)));
            return;
        }
    };

    // 查询用户信息，包含邀请码
    let user_result = sqlx::query(
        r#"
        SELECT U.id, U.phone, U.email, U.acctno, U.nickname, U.avatars, U.inviter_id, U.fen, U.vip, 
               U.extend, U.ban, U.ban_msg, U.password, U.sn_list, U.sn_max,
               A.inv_code
        FROM u_user AS U 
        LEFT JOIN u_agent AS A ON U.id = A.uid 
        WHERE U.id = ?
        "#
    )
    .bind(uid)
    .fetch_optional(app_state.get_db())
    .await;

    let user_row = match user_result {
        Ok(Some(row)) => row,
        Ok(None) => {
            res.render(Json(SignedApiResponse::<()>::error("登录信息不存在", 201, app_key)));
            return;
        }
        Err(e) => {
            tracing::error!("数据库查询失败: {}", e);
            res.render(Json(SignedApiResponse::<()>::error("数据库错误", 201, app_key)));
            return;
        }
    };

    // 提取用户数据
    let user_id: u64 = user_row.try_get(0).unwrap_or(0);
    let phone: Option<i64> = user_row.try_get(1).ok();
    let email: Option<String> = user_row.try_get(2).ok();
    let acctno: Option<String> = user_row.try_get(3).ok();
    let nickname: Option<String> = user_row.try_get(4).ok();
    let avatars: Option<String> = user_row.try_get(5).ok();
    let inviter_id: Option<i64> = user_row.try_get(6).ok();
    let fen: i64 = user_row.try_get(7).unwrap_or(0);
    let vip: Option<i64> = user_row.try_get(8).ok();
    let extend: Option<String> = user_row.try_get(9).ok();
    let ban: Option<i64> = user_row.try_get(10).ok();
    let ban_msg: Option<String> = user_row.try_get(11).ok();
    let password: String = user_row.try_get(12).unwrap_or_default();
    let sn_list: Option<String> = user_row.try_get(13).ok();
    let sn_max: i32 = user_row.try_get(14).unwrap_or(0);
    let inv_code: Option<String> = user_row.try_get(15).ok();

    // PHP: if($Ures['ban'] > time())$this->out->e(127,$Ures['ban_msg']);
    if let Some(ban_time) = ban
        && ban_time > Utc::now().timestamp() {
            let msg = ban_msg.unwrap_or_else(|| "账号已被禁用".to_string());
            res.render(Json(SignedApiResponse::<()>::error(msg, 127, app_key)));
            return;
        }

    let current_time = Utc::now().timestamp();
    let mut token_state = "y".to_string();

    // PHP: 设备绑定逻辑
    let sn_list_val = sn_list.clone();
    if sn_list_val.is_none() || sn_list_val.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
        // PHP: $sn_list = json_encode([['udid'=>$wxlogonInfo['udid'],'time'=>time()]]);
        let new_sn_list = json!([{"udid": &wxlogon_info.udid, "time": current_time}]).to_string();
        let _ = sqlx::query("UPDATE u_user SET sn_list = ? WHERE id = ?")
            .bind(&new_sn_list)
            .bind(user_id)
            .execute(app_state.get_db())
            .await;
    } else {
        // PHP: $client_Arr = json_decode($Ures['sn_list'],true);
        let client_arr: Vec<serde_json::Value> = serde_json::from_str(sn_list_val.as_ref().unwrap()).unwrap_or_default();
        
        // PHP: $found_key = array_search($wxlogonInfo['udid'],array_column($client_Arr,'udid'));
        let found = client_arr.iter().any(|item| {
            item.get("udid").and_then(|v| v.as_str()) == Some(&wxlogon_info.udid)
        });

        if !found {
            // PHP: 新设备登录
            if app_info.logon_sn_num > 0 {
                if client_arr.len() >= (app_info.logon_sn_num + sn_max) as usize {
                    token_state = "n".to_string();
                } else {
                    let mut new_arr = client_arr.clone();
                    new_arr.push(json!({"udid": &wxlogon_info.udid, "time": current_time}));
                    let sn_list_json = serde_json::to_string(&new_arr).unwrap_or_default();
                    let _ = sqlx::query("UPDATE u_user SET sn_list = ? WHERE id = ?")
                        .bind(&sn_list_json)
                        .bind(user_id)
                        .execute(app_state.get_db())
                        .await;
                }
            } else {
                // PHP: 不限制设备数，绑定新设备并踢掉其他设备
                let new_sn_list = json!([{"udid": &wxlogon_info.udid, "time": current_time}]).to_string();
                let _ = sqlx::query("UPDATE u_user SET sn_list = ? WHERE id = ?")
                    .bind(&new_sn_list)
                    .bind(user_id)
                    .execute(app_state.get_db())
                    .await;
                // TODO: 删除该用户所有token
            }
        } else {
            // PHP: 已绑定设备登录，检查是否允许多开
            if app_info.logon_sn_dk != "y" {
                let udid_md5 = md5_to_str(&md5_hex(wxlogon_info.udid.as_bytes())).to_string();
                let dk_key = format!("logon_{}_{}_{}", app_info.id, user_id, udid_md5);
                if redis_util.exists(redis_pool, &dk_key).await.unwrap_or(false) {
                    res.render(Json(SignedApiResponse::<()>::error("已经登录了", 201, app_key)));
                    return;
                }
            }
        }
    }

    // PHP: $token = md5(uniqid().$Ures['id'].$wxlogonInfo['udid']);
    let random_num: u64 = rand::thread_rng().r#gen();
    let token = {
        let mut token_data = String::with_capacity(64);
        use std::fmt::Write;
        let _ = write!(&mut token_data, "{}{}{}", Utc::now().timestamp_millis(), user_id, &wxlogon_info.udid);
        md5_to_str(&md5_hex(token_data.as_bytes())).to_string()
    };

    // 构建用户信息
    let vip_exp_time = vip.unwrap_or(0);
    let vip_exp_date = if vip_exp_time > 0 {
        let dt = chrono::DateTime::<chrono::Utc>::from_timestamp(vip_exp_time, 0);
        dt.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string()).unwrap_or_default()
    } else {
        "未开通".to_string()
    };

    let extend_val: Option<serde_json::Value> = extend.and_then(|s| serde_json::from_str(&s).ok());
    let app_url = app_state.config().app().host().to_string();
    let pic_url = avatars.map(|a| if a.starts_with("http") { a } else { format!("{}{}", app_url, a) }).unwrap_or_default();

    // PHP: $this->__setToken($token,['uid'=>$Ures['id'],'udid'=>$wxlogonInfo['udid'],'p'=>$Ures['password'],'appid'=>$this->app['id']]);
    let token_data = TokenData {
        uid: user_id,
        udid: wxlogon_info.udid.clone(),
        appid: app_info.id,
        user_type: None,
        p: password.clone(),
    };

    let token_pre = format!("{}_{}_", app_info.app_type, app_info.id);
    if let Err(e) = set_token(redis_util, redis_pool, &token_pre, &token, &token_data, app_info.logon_token_exp).await {
        tracing::error!("设置Token失败: {}", e);
        res.render(Json(SignedApiResponse::<()>::error("登录失败，token记录失败", 201, app_key)));
        return;
    }

    // PHP: 返回登录信息 - 按API文档格式
    res.render(Json(SignedApiResponse::success(app_key, Some(json!({
        "token": token,
        "state": token_state,
        "info": {
            "uid": user_id,
            "phone": phone,
            "email": email,
            "acctno": acctno,
            "name": nickname,
            "pic": pic_url,
            "invID": inviter_id.unwrap_or(0),
            "invCode": inv_code,
            "fen": fen,
            "vipExpTime": vip_exp_time,
            "vipExpDate": vip_exp_date,
            "extend": extend_val
        }
    })))));
}