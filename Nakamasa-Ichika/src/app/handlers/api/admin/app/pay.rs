//! Admin Pay controller
//! 管理员支付控制器
//!
//! ## 设计说明
//!
//! 支付通道采用「通道定义表」模式：将每个支付通道（支付宝、微信等）的定义
//! 集中在一个 const 数组中，包含其 ID、显示名称以及到数据库列的映射。
//! 新增支付通道只需在 `CHANNEL_DEFS` 中追加一条定义，后端自动处理
//! 查询/保存，前端自动渲染。

use chrono::Utc;
use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::app::plugins::pay::{AliPayPlugin, JiePayPlugin, PayPlugin, WxPayPlugin};
use crate::app::utils::response::ApiResponse;
use crate::app::utils::validator::Validator;
use crate::core::middleware::get_client_ip;
use crate::core::AppState;

// ============================================================================
// 通道定义
// ============================================================================

/// 支付通道定义 —— 扩展支付方式只需在此处追加条目
const CHANNEL_DEFS: &[ChannelDef] = &[
    ChannelDef {
        id: "ali",
        label: "支付宝",
        icon: "icon-alipay",
        state_col: "pay_ali_state",
        type_col: "pay_ali_type",
        config_col: "pay_ali_config",
    },
    ChannelDef {
        id: "wx",
        label: "微信支付",
        icon: "icon-wechat",
        state_col: "pay_wx_state",
        type_col: "pay_wx_type",
        config_col: "pay_wx_config",
    },
];

struct ChannelDef {
    /// 通道 ID，唯一标识（如 "ali", "wx"）
    id: &'static str,
    /// 显示名称（如 "支付宝", "微信支付"）
    label: &'static str,
    /// 图标标识（给前端用）
    icon: &'static str,
    /// 数据库状态列名（如 "pay_ali_state"）
    state_col: &'static str,
    /// 数据库引擎类型列名（如 "pay_ali_type"）
    type_col: &'static str,
    /// 数据库配置 BLOB 列名（如 "pay_ali_config"）
    config_col: &'static str,
}

// ============================================================================
// 获取支付配置
// ============================================================================

/// 获取支付插件列表和配置信息
#[handler]
pub async fn get_info(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let app_state = match depot.obtain::<Arc<AppState>>() {
        Ok(s) => s,
        Err(_) => {
            res.render(Json(ApiResponse::<()>::error("服务器错误", 201)));
            return;
        }
    };

    // 获取 appid
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

    // 查询应用的一行数据，SELECT 所有支付相关列
    let row = sqlx::query_as::<_, PayDbRow>(
        "SELECT id, pay_ali_state, pay_ali_type, pay_ali_config, pay_wx_state, pay_wx_type, pay_wx_config FROM u_app WHERE id = ?"
    )
    .bind(appid)
    .fetch_optional(app_state.get_db())
    .await;

    let row = match row {
        Ok(Some(r)) => r,
        Ok(None) => {
            res.render(Json(ApiResponse::<()>::error("应用不存在", 201)));
            return;
        }
        Err(e) => {
            tracing::error!("查询支付配置失败: {}", e);
            res.render(Json(ApiResponse::<()>::error("数据库错误", 201)));
            return;
        }
    };

    // 根据通道定义表生成 channels 数组
    let channels: Vec<serde_json::Value> = CHANNEL_DEFS
        .iter()
        .map(|def| {
            let (state, r#type, config) = row.get_channel(def);
            channel_json(def, state, r#type, config)
        })
        .collect();

    // 生成插件列表（与之前不变）
    let plugins = generate_plugins_json();

    res.render(Json(ApiResponse::success(
        "成功",
        Some(serde_json::json!({
            "channels": channels,
            "plugins": plugins,
        })),
    )));
}

// ============================================================================
// 编辑支付配置
// ============================================================================

/// 编辑支付配置请求体
#[derive(Debug, Deserialize)]
struct EditPayRequest {
    id: u64,
    /// 通道配置数组
    channels: Vec<ChannelConfig>,
}

/// 单个通道的配置
#[derive(Debug, Deserialize)]
struct ChannelConfig {
    /// 通道 ID（需匹配 CHANNEL_DEFS 中的 id）
    #[serde(rename = "id")]
    channel_id: String,
    /// 启用状态 "on" / "off"
    state: String,
    /// 选择的支付引擎类型（如 "jie", "ali", "wx"）
    #[serde(rename = "type")]
    plugin_type: String,
    /// 引擎配置参数（JSON 对象）
    config: serde_json::Value,
}

#[handler]
pub async fn edit(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let app_state = match depot.obtain::<Arc<AppState>>() {
        Ok(s) => s,
        Err(_) => {
            res.render(Json(ApiResponse::<()>::error("服务器错误", 201)));
            return;
        }
    };

    let edit_req = match req.parse_json::<EditPayRequest>().await {
        Ok(data) => data,
        Err(_) => {
            res.render(Json(ApiResponse::<()>::error("参数解析失败", 201)));
            return;
        }
    };

    // 验证 id
    let mut validator = Validator::new();
    validator.int("id", edit_req.id as i64, 1, 99999999999);
    if let Err(msg) = validator.validate() {
        res.render(Json(ApiResponse::<()>::error(msg, 201)));
        return;
    }

    // 验证每个通道，收集需要更新的列和值
    let mut ali_state: Option<String> = None;
    let mut ali_type: Option<String> = None;
    let mut ali_config: Option<Vec<u8>> = None;
    let mut wx_state: Option<String> = None;
    let mut wx_type: Option<String> = None;
    let mut wx_config: Option<Vec<u8>> = None;

    for ch in &edit_req.channels {
        // 根据通道 ID 查找定义
        let def = CHANNEL_DEFS.iter().find(|d| d.id == ch.channel_id);
        let def = match def {
            Some(d) => d,
            None => {
                res.render(Json(ApiResponse::<()>::error(
                    format!("未知的支付通道: {}", ch.channel_id),
                    201,
                )));
                return;
            }
        };

        // 验证 state
        if ch.state != "on" && ch.state != "off" {
            res.render(Json(ApiResponse::<()>::error(
                format!("{} 状态设置有误", def.label),
                201,
            )));
            return;
        }

        // 验证 plugin_type
        if ch.plugin_type.len() < 2 || ch.plugin_type.len() > 12
            || !ch.plugin_type.chars().all(|c| c.is_alphanumeric())
        {
            res.render(Json(ApiResponse::<()>::error(
                format!("{} 支付引擎不规范", def.label),
                201,
            )));
            return;
        }

        // 验证 config 必须是对象
        if !ch.config.is_object() {
            res.render(Json(ApiResponse::<()>::error(
                format!("{} 支付参数不规范", def.label),
                201,
            )));
            return;
        }

        // 序列化 config
        let config_bytes = match serde_json::to_string(&ch.config) {
            Ok(s) => Some(s.as_bytes().to_vec()),
            Err(e) => {
                tracing::error!("{} 配置序列化失败: {}", def.label, e);
                res.render(Json(ApiResponse::<()>::error(
                    format!("{} 配置序列化失败", def.label),
                    201,
                )));
                return;
            }
        };

        // 按通道列名映射到对应变量
        match def.id {
            "ali" => {
                ali_state = Some(ch.state.clone());
                ali_type = Some(ch.plugin_type.clone());
                ali_config = config_bytes;
            }
            "wx" => {
                wx_state = Some(ch.state.clone());
                wx_type = Some(ch.plugin_type.clone());
                wx_config = config_bytes;
            }
            _ => {
                res.render(Json(ApiResponse::<()>::error(
                    format!("不支持的支付通道: {}", def.id),
                    201,
                )));
                return;
            }
        }
    }

    // 更新数据库
    let result = sqlx::query(
        "UPDATE u_app SET 
         pay_ali_state = ?, pay_ali_type = ?, pay_ali_config = ?,
         pay_wx_state = ?, pay_wx_type = ?, pay_wx_config = ?
         WHERE id = ?",
    )
    .bind(&ali_state)
    .bind(&ali_type)
    .bind(&ali_config)
    .bind(&wx_state)
    .bind(&wx_type)
    .bind(&wx_config)
    .bind(edit_req.id)
    .execute(app_state.get_db())
    .await;

    match result {
        Ok(r) => {
            if r.rows_affected() > 0 {
                let admin_id = depot
                    .get::<u64>("admin_id")
                    .copied()
                    .unwrap_or(0);
                let ip = get_client_ip(req).to_string();
                let now = Utc::now().timestamp();
                let _ = sqlx::query(
                    "INSERT INTO u_logs (ug, uid, type, state, time, ip, appid) VALUES (?, ?, ?, ?, ?, ?, ?)"
                )
                .bind("adm")
                .bind(admin_id)
                .bind("pay_edit")
                .bind(true)
                .bind(now)
                .bind(&ip)
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

// ============================================================================
// 辅助函数
// ============================================================================

/// 数据库查询结果行
#[derive(Debug, sqlx::FromRow)]
struct PayDbRow {
    id: u64,
    pay_ali_state: Option<String>,
    pay_ali_type: Option<String>,
    pay_ali_config: Option<Vec<u8>>,
    pay_wx_state: Option<String>,
    pay_wx_type: Option<String>,
    pay_wx_config: Option<Vec<u8>>,
}

impl PayDbRow {
    /// 根据通道定义提取对应的 (state, type, config_json)
    fn get_channel(&self, def: &ChannelDef) -> (String, String, serde_json::Value) {
        match def.id {
            "ali" => (
                self.pay_ali_state.clone().unwrap_or_else(|| "off".to_string()),
                self.pay_ali_type.clone().unwrap_or_else(|| "jie".to_string()),
                blob_to_json(&self.pay_ali_config),
            ),
            "wx" => (
                self.pay_wx_state.clone().unwrap_or_else(|| "off".to_string()),
                self.pay_wx_type.clone().unwrap_or_else(|| "jie".to_string()),
                blob_to_json(&self.pay_wx_config),
            ),
            _ => ("off".to_string(), String::new(), serde_json::json!({})),
        }
    }
}

/// 将 BLOB 字节转换为 JSON Value
fn blob_to_json(bytes: &Option<Vec<u8>>) -> serde_json::Value {
    bytes
        .as_ref()
        .and_then(|b| String::from_utf8(b.clone()).ok())
        .and_then(|s| {
            if s.trim().is_empty() {
                None
            } else {
                serde_json::from_str(&s).ok()
            }
        })
        .unwrap_or_else(|| serde_json::json!({}))
}

/// 为单个通道生成 JSON 对象
fn channel_json(def: &ChannelDef, state: String, r#type: String, config: serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "id": def.id,
        "label": def.label,
        "icon": def.icon,
        "state": state,
        "type": r#type,
        "config": config,
    })
}

/// 生成可用支付引擎列表
fn generate_plugins_json() -> Vec<serde_json::Value> {
    let jie_plugin = JiePayPlugin::new();
    let ali_plugin = AliPayPlugin::new();
    let wx_plugin = WxPayPlugin::new();

    vec![
        serde_json::json!({
            "id": jie_plugin.plugin_type(),
            "type": jie_plugin.config_form()["type"],
            "name": jie_plugin.name(),
            "extra": "申请地址: http://pay.jienet.com 邀请码: NzMy",
            "form": jie_plugin.config_form()["form"]
        }),
        serde_json::json!({
            "id": wx_plugin.plugin_type(),
            "type": wx_plugin.config_form()["type"],
            "name": wx_plugin.name(),
            "form": wx_plugin.config_form()["form"]
        }),
        serde_json::json!({
            "id": ali_plugin.plugin_type(),
            "type": ali_plugin.config_form()["type"],
            "name": ali_plugin.name(),
            "form": ali_plugin.config_form()["form"]
        }),
    ]
}