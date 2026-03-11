//! Admin System controller
//! 管理员系统控制器

use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::Row;

use crate::app::utils::response::ApiResponse;
use std::sync::Arc;
use crate::core::app_state::AppState;

#[derive(Debug, Serialize)]
struct UInfoData {
    exp_time: String,
    phone: String,
    r#type: String,  // 使用 r# 前缀来使用保留字作为字段名
    version: String,
}

#[handler]
pub async fn uinfo(_req: &mut Request, _depot: &mut Depot, res: &mut Response) {
    let data = UInfoData {
        exp_time: "永久".to_string(),
        phone: "185****0927".to_string(),
        r#type: "pro".to_string(),
        version: "3.3".to_string(),
    };

    res.render(Json(ApiResponse::success("成功", Some(data))));
}

#[derive(Debug, Deserialize)]
struct UloginRequest {
    user: String,
    password: String,
}

#[handler]
pub async fn ulogin(_req: &mut Request, _depot: &mut Depot, res: &mut Response) {
    res.render(Json(ApiResponse::success_msg("登录成功")));
}

#[derive(Debug, Serialize)]
struct UrefreshData {
    exp_time: String,
    phone: String,
    r#type: String,
    version: f32,
}

#[handler]
pub async fn urefresh(_req: &mut Request, _depot: &mut Depot, res: &mut Response) {
    let data = UrefreshData {
        exp_time: "永久".to_string(),
        phone: "185****0927".to_string(),
        r#type: "pro".to_string(),
        version: 3.3,
    };

    res.render(Json(ApiResponse::success("授权信息刷新成功", Some(data))));
}

#[handler]
pub async fn uquit(_req: &mut Request, _depot: &mut Depot, res: &mut Response) {
    res.render(Json(ApiResponse::success_msg("退出成功")));
}

#[handler]
pub async fn clear_cache(_req: &mut Request, _depot: &mut Depot, res: &mut Response) {
    // 清除缓存功能暂时返回成功
    // TODO: 实现实际的Redis缓存清除逻辑
    res.render(Json(ApiResponse::success_msg("清除缓存成功")));
}

#[handler]
pub async fn get_set(_req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let app_state = depot.obtain::<Arc<AppState>>().unwrap();
    
    let query = "SELECT `key`, `value` FROM u_settings";
    
    let result = sqlx::query_as::<_, (String, String)>(query)
        .fetch_all(app_state.get_db())
        .await;

    match result {
        Ok(rows) => {
            let settings: std::collections::HashMap<String, String> = rows.into_iter().collect();
            res.render(Json(ApiResponse::success("成功", Some(settings))));
        }
        Err(e) => {
            tracing::error!("获取设置失败: {}", e);
            res.render(Json(ApiResponse::<()>::error("获取设置失败", 201)));
        }
    }
}

#[derive(Debug, Deserialize)]
struct EditSetRequest {
    key: String,
    value: String,
}

#[handler]
pub async fn edit_set(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let app_state = depot.obtain::<Arc<AppState>>().unwrap();
    
    let edit_req = match req.parse_json::<EditSetRequest>().await {
        Ok(data) => data,
        Err(_) => {
            res.render(Json(ApiResponse::<()>::error("参数解析失败", 201)));
            return;
        }
    };

    // 检查是否存在，存在则更新，不存在则插入
    let check_query = "SELECT id FROM u_settings WHERE `key` = ?";
    let exists = sqlx::query(check_query)
        .bind(&edit_req.key)
        .fetch_optional(app_state.get_db())
        .await
        .is_ok();

    let result = if exists {
        sqlx::query("UPDATE u_settings SET `value` = ? WHERE `key` = ?")
            .bind(&edit_req.value)
            .bind(&edit_req.key)
            .execute(app_state.get_db())
            .await
    } else {
        sqlx::query("INSERT INTO u_settings (`key`, `value`) VALUES (?, ?)")
            .bind(&edit_req.key)
            .bind(&edit_req.value)
            .execute(app_state.get_db())
            .await
    };

    match result {
        Ok(_) => {
            res.render(Json(ApiResponse::success_msg("编辑设置成功")));
        }
        Err(e) => {
            tracing::error!("编辑设置失败: {}", e);
            res.render(Json(ApiResponse::<()>::error("编辑设置失败", 201)));
        }
    }
}

#[handler]
pub async fn get_user_api_router(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let app_state = depot.obtain::<Arc<AppState>>().unwrap();
    
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

    let query = "SELECT api_router FROM u_app WHERE id = ?";
    let result = sqlx::query(query)
        .bind(appid)
        .fetch_optional(app_state.get_db())
        .await;

    match result {
        Ok(Some(row)) => {
            if let Some(api_router) = row.get::<Option<String>, _>("api_router") {
                res.render(Json(ApiResponse::success("成功", Some(api_router))));
            } else {
                res.render(Json(ApiResponse::success("成功", Some("default"))));
            }
        }
        Ok(None) => {
            res.render(Json(ApiResponse::<()>::error("应用不存在", 201)));
        }
        Err(e) => {
            tracing::error!("获取用户API路由失败: {}", e);
            res.render(Json(ApiResponse::<()>::error("获取用户API路由失败", 201)));
        }
    }
}

#[derive(Debug, Deserialize)]
struct EditUserApiRouterRequest {
    api_router: String,
}

#[handler]
pub async fn edit_user_api_router(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let app_state = depot.obtain::<Arc<AppState>>().unwrap();
    
    let edit_req = match req.parse_json::<EditUserApiRouterRequest>().await {
        Ok(data) => data,
        Err(_) => {
            res.render(Json(ApiResponse::<()>::error("参数解析失败", 201)));
            return;
        }
    };

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

    let result = sqlx::query("UPDATE u_app SET api_router = ? WHERE id = ?")
        .bind(&edit_req.api_router)
        .bind(appid)
        .execute(app_state.get_db())
        .await;

    match result {
        Ok(r) => {
            if r.rows_affected() > 0 {
                res.render(Json(ApiResponse::success_msg("编辑用户API路由成功")));
            } else {
                res.render(Json(ApiResponse::<()>::error("编辑用户API路由失败", 201)));
            }
        }
        Err(e) => {
            tracing::error!("编辑用户API路由失败: {}", e);
            res.render(Json(ApiResponse::<()>::error("编辑用户API路由失败", 201)));
        }
    }
}

#[handler]
pub async fn switch_user_api_router(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let app_state = depot.obtain::<Arc<AppState>>().unwrap();
    
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

    // 切换到默认路由
    let result = sqlx::query("UPDATE u_app SET api_router = 'default' WHERE id = ?")
        .bind(appid)
        .execute(app_state.get_db())
        .await;

    match result {
        Ok(r) => {
            if r.rows_affected() > 0 {
                res.render(Json(ApiResponse::success_msg("切换用户API路由成功")));
            } else {
                res.render(Json(ApiResponse::<()>::error("切换用户API路由失败", 201)));
            }
        }
        Err(e) => {
            tracing::error!("切换用户API路由失败: {}", e);
            res.render(Json(ApiResponse::<()>::error("切换用户API路由失败", 201)));
        }
    }
}

#[handler]
pub async fn get_user_api_code(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let app_state = depot.obtain::<Arc<AppState>>().unwrap();
    
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

    let query = "SELECT api_code FROM u_app WHERE id = ?";
    let result = sqlx::query(query)
        .bind(appid)
        .fetch_optional(app_state.get_db())
        .await;

    match result {
        Ok(Some(row)) => {
            if let Some(api_code) = row.get::<Option<String>, _>("api_code") {
                res.render(Json(ApiResponse::success("成功", Some(api_code))));
            } else {
                res.render(Json(ApiResponse::success("成功", Some(""))));
            }
        }
        Ok(None) => {
            res.render(Json(ApiResponse::<()>::error("应用不存在", 201)));
        }
        Err(e) => {
            tracing::error!("获取用户API代码失败: {}", e);
            res.render(Json(ApiResponse::<()>::error("获取用户API代码失败", 201)));
        }
    }
}

#[derive(Debug, Deserialize)]
struct EditUserApiCodeRequest {
    api_code: String,
}

#[handler]
pub async fn edit_user_api_code(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let app_state = depot.obtain::<Arc<AppState>>().unwrap();
    
    let edit_req = match req.parse_json::<EditUserApiCodeRequest>().await {
        Ok(data) => data,
        Err(_) => {
            res.render(Json(ApiResponse::<()>::error("参数解析失败", 201)));
            return;
        }
    };

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

    let result = sqlx::query("UPDATE u_app SET api_code = ? WHERE id = ?")
        .bind(&edit_req.api_code)
        .bind(appid)
        .execute(app_state.get_db())
        .await;

    match result {
        Ok(r) => {
            if r.rows_affected() > 0 {
                res.render(Json(ApiResponse::success_msg("编辑用户API代码成功")));
            } else {
                res.render(Json(ApiResponse::<()>::error("编辑用户API代码失败", 201)));
            }
        }
        Err(e) => {
            tracing::error!("编辑用户API代码失败: {}", e);
            res.render(Json(ApiResponse::<()>::error("编辑用户API代码失败", 201)));
        }
    }
}

#[handler]
pub async fn switch_user_api_code(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let app_state = depot.obtain::<Arc<AppState>>().unwrap();
    
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

    // 切换到默认代码
    let result = sqlx::query("UPDATE u_app SET api_code = '' WHERE id = ?")
        .bind(appid)
        .execute(app_state.get_db())
        .await;

    match result {
        Ok(r) => {
            if r.rows_affected() > 0 {
                res.render(Json(ApiResponse::success_msg("切换用户API代码成功")));
            } else {
                res.render(Json(ApiResponse::<()>::error("切换用户API代码失败", 201)));
            }
        }
        Err(e) => {
            tracing::error!("切换用户API代码失败: {}", e);
            res.render(Json(ApiResponse::<()>::error("切换用户API代码失败", 201)));
        }
    }
}