//! 解绑设备
//!
//! 功能说明：
//! 用户解绑指定的设备码，解绑后该设备无法自动登录。
//!
//! 处理流程：
//! 1. 验证token和udid参数
//! 2. 从用户sn_list中移除指定设备
//! 3. 删除该设备的在线状态
//! 4. 返回成功

use chrono::Utc;
use salvo::prelude::*;
use std::sync::Arc;

use crate::app::middleware::app_context::AppInfo;
use crate::app::middleware::user_auth::UserInfo;
use crate::app::models::common::DeviceInfo;
use crate::app::models::requests::ReUdidRequest;
use crate::app::utils::response::{
    SignedApiResponse, render_error, render_success, render_success_msg, render_success_with_msg,
};
use crate::app::utils::validator::Validator;
use crate::core::AppState;
use crate::core::middleware::get_client_ip;

#[handler]
pub async fn re_udid(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let app_state = match depot.obtain::<Arc<AppState>>() {
        Ok(s) => s,
        Err(_) => {
            render_error(res, "服务器错误", 201, "");
            return;
        }
    };

    // 获取应用信息
    let app_info = match depot.get::<AppInfo>("app_info") {
        Ok(info) => info,
        Err(_) => {
            render_error(res, "应用信息不存在", 201, "");
            return;
        }
    };
    let app_key = &app_info.app_key;
    let logon_sn_num = app_info.logon_sn_num;
    let logon_sn_unbdeVal = app_info.logon_sn_unbdeVal;
    let logon_sn_unbdeType = &app_info.logon_sn_unbdeType;

    let re_req = match req.parse_json::<ReUdidRequest>().await {
        Ok(data) => data,
        Err(_) => {
            render_error(res, "参数解析失败", 201, app_key);
            return;
        }
    };

    // PHP: 'token' => ['wordnum','32,32','TOKEN有误'],
    // PHP: 'udid' => ['reg','[a-zA-Z0-9_-]+','机器码有误']
    let mut validator = Validator::new();
    validator
        .wordnum("token", &re_req.token, 32, 32)
        .reg("udid", &re_req.udid, "[a-zA-Z0-9_-]+");

    if let Err(msg) = validator.validate() {
        render_error(res, msg, 201, app_key);
        return;
    }

    // 从 depot 获取用户信息（由 UserAuth 中间件提供）
    let user_info = match depot.get::<UserInfo>("user_info") {
        Ok(info) => info,
        Err(_) => {
            render_error(res, "未授权", 201, app_key);
            return;
        }
    };

    let (uid, appid) = (user_info.uid, user_info.appid);
    let user_type = user_info.user_type.as_str();
    let current_udid = user_info.udid.as_str();
    let token_state = user_info.token_state.as_str();
    let current_time = Utc::now().timestamp();
    let ip = get_client_ip(req);

    // PHP: $snList_Arr = json_decode($this->user['sn_list'],true);
    // 解析设备列表
    let sn_list_arr: Vec<DeviceInfo> = user_info
        .sn_list
        .clone()
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();

    // PHP: $find = false;
    // PHP: $snList = [];
    // PHP: foreach ($snList_Arr as $rows){
    //     if($rows['udid'] == $_POST['udid']){
    //         $find = true;
    //     }else{
    //         $snList[] = ['udid'=>$rows['udid'],'time'=>$rows['time']];
    //     }
    // }
    let mut found = false;
    let mut new_sn_list: Vec<DeviceInfo> = Vec::new();

    for device in &sn_list_arr {
        if device.udid == re_req.udid {
            found = true;
        } else {
            new_sn_list.push(device.clone());
        }
    }

    // PHP: if(!$find)$this->out->e(201,'解绑设备不存在');
    if !found {
        render_error(res, "解绑设备不存在", 201, app_key);
        return;
    }

    // PHP: if($this->user['tokenState'] == 'n' && count($snList) < $this->app['logon_sn_num']+$this->user['sn_max']){
    //     $snList[] = ['udid'=>$this->user['udid'],'time'=>time()];
    // }
    // 如果当前设备token_state为n且解绑后有空位，自动绑定当前设备
    let max_devices = (logon_sn_num + user_info.sn_max) as usize;
    if token_state == "n" && new_sn_list.len() < max_devices {
        new_sn_list.push(DeviceInfo {
            udid: current_udid.to_string(),
            time: current_time,
        });
    }

    let sn_list_json = serde_json::to_string(&new_sn_list).unwrap_or_default();

    // 构建更新数据
    let mut update_data = serde_json::json!({
        "sn_list": sn_list_json
    });

    // PHP: if($this->app['logon_sn_unbdeVal'] > 0){
    //     if($this->app['logon_sn_unbdeType'] == 'vip'){
    //         ...VIP消耗逻辑
    //     }else{
    //         ...积分消耗逻辑
    //     }
    // }

    if logon_sn_unbdeVal > 0 {
        if logon_sn_unbdeType == "vip" {
            // VIP消耗
            if user_type == "user" {
                // PHP: if($this->user['vip'] < time())$this->out->e(170);
                let user_vip = user_info.vip.unwrap_or(0);
                if user_vip < current_time {
                    render_error(res, "VIP到期无法解绑", 170, app_key);
                    return;
                }
                // PHP: if($this->user['vip'] < 9999999999){
                //     $data['vip'] = $this->user['vip']-$this->app['logon_sn_unbdeVal'];
                // }
                if user_vip < 9999999999 {
                    update_data["vip"] = serde_json::json!(user_vip - logon_sn_unbdeVal as i64);
                }
            } else {
                // 卡密用户VIP消耗
                let user_vip_exp = user_info.vip_exp.unwrap_or(0);
                if user_vip_exp < current_time {
                    render_error(res, "VIP到期无法解绑", 170, app_key);
                    return;
                }
                if user_vip_exp < 9999999999 {
                    update_data["vip_exp"] =
                        serde_json::json!(user_vip_exp - logon_sn_unbdeVal as i64);
                }
            }
        } else {
            // 积分消耗
            if user_type == "user" {
                // PHP: if($this->user['fen'] < $this->app['logon_sn_unbdeVal'])$this->out->e(171);
                if user_info.fen < logon_sn_unbdeVal as i64 {
                    render_error(res, "积分余额不足", 171, app_key);
                    return;
                }
                update_data["fen"] = serde_json::json!(user_info.fen - logon_sn_unbdeVal as i64);
            } else {
                // 卡密用户积分消耗
                let user_val = user_info.val.unwrap_or(0);
                if user_val < logon_sn_unbdeVal as i64 {
                    render_error(res, "积分余额不足", 171, app_key);
                    return;
                }
                update_data["val"] = serde_json::json!(user_val - logon_sn_unbdeVal as i64);
            }
        }
    }

    // 根据用户类型选择表
    let table_name = if user_type == "kami" {
        "u_cdk_kami"
    } else {
        "u_user"
    };

    // 构建动态SQL
    let update_fields: Vec<String> = update_data
        .as_object()
        .unwrap()
        .iter()
        .map(|(k, _)| format!("{} = ?", k))
        .collect();
    let update_sql = format!(
        "UPDATE {} SET {} WHERE id = ? AND appid = ?",
        table_name,
        update_fields.join(", ")
    );

    // 构建参数
    let mut query = sqlx::query(&update_sql);
    for (_k, v) in update_data.as_object().unwrap() {
        if let Some(s) = v.as_str() {
            query = query.bind(s);
        } else if let Some(n) = v.as_i64() {
            query = query.bind(n);
        }
    }
    query = query.bind(uid).bind(appid);

    let result = query.execute(app_state.get_db()).await;

    match result {
        Ok(r) => {
            if r.rows_affected() > 0 {
                // PHP: $this->log->u($this->app['app_type'],$this->user['id'])->add($res);
                // 记录日志
                let _ = sqlx::query(
                    "INSERT INTO u_logs (ug, uid, type, state, time, ip, appid) VALUES (?, ?, ?, ?, ?, ?, ?)"
                )
                .bind(user_type)
                .bind(uid)
                .bind("reUdid")
                .bind(true)
                .bind(current_time)
                .bind(ip)
                .bind(appid)
                .execute(app_state.get_db())
                .await;

                render_success(res, app_key, None::<()>, app_info.mi.as_ref());
            } else {
                render_error(res, "解绑失败", 201, app_key);
            }
        }
        Err(e) => {
            tracing::error!("解绑设备失败: {}", e);
            render_error(res, "解绑失败", 201, app_key);
        }
    }
}
