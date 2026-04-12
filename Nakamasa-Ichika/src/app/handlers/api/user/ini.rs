//! 获取配置
//! 返回应用配置信息：版本、公告、扩展配置
//! 注意: tokenCheck=false，不需要认证

use salvo::prelude::*;
use std::sync::Arc;
use std::collections::HashMap;
use serde::Serialize;

use crate::core::AppState;
use crate::app::middleware::app_context::AppInfo;
use crate::app::utils::response::{SignedApiResponse, render_success, render_success_msg, render_success_with_msg, render_error};

/// 版本信息
#[derive(Serialize)]
struct VersionData {
    /// 当前版本
    current: String,
    /// 最新版本
    latest: String,
    /// 最新版本更新内容
    latest_content: String,
    /// 最新版本下载地址
    latest_url: String,
    /// 版本号数值（如 1.2.0 → 1002000）
    number: i64,
}

/// 公告信息
#[derive(Serialize)]
struct NoticeData {
    id: u64,
    visit: i64,
    content: String,
    time: i64,
}

/// 响应数据
#[derive(Serialize)]
struct IniData {
    #[serde(skip_serializing_if = "Option::is_none")]
    extend: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    notice: Option<NoticeData>,
    version: VersionData,
}

/// 将版本号字符串转换为数值
/// 格式: 主版本号(3位) + 次版本号(3位) + 修订号(3位)
/// 例如: "1.2.0" → 1002000, "2.10.5" → 2010005
#[inline]
fn version_to_number(version: &str) -> i64 {
    let mut parts = version.split('.');
    let major = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let minor = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let patch = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    major * 1_000_000 + minor * 1_000 + patch
}

#[handler]
pub async fn ini(_req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let app_state = depot.obtain::<Arc<AppState>>().unwrap();

    // 从 depot 获取应用信息（避免 clone）
    let app_info = match depot.get::<AppInfo>("app_info") {
        Ok(info) => info,
        Err(_) => {
            render_error(res, "应用不存在", 201, "");
            return;
        }
    };

    let appid = app_info.id;
    let app_key = &app_info.app_key;

    // 获取最新版本信息（从数据库查询最新发布的版本）
    let latest_version_result = sqlx::query_as::<_, (String, Option<String>, Option<String>)>(
        "SELECT ver_val, ver_content, ver_url FROM u_app_ver WHERE appid = ? AND ver_state = 'on' ORDER BY ver_major DESC, ver_minor DESC, ver_patch DESC LIMIT 1"
    )
    .bind(appid)
    .fetch_optional(app_state.get_db())
    .await;

    // 获取当前客户端版本（从请求参数或默认）
    let current_version = depot.get::<String>("app_version")
        .map(|s| s.as_str())
        .unwrap_or("1.0.0");

    // 解析最新版本信息
    let (latest, latest_content, latest_url) = match latest_version_result {
        Ok(Some((ver_val, content, url))) => {
            (ver_val, content.unwrap_or_default(), url.unwrap_or_default())
        }
        _ => (current_version.to_string(), String::new(), String::new())
    };

    // 构建版本数据
    let version_data = VersionData {
        current: current_version.to_string(),
        latest: latest.clone(),
        latest_content,
        latest_url,
        number: version_to_number(&latest),
    };

    // 获取最新通知（appid 为 NULL 表示全局公告，所有应用都可见）
    let notice_data = fetch_notice(app_state.get_db(), appid).await;

    // 获取扩展配置
    let extend_data = fetch_extend(app_state.get_db(), appid).await;

    // 构建响应数据
    let data = IniData {
        extend: extend_data,
        notice: notice_data,
        version: version_data,
    };

    render_success(res, app_key, Some(data), app_info.mi.as_ref());
}

/// 获取公告信息
async fn fetch_notice(pool: &sqlx::MySqlPool, appid: u64) -> Option<NoticeData> {
    // 先尝试获取应用专属公告
    let result = sqlx::query_as::<_, (u64, i64, Option<String>, Option<i64>)>(
        "SELECT id, visit, content, time FROM u_app_notice WHERE appid = ? ORDER BY id DESC LIMIT 1"
    )
    .bind(appid)
    .fetch_optional(pool)
    .await;

    let (notice_id, visit, content, time) = match result {
        Ok(Some(row)) => row,
        Ok(None) => {
            // 尝试获取全局公告
            let global = sqlx::query_as::<_, (u64, i64, Option<String>, Option<i64>)>(
                "SELECT id, visit, content, time FROM u_app_notice WHERE appid IS NULL ORDER BY id DESC LIMIT 1"
            )
            .fetch_optional(pool)
            .await;
            
            match global {
                Ok(Some(row)) => row,
                _ => return None,
            }
        }
        Err(e) => {
            tracing::warn!("查询公告失败: {}", e);
            return None;
        }
    };

    // 更新访问次数
    let _ = sqlx::query("UPDATE u_app_notice SET visit = visit + 1 WHERE id = ?")
        .bind(notice_id)
        .execute(pool)
        .await;

    Some(NoticeData {
        id: notice_id,
        visit,
        content: content.unwrap_or_default(),
        time: time.unwrap_or(0),
    })
}

/// 获取扩展配置
async fn fetch_extend(pool: &sqlx::MySqlPool, appid: u64) -> Option<serde_json::Value> {
    let result = sqlx::query_as::<_, (String, String)>(
        "SELECT var_key, var_val FROM u_app_extend WHERE appid = ? OR appid IS NULL ORDER BY id DESC"
    )
    .bind(appid)
    .fetch_all(pool)
    .await;

    let rows = result.ok()?;
    if rows.is_empty() {
        return None;
    }

    // 使用HashMap来分组相同key的值
    let mut app_exten: HashMap<String, Vec<String>> = HashMap::new();
    for (var_key, var_val) in rows {
        app_exten.entry(var_key).or_default().push(var_val);
    }

    // 构建extend - PHP逻辑：多个值保持数组，单个值提取值
    let mut extend = serde_json::Map::new();
    for (k, v) in app_exten {
        if v.len() > 1 {
            extend.insert(k, serde_json::Value::Array(v.into_iter().map(serde_json::Value::String).collect()));
        } else if let Some(first) = v.into_iter().next() {
            extend.insert(k, serde_json::Value::String(first));
        }
    }

    if extend.is_empty() {
        None
    } else {
        Some(serde_json::Value::Object(extend))
    }
}