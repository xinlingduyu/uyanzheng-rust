//! 获取设备列表
//! 
//! 功能说明：
//! 获取当前用户已绑定的所有设备列表。
//!
//! 处理流程：
//! 1. 验证token参数
//! 2. 从用户sn_list字段解析设备列表
//! 3. 返回设备码和绑定时间列表

use salvo::prelude::*;
use std::sync::Arc;
use serde::{Deserialize, Serialize};

use crate::core::AppState;
use crate::app::utils::response::SignedApiResponse;
use crate::app::utils::validator::Validator;
use crate::app::models::requests::GetUdidRequest;
use crate::app::middleware::user_auth::UserInfo;
use crate::app::middleware::app_context::AppInfo;

/// 设备信息 - 匹配JSON响应格式
#[derive(Debug, Serialize, Deserialize)]
struct DeviceItem {
    time: i64,
    udid: String,
}

#[handler]
pub async fn get_udid(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let _app_state = depot.obtain::<Arc<AppState>>().unwrap();
    
    // 获取 app_key（零拷贝）
    let app_key = depot.get::<AppInfo>("app_info").map(|i| i.app_key.as_str()).unwrap_or("");
    
    let get_req = match req.parse_json::<GetUdidRequest>().await {
        Ok(data) => data,
        Err(_) => {
            res.render(Json(SignedApiResponse::<()>::error("参数解析失败", 201, app_key)));
            return;
        }
    };

    // PHP: 'token' => ['wordnum','32,32','TOKEN有误']
    let mut validator = Validator::new();
    validator.wordnum("token", &get_req.token, 32, 32);
    
    if let Err(msg) = validator.validate() {
        res.render(Json(SignedApiResponse::<()>::error(msg, 201, app_key)));
        return;
    }

    // 从 depot 获取用户信息（避免 clone，直接使用引用）
    let user_info = match depot.get::<UserInfo>("user_info") {
        Ok(info) => info,
        Err(_) => {
            res.render(Json(SignedApiResponse::<()>::error("未授权", 201, app_key)));
            return;
        }
    };

    // 直接从用户信息中获取设备列表（避免额外数据库查询）
    // PHP: $this->out->setData(['list'=>json_decode($this->user['sn_list'],true)])->e(200,'获取成功');
    let device_list: Vec<DeviceItem> = user_info.sn_list.as_ref()
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    res.render(Json(SignedApiResponse::success(app_key, Some(device_list))));
}
