//! 获取个人信息
//! 一比一还原PHP: controller/user/info.php
use salvo::prelude::*;
use std::sync::Arc;

use crate::core::AppState;
use crate::app::utils::response::SignedApiResponse;
use crate::app::middleware::user_auth::UserInfo;
use crate::app::middleware::app_context::AppInfo;

#[handler]
pub async fn get_info(_req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let app_state = depot.obtain::<Arc<AppState>>().unwrap();
    
    // 从 depot 获取用户信息（避免 clone，直接使用引用）
    // PHP: $this->user
    let user_info = match depot.get::<UserInfo>("user_info") {
        Ok(info) => info,
        Err(_) => {
            let app_info = depot.get::<AppInfo>("app_info").unwrap();
            res.render(Json(SignedApiResponse::<()>::error("未授权", 201, &app_info.app_key)));
            return;
        }
    };

    // 获取应用信息（避免 clone）
    // PHP: $this->app
    let app_info = match depot.get::<AppInfo>("app_info") {
        Ok(info) => info,
        Err(_) => {
            res.render(Json(SignedApiResponse::<()>::error("应用信息不存在", 201, "")));
            return;
        }
    };

    let app_key = &app_info.app_key;
    let app_url = app_state.config().app().host();

    // PHP: $method = '__'.$this->app['app_type'];
    // PHP: $this->$method();
    if user_info.user_type == "user" {
        // PHP: __user() 方法
        let info = build_user_info(user_info, app_info, app_url);
        res.render(Json(SignedApiResponse::success(app_key, Some(info))));
    } else if user_info.user_type == "kami" {
        // PHP: __kami() 方法
        let info = build_kami_info(user_info, app_info);
        res.render(Json(SignedApiResponse::success(app_key, Some(info))));
    } else {
        res.render(Json(SignedApiResponse::<()>::error("用户类型错误", 201, app_key)));
    }
}

/// 构建普通用户信息 - 一比一还原PHP __user方法
#[inline]
fn build_user_info(user: &UserInfo, app: &AppInfo, app_url: &str) -> serde_json::Value {
    // PHP: 'pic'=>empty($this->user['avatars'])?'':getUrl().$this->user['avatars']
    let pic = user.avatars.as_deref()
        .filter(|a| !a.is_empty())
        .map(|a| format!("{}{}", app_url, a))
        .unwrap_or_default();

    // PHP: 'vipExpDate'=>date("Y-m-d H:i:s",$this->user['vip'])
    let vip_exp_date = user.vip.and_then(|v| {
        chrono::DateTime::from_timestamp(v, 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
    });

    // PHP: 'extend'=>empty($this->user['extend'])?null:json_decode($this->user['extend'],true)
    let extend: Option<serde_json::Value> = user.extend.as_deref()
        .filter(|e| !e.is_empty())
        .and_then(|e| serde_json::from_str(e).ok());

    // PHP: 'snMax'=>intval($this->user['sn_max'])+intval($this->app['logon_sn_num'])
    let sn_max = user.sn_max + app.logon_sn_num;

    serde_json::json!({
        "uid": user.uid,
        "phone": user.phone,
        "email": user.email,
        "acctno": user.acctno,
        "name": user.nickname,
        "pic": pic,
        "invID": user.inviter_id.unwrap_or(0),
        "fen": user.fen,
        "vipExpTime": user.vip.unwrap_or(0),
        "vipExpDate": vip_exp_date,
        "extend": extend,
        "snMax": sn_max,
        "agent": user.agent
    })
}

/// 构建卡密用户信息 - 一比一还原PHP __kami方法
#[inline]
fn build_kami_info(user: &UserInfo, app: &AppInfo) -> serde_json::Value {
    // PHP: 'snMax'=>intval($this->user['sn_max'])+intval($this->app['logon_sn_num'])
    let sn_max = user.sn_max + app.logon_sn_num;

    // 基础信息
    let mut info = serde_json::json!({
        "uid": user.uid,
        "phone": user.phone,
        "email": user.email,
        "cardNo": user.card_no,
        "snMax": sn_max,
        "agent": user.agent
    });

    // PHP: if($this->user['type'] == 'vip')
    if user.kami_type.as_deref() == Some("vip") {
        // PHP: $info['vipExpTime'] = $this->user['vip_exp'];
        info["vipExpTime"] = serde_json::Value::Number(user.vip_exp.unwrap_or(0).into());
        
        // PHP: $info['vipExpDate'] = date("Y-m-d H:i:s",$this->user['vip_exp']);
        let vip_exp_date = user.vip_exp
            .and_then(|v| chrono::DateTime::from_timestamp(v, 0))
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_default();
        info["vipExpDate"] = serde_json::Value::String(vip_exp_date);
    } else {
        // PHP: $info['fen'] = $this->user['val'];
        info["fen"] = serde_json::Value::Number(user.val.unwrap_or(0).into());
    }

    info
}