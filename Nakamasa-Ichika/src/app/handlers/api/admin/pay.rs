//! Admin Pay controller
//! 管理员支付控制器

use salvo::prelude::*;
use serde::Deserialize;
use std::sync::Arc;
use chrono::Utc;

use crate::app::utils::response::ApiResponse;
use crate::app::utils::validator::Validator;
use crate::app::plugins::pay::{PayPlugin, JiePayPlugin, AliPayPlugin, WxPayPlugin};
use crate::core::AppState;

/// 获取支付插件列表和配置信息
#[handler]
pub async fn get_info(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let app_state = depot.obtain::<Arc<AppState>>().unwrap();
    
    // 创建插件实例获取元数据
    let jie_plugin = JiePayPlugin::new();
    let ali_plugin = AliPayPlugin::new();
    let wx_plugin = WxPayPlugin::new();
    
    let pay_plug = serde_json::json!([
        {
            "id": jie_plugin.plugin_type(),
            "type": jie_plugin.config_form()["type"],
            "name": jie_plugin.name(),
            "extra": "申请地址: http://pay.jienet.com 邀请码: NzMy",
            "form": jie_plugin.config_form()["form"]
        },
        {
            "id": wx_plugin.plugin_type(),
            "type": wx_plugin.config_form()["type"],
            "name": wx_plugin.name(),
            "form": wx_plugin.config_form()["form"]
        },
        {
            "id": ali_plugin.plugin_type(),
            "type": ali_plugin.config_form()["type"],
            "name": ali_plugin.name(),
            "form": ali_plugin.config_form()["form"]
        }
    ]);
    
    // 获取appid
    let appid = match req.headers().get("appid") {
        Some(h) => match h.to_str() {
            Ok(s) => match s.parse::<u64>() {
                Ok(id) => id,
                Err(_) => {
                    res.render(Json(ApiResponse::<()>::error("APPID格式错误", 201)));
                    return;
                }
            },
            Err(_) => {
                res.render(Json(ApiResponse::<()>::error("APPID格式错误", 201)));
                return;
            }
        },
        None => {
            res.render(Json(ApiResponse::<()>::error("APPID不能为空", 201)));
            return;
        }
    };
    
    // 从数据库获取当前应用的支付配置
    let pay_config = sqlx::query_as::<_, (
        u64, Option<String>, Option<String>, Option<Vec<u8>>,
        Option<String>, Option<String>, Option<Vec<u8>>
    )>(
        "SELECT id, pay_ali_state, pay_ali_type, pay_ali_config, pay_wx_state, pay_wx_type, pay_wx_config FROM u_app WHERE id = ?"
    )
    .bind(appid)
    .fetch_optional(app_state.get_db())
    .await;

    let info = match pay_config {
        Ok(Some(row)) => {
            // 将BLOB转换为JSON对象
            let ali_config_json: serde_json::Value = row.3
                .and_then(|bytes| String::from_utf8(bytes).ok())
                .and_then(|s| {
                    if s.trim().is_empty() {
                        None
                    } else {
                        serde_json::from_str(&s).ok()
                    }
                })
                .unwrap_or_else(|| serde_json::json!({}));

            let wx_config_json: serde_json::Value = row.6
                .and_then(|bytes| String::from_utf8(bytes).ok())
                .and_then(|s| {
                    if s.trim().is_empty() {
                        None
                    } else {
                        serde_json::from_str(&s).ok()
                    }
                })
                .unwrap_or_else(|| serde_json::json!({}));

            serde_json::json!({
                "id": row.0,
                "pay_ali_state": row.1.unwrap_or_else(|| "off".to_string()),
                "pay_ali_type": row.2.unwrap_or_else(|| "jie".to_string()),
                "pay_ali_config": ali_config_json,
                "pay_wx_state": row.4.unwrap_or_else(|| "off".to_string()),
                "pay_wx_type": row.5.unwrap_or_else(|| "jie".to_string()),
                "pay_wx_config": wx_config_json
            })
        },
        Ok(None) => serde_json::json!({
            "id": appid,
            "pay_ali_state": "off",
            "pay_ali_type": "jie",
            "pay_ali_config": null,
            "pay_wx_state": "off",
            "pay_wx_type": "jie",
            "pay_wx_config": null
        }),
        Err(e) => {
            tracing::error!("查询支付配置失败: {}", e);
            res.render(Json(ApiResponse::<()>::error("数据库错误", 201)));
            return;
        }
    };
    
    res.render(Json(ApiResponse::success("成功", Some(serde_json::json!({
        "info": info,
        "plug": pay_plug
    })))));
}

/// 编辑支付配置
#[derive(Debug, Deserialize)]
struct EditPayRequest {
    id: u64,
    #[serde(default)]
    pay_ali_state: Option<String>,
    #[serde(default)]
    pay_ali_type: Option<String>,
    #[serde(default)]
    pay_ali_config: Option<serde_json::Value>,
    #[serde(default)]
    pay_wx_state: Option<String>,
    #[serde(default)]
    pay_wx_type: Option<String>,
    #[serde(default)]
    pay_wx_config: Option<serde_json::Value>,
}

#[handler]
pub async fn edit(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let app_state = depot.obtain::<Arc<AppState>>().unwrap();
    
    let edit_req = match req.parse_json::<EditPayRequest>().await {
        Ok(data) => data,
        Err(_) => {
            res.render(Json(ApiResponse::<()>::error("参数解析失败", 201)));
            return;
        }
    };

    // 验证参数
    let mut validator = Validator::new();
    validator.int("id", edit_req.id as i64, 1, 99999999999);
    
    // 验证pay_ali_type
    if let Some(ref ali_type) = edit_req.pay_ali_type {
        if ali_type.len() < 2 || ali_type.len() > 12 {
            res.render(Json(ApiResponse::<()>::error("支付宝支付引擎不规范", 201)));
            return;
        }
        if !ali_type.chars().all(|c| c.is_alphanumeric()) {
            res.render(Json(ApiResponse::<()>::error("支付宝支付引擎不规范", 201)));
            return;
        }
    }
    
    // 验证pay_wx_type
    if let Some(ref wx_type) = edit_req.pay_wx_type {
        if wx_type.len() < 2 || wx_type.len() > 12 {
            res.render(Json(ApiResponse::<()>::error("微信支付引擎不规范", 201)));
            return;
        }
        if !wx_type.chars().all(|c| c.is_alphanumeric()) {
            res.render(Json(ApiResponse::<()>::error("微信支付引擎不规范", 201)));
            return;
        }
    }
    
    // 验证pay_ali_config必须是对象
    if let Some(ref ali_config) = edit_req.pay_ali_config
        && !ali_config.is_object() {
            res.render(Json(ApiResponse::<()>::error("支付宝支付参数不规范", 201)));
            return;
        }

    // 验证pay_wx_config必须是对象
    if let Some(ref wx_config) = edit_req.pay_wx_config
        && !wx_config.is_object() {
            res.render(Json(ApiResponse::<()>::error("微信支付参数不规范", 201)));
            return;
        }
    
    if let Some(ref state) = edit_req.pay_ali_state
        && state != "on" && state != "off" {
            res.render(Json(ApiResponse::<()>::error("支付宝控制状态设置有误", 201)));
            return;
        }
    
    if let Some(ref state) = edit_req.pay_wx_state
        && state != "on" && state != "off" {
            res.render(Json(ApiResponse::<()>::error("微信控制状态设置有误", 201)));
            return;
        }
    
    if let Err(msg) = validator.validate() {
        res.render(Json(ApiResponse::<()>::error(msg, 201)));
        return;
    }

    // 准备更新数据
    let ali_config_json = edit_req.pay_ali_config.map(|v| serde_json::to_string(&v).unwrap_or_else(|_| String::new()));
    let wx_config_json = edit_req.pay_wx_config.map(|v| serde_json::to_string(&v).unwrap_or_else(|_| String::new()));
    
    // 转换为Vec<u8>以适配BLOB类型
    let ali_config_bytes = ali_config_json.as_ref().map(|s| s.as_bytes().to_vec());
    let wx_config_bytes = wx_config_json.as_ref().map(|s| s.as_bytes().to_vec());
    
    // 更新数据库
    let result = sqlx::query(
        "UPDATE u_app SET 
         pay_ali_state = ?, pay_ali_type = ?, pay_ali_config = ?,
         pay_wx_state = ?, pay_wx_type = ?, pay_wx_config = ?
         WHERE id = ?"
    )
    .bind(&edit_req.pay_ali_state)
    .bind(&edit_req.pay_ali_type)
    .bind(&ali_config_bytes)
    .bind(&edit_req.pay_wx_state)
    .bind(&edit_req.pay_wx_type)
    .bind(&wx_config_bytes)
    .bind(edit_req.id)
    .execute(app_state.get_db())
    .await;

    match result {
        Ok(r) => {
            if r.rows_affected() > 0 {
                // 记录日志
                let admin_id = 1u64; // TODO: 从token获取
                let now = Utc::now().timestamp();
                let _ = sqlx::query(
                    "INSERT INTO u_logs (ug, uid, type, state, time, ip, appid) VALUES (?, ?, ?, ?, ?, ?, ?)"
                )
                .bind("adm")
                .bind(admin_id)
                .bind("pay_edit")
                .bind(true)
                .bind(now)
                .bind("127.0.0.1") // TODO: 获取真实IP
                .bind(Option::<i64>::None)
                .execute(app_state.get_db())
                .await;
                
                res.render(Json(ApiResponse::success_msg("编辑成功")));
            } else {
                res.render(Json(ApiResponse::<()>::error("编辑失败", 201)));
            }
        }
        Err(e) => {
            tracing::error!("更新支付配置失败: {}", e);
            res.render(Json(ApiResponse::<()>::error("数据库错误", 201)));
        }
    }
}