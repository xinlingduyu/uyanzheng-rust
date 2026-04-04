//! 微信扫码登录
//! 
//! 功能说明：
//! 获取微信开放平台扫码登录URL，用于PC端网页扫码登录。
//!
//! 处理流程：
//! 1. 验证udid参数（设备标识）
//! 2. 获取应用的微信登录配置
//! 3. 生成state标识并存储登录信息到Redis
//! 4. 构建微信授权登录URL
//! 5. 返回登录URL和state供前端生成二维码

use salvo::prelude::*;
use std::sync::Arc;
use chrono::Utc;
use urlencoding::encode;
use rand::Rng;

use crate::core::AppState;
use crate::core::md5_optimize::{md5_hex, md5_to_str};
use crate::app::utils::response::SignedApiResponse;
use crate::app::utils::validator::Validator;
use crate::app::models::requests::WxLogonRequest;
use crate::app::middleware::app_context::AppInfo;
use serde_json::json;
use serde::{Serialize, Deserialize};

/// 微信登录信息 - 存储在Redis中
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WxLogonInfo {
    appid: u64,
    udid: String,
    ip: String,
    invid: Option<i64>,
    wx_config: serde_json::Value, // 存储微信配置
    create_time: i64,
}

#[handler]
pub async fn wx_logon(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let app_state = depot.obtain::<Arc<AppState>>().unwrap();
    
    // 获取 app_key
    let app_key = depot.get::<AppInfo>("app_info").map(|i| i.app_key.as_str()).unwrap_or("");
    
    let wx_req = match req.parse_json::<WxLogonRequest>().await {
        Ok(data) => data,
        Err(_) => {
            res.render(Json(SignedApiResponse::<()>::error("参数解析失败", 201, app_key)));
            return;
        }
    };

    // PHP: $checkRules = ['invid' => ['int','1,11','邀请人ID填写有误',true], 'udid' => ['reg','[a-zA-Z0-9_-]+','机器码有误']];
    let mut validator = Validator::new();
    validator.reg("udid", &wx_req.udid, "[a-zA-Z0-9_-]+");
    // invid 是可选的
    
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

    // PHP: if($this->app['app_type'] != 'user')$this->out->e(115);
    if app_info.app_type != "user" {
        res.render(Json(SignedApiResponse::<()>::error("当前应用不支持调用该接口", 115, app_key)));
        return;
    }

    // PHP: if(empty($this->app['logon_wxopen_config']))$this->out->e(201,'微信登录未配置');
    let wx_config_str = match &app_info.logon_open_wxconfig {
        Some(config) => config,
        None => {
            res.render(Json(SignedApiResponse::<()>::error("微信登录未配置", 201, app_key)));
            return;
        }
    };

    // PHP: $wxConfig = json_decode($this->app['logon_wxopen_config'],true);
    let wx_config: serde_json::Value = match serde_json::from_str(wx_config_str) {
        Ok(json) => json,
        Err(_) => {
            res.render(Json(SignedApiResponse::<()>::error("微信登录配置有误", 201, app_key)));
            return;
        }
    };

    // PHP: if(!$wxConfig || !isset($wxConfig['appID']) || !isset($wxConfig['state']) || !isset($wxConfig['appSecret']))$this->out->e(201,'微信登录配置有误');
    let app_id = wx_config.get("appID").and_then(|v| v.as_str()).unwrap_or("");
    let app_secret = wx_config.get("appSecret").and_then(|v| v.as_str()).unwrap_or("");
    let state_config = wx_config.get("state").and_then(|v| v.as_str()).unwrap_or("");

    // PHP: if($wxConfig['state'] != 'on')$this->out->e(201,'微信登录未开启');
    if state_config != "on" {
        res.render(Json(SignedApiResponse::<()>::error("微信登录未开启", 201, app_key)));
        return;
    }

    // PHP: if(empty($wxConfig['appID']))$this->out->e(201,'微信登录appID未配置');
    if app_id.is_empty() {
        res.render(Json(SignedApiResponse::<()>::error("微信登录appID未配置", 201, app_key)));
        return;
    }

    // PHP: if(empty($wxConfig['appSecret']))$this->out->e(201,'微信登录appSecret未配置');
    if app_secret.is_empty() {
        res.render(Json(SignedApiResponse::<()>::error("微信登录appSecret未配置", 201, app_key)));
        return;
    }

    let appid = app_info.id;
    let app_url = app_state.config().app().host().to_string();
    let current_time = Utc::now().timestamp();

    // PHP: $state = md5(uniqid(rand(),TRUE));
    let random_num: u64 = rand::thread_rng().r#gen();
    let state = {
        let mut state_data = String::with_capacity(64);
        use std::fmt::Write;
        let _ = write!(&mut state_data, "{}{}{}", current_time, random_num, appid);
        md5_to_str(&md5_hex(state_data.as_bytes())).to_string()
    };
    
    // 获取客户端IP
    let client_ip = get_client_ip(req);

    // PHP: $data = ['appid'=>$this->app['id'],'udid'=>$_POST['udid'],'ip'=>$this->ip,'invid'=>isset($_POST['invid']) && !empty($_POST['invid'])?$_POST['invid']:null,'wxConfig'=>$wxConfig];
    let wxlogon_info = WxLogonInfo {
        appid,
        udid: wx_req.udid.clone(),
        ip: client_ip,
        invid: wx_req.invid,
        wx_config: wx_config.clone(),
        create_time: current_time,
    };

    // PHP: $this->redis->setex('wxlogon_info_'.$state,600,json_encode($data));
    let redis_key = format!("wxlogon_info_{}", state);
    let redis_util = &app_state.redis_util;
    let redis_pool = match app_state.redis_pool.as_ref() {
        Some(pool) => pool,
        None => {
            res.render(Json(SignedApiResponse::<()>::error("Redis未初始化", 201, app_key)));
            return;
        }
    };
    
    let info_json = match serde_json::to_string(&wxlogon_info) {
        Ok(json) => json,
        Err(_) => {
            res.render(Json(SignedApiResponse::<()>::error("数据序列化失败", 201, app_key)));
            return;
        }
    };
    
    if let Err(e) = redis_util.setex(redis_pool, &redis_key, 600, &info_json).await {
        tracing::error!("Redis存储失败: {}", e);
        res.render(Json(SignedApiResponse::<()>::error("存储登录信息失败", 201, app_key)));
        return;
    }

    // PHP: $callback = urlencode(getUrl().'/api/user/wxlogonCallback');
    let callback_url = format!("{}/api/user/wxlogonCallback", app_url);
    let encoded_callback = encode(&callback_url);

    // PHP: $wxurl = "https://open.weixin.qq.com/connect/qrconnect?appid=".$wxConfig['appID']."&redirect_uri={$callback}&response_type=code&scope=snsapi_login&state={$state}#wechat_redirect";
    let wx_url = format!(
        "https://open.weixin.qq.com/connect/qrconnect?appid={}&redirect_uri={}&response_type=code&scope=snsapi_login&state={}#wechat_redirect",
        app_id, encoded_callback, state
    );

    // PHP: $this->out->setData(['url'=>$wxurl,'uuid'=>$state])->e(200,'获取成功');
    res.render(Json(SignedApiResponse::success(app_key, Some(json!({
        "url": wx_url,
        "uuid": state
    })))));
}

/// 获取客户端IP
fn get_client_ip(req: &Request) -> String {
    if let Some(x_real_ip) = req.headers().get("X-Real-IP")
        && let Ok(ip) = x_real_ip.to_str() {
            return ip.to_string();
        }
    
    if let Some(x_forwarded_for) = req.headers().get("X-Forwarded-For")
        && let Ok(ip) = x_forwarded_for.to_str() {
            return ip.split(',').next().unwrap_or("").trim().to_string();
        }

    "127.0.0.1".to_string()
}