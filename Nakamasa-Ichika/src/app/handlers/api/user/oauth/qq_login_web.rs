//! QQ网页登录
//!
//! 功能说明：
//! 获取QQ互联网页扫码登录URL，用于PC端网页登录。
//!
//! 处理流程：
//! 1. 验证udid参数（设备标识）
//! 2. 获取应用的QQ登录配置
//! 3. 生成state标识并存储登录信息到Redis
//! 4. 构建QQ授权登录URL
//! 5. 返回登录URL和state供前端跳转

use chrono::Utc;
use rand::Rng;
use salvo::prelude::*;
use std::sync::Arc;
use urlencoding::encode;

use crate::app::middleware::app_context::AppInfo;
use crate::app::models::requests::WxLogonRequest;
use crate::app::utils::response::{
    render_error, render_success,
};
use crate::app::utils::validator::Validator;
use crate::core::AppState;
use crate::core::md5_optimize::md5_concat_ints;
use crate::core::middleware::get_client_ip;
use serde::{Deserialize, Serialize};
use serde_json::json;

/// QQ登录信息 - 存储在Redis中
#[derive(Debug, Clone, Serialize, Deserialize)]
struct QqLogonInfo {
    appid: u64,
    udid: String,
    ip: String,
    invid: Option<i64>,
    create_time: i64,
}

#[handler]
pub async fn qq_login_web(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let app_state = match depot.obtain::<Arc<AppState>>() {
        Ok(s) => s,
        Err(_) => {
            render_error(res, "服务器错误", 201, "");
            return;
        }
    };

    // 获取应用信息（零拷贝）
    let app_info = match depot.get::<AppInfo>("app_info") {
        Ok(info) => info,
        Err(_) => {
            render_error(res, "应用信息不存在", 201, "");
            return;
        }
    };
    let app_key = app_info.app_key.as_str();

    let qq_req = match req.parse_json::<WxLogonRequest>().await {
        Ok(data) => data,
        Err(_) => {
            render_error(res, "参数解析失败", 201, app_key);
            return;
        }
    };

    // PHP: $checkRules = ['invid' => ['int','1,11','邀请人ID填写有误',true], 'udid' => ['reg','[a-zA-Z0-9_-]+','机器码有误']];
    let mut validator = Validator::new();
    validator.reg("udid", &qq_req.udid, "[a-zA-Z0-9_-]+");

    if let Err(msg) = validator.validate() {
        render_error(res, msg, 201, app_key);
        return;
    }

    // PHP: if($this->app['app_type'] != 'user')$this->out->e(115);
    if app_info.app_type != "user" {
        render_error(res, "当前应用不支持调用该接口", 115, app_key);
        return;
    }

    // PHP: if(empty($this->app['logon_qqopen_config']))$this->out->e(201,'QQ登录未配置');
    let qq_config_str = match &app_info.logon_open_qqconfig {
        Some(config) => config,
        None => {
            render_error(res, "QQ登录未配置", 201, app_key);
            return;
        }
    };

    // PHP: $qqConf = json_decode($this->app['logon_qqopen_config'],true);
    let qq_config: serde_json::Value = match serde_json::from_str(qq_config_str) {
        Ok(json) => json,
        Err(_) => {
            render_error(res, "QQ登录配置有误", 201, app_key);
            return;
        }
    };

    // PHP: if(!$qqConf || !isset($qqConf['appID']) || !isset($qqConf['state']) || !isset($qqConf['appKey']))$this->out->e(201,'微信登录配置有误');
    let app_id = qq_config
        .get("appID")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let app_key_qq = qq_config
        .get("appKey")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let state_config = qq_config
        .get("state")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // PHP: if($qqConf['state'] != 'on')$this->out->e(201,'QQ登录未开启');
    if state_config != "on" {
        render_error(res, "QQ登录未开启", 201, app_key);
        return;
    }

    // PHP: if(empty($qqConf['appID']))$this->out->e(201,'QQ登录appID未配置');
    if app_id.is_empty() {
        render_error(res, "QQ登录appID未配置", 201, app_key);
        return;
    }

    // PHP: if(empty($qqConf['appKey']))$this->out->e(201,'QQ登录appKey未配置');
    if app_key_qq.is_empty() {
        render_error(res, "QQ登录appKey未配置", 201, app_key);
        return;
    }

    let appid = app_info.id;
    let app_url = app_state.config().app().host().to_string();
    let current_time = Utc::now().timestamp();

    // PHP: $state = md5(uniqid(rand(),TRUE));
    let random_num: u64 = rand::thread_rng().r#gen();
    let state = md5_concat_ints(current_time, random_num as i64, appid as i64);

    // 获取客户端IP
    let client_ip = get_client_ip(req);

    // PHP: $data = ['appid'=>$this->app['id'],'udid'=>$_POST['udid'],'ip'=>$this->ip,'invid'=>isset($_POST['invid']) && !empty($_POST['invid'])?$_POST['invid']:null];
    let qqlogon_info = QqLogonInfo {
        appid,
        udid: qq_req.udid.clone(),
        ip: client_ip.to_string(),
        invid: qq_req.invid,
        create_time: current_time,
    };

    // PHP: $this->redis->setex('qqlogon_info_'.$state,600,json_encode($data));
    let redis_key = format!("qqlogon_info_{}", state);
    let redis_util = &app_state.redis_util;
    let redis_pool = match app_state.redis_pool.as_ref() {
        Some(pool) => pool,
        None => {
            render_error(res, "Redis未初始化", 201, app_key);
            return;
        }
    };

    let info_json = match serde_json::to_string(&qqlogon_info) {
        Ok(json) => json,
        Err(_) => {
            render_error(res, "数据序列化失败", 201, app_key);
            return;
        }
    };

    if let Err(e) = redis_util
        .setex(redis_pool, &redis_key, 600, &info_json)
        .await
    {
        tracing::error!("Redis存储失败: {}", e);
        render_error(res, "存储登录信息失败", 201, app_key);
        return;
    }

    // PHP: $callback = urlencode(getUrl().'/api/user/qqlogon/callback');
    let callback_url = format!("{}/api/user/qqlogonCallback", app_url);
    let encoded_callback = encode(&callback_url);

    // PHP: $url = "https://graph.qq.com/oauth2.0/authorize?response_type=code&client_id=".$qqConf['appID']."&redirect_uri={$callback}&state={$state}";
    let qq_url = format!(
        "https://graph.qq.com/oauth2.0/authorize?response_type=code&client_id={}&redirect_uri={}&state={}",
        app_id, encoded_callback, state
    );

    // PHP: $this->out->setData(['url'=>$url,'uuid'=>$state])->e(200,'获取成功');
    render_success(
        res,
        app_key,
        Some(json!({
            "url": qq_url,
            "uuid": state
        })),
        app_info.mi.as_ref(),
    );
}
