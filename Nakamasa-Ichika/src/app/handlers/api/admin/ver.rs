//! Admin Ver controller
//! 管理员版本控制器

use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::Row;

use crate::app::utils::response::ApiResponse;
use std::sync::Arc;
use crate::core::app_state::AppState;

#[derive(Debug, Serialize)]
struct GroupItem {
    name: Option<String>,
    ver_key: String,
}

#[handler]
pub async fn get_group(req: &mut Request, depot: &mut Depot, res: &mut Response) {
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

    let query = "SELECT DISTINCT name, ver_key FROM u_app_ver WHERE appid = ? GROUP BY name, ver_key";
    
    let result = sqlx::query_as::<_, (Option<String>, String)>(query)
        .bind(appid)
        .fetch_all(app_state.get_db())
        .await;

    match result {
        Ok(rows) => {
            let list: Vec<GroupItem> = rows.into_iter().map(|row| GroupItem {
                name: row.0,
                ver_key: row.1,
            }).collect();

            res.render(Json(ApiResponse::success("成功", Some(list))));
        }
        Err(e) => {
            tracing::error!("数据库查询失败: {}", e);
            res.render(Json(ApiResponse::<()>::error("列表获取失败", 201)));
        }
    }
}

#[derive(Debug, Deserialize)]
struct GetListRequest {
    #[serde(default)]
    pg: Option<u32>,
    #[serde(default)]
    size: Option<u32>,
    #[serde(default)]
    so: Option<SearchOptions>,
}

#[derive(Debug, Deserialize)]
struct SearchOptions {
    #[serde(default)]
    keyword: Option<String>,
}

#[derive(Debug, Serialize)]
struct VerItem {
    id: u64,
    name: Option<String>,
    ver_key: String,
    ver_major: i32,
    ver_minor: i32,
    ver_patch: i32,
    ver_state: String,
    ver_off_msg: Option<String>,
    ver_url: Option<String>,
    ver_content: Option<String>,
    mid: Option<u64>,
    discard: bool,
    appid: u64,
    mi_name: Option<String>,
    mi_type: Option<String>,
}

#[derive(Debug, Serialize)]
struct ListResponse {
    #[serde(rename = "currentPage")]
    current_page: u32,
    #[serde(rename = "dataTotal")]
    data_total: u64,
    list: Vec<VerItem>,
    #[serde(rename = "pageTotal")]
    page_total: u32,
}

#[handler]
pub async fn get_list(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let app_state = depot.obtain::<Arc<AppState>>().unwrap();
    
    let list_req = match req.parse_json::<GetListRequest>().await {
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

    let page = list_req.pg.unwrap_or(1).max(1);
    let page_size = list_req.size.unwrap_or(10).max(1);
    let offset = (page - 1) * page_size;

    // 查询总数
    let count_query = "SELECT COUNT(*) as total FROM u_app_ver WHERE appid = ?";
    let count_result = sqlx::query_as::<_, (i64,)>(count_query)
        .bind(appid)
        .fetch_one(app_state.get_db())
        .await;

    let total = match count_result {
        Ok((count,)) => count as u64,
        Err(_) => {
            res.render(Json(ApiResponse::<()>::error("查询总数失败", 201)));
            return;
        }
    };

    let page_total = if total > 0 {
        ((total as f64) / (page_size as f64)).ceil() as u32
    } else {
        0
    };

    let mut query = "SELECT V.id, V.name, V.ver_key, V.ver_major, V.ver_minor, V.ver_patch, V.ver_state, V.ver_off_msg, V.ver_url, V.ver_content, V.mid, V.discard, V.appid, M.name as mname, M.type as mtype FROM u_app_ver AS V LEFT JOIN u_app_mi AS M ON (V.mid=M.id) WHERE V.appid = ?".to_string();
    let mut params: Vec<String> = vec![appid.to_string()];

    if let Some(ref so) = list_req.so {
        if let Some(ref keyword) = so.keyword {
            if !keyword.is_empty() {
                query = format!("{} AND (V.name LIKE ? OR V.ver_key LIKE ?)", query);
                params.push(format!("%{}%", keyword));
                params.push(format!("%{}%", keyword));
            }
        }
    }

    query = format!("{} ORDER BY V.id DESC LIMIT ? OFFSET ?", query);
    params.push(page_size.to_string());
    params.push(offset.to_string());

    let mut sql_query = sqlx::query(&query);
    for param in &params {
        sql_query = sql_query.bind(param);
    }

    let result = sql_query.fetch_all(app_state.get_db()).await;

    match result {
        Ok(rows) => {
            tracing::debug!("查询到 {} 行数据", rows.len());
            let list: Vec<VerItem> = rows.into_iter().map(|row| VerItem {
                id: row.get("id"),
                name: row.get("name"),
                ver_key: row.get("ver_key"),
                ver_major: row.get("ver_major"),
                ver_minor: row.get("ver_minor"),
                ver_patch: row.get("ver_patch"),
                ver_state: row.get("ver_state"),
                ver_off_msg: row.get("ver_off_msg"),
                ver_url: row.get("ver_url"),
                ver_content: row.get("ver_content"),
                mid: row.get("mid"),
                discard: row.get::<bool, _>("discard"),
                appid: row.get("appid"),
                mi_name: row.get("mname"),
                mi_type: row.get("mtype"),
            }).collect();

            tracing::debug!("序列化后的列表: {:?}", list);
            
            let response = ListResponse {
                current_page: page,
                data_total: total,
                list,
                page_total,
            };
            
            res.render(Json(ApiResponse::success("成功", Some(response))));
        }
        Err(e) => {
            tracing::error!("数据库查询失败: {}", e);
            res.render(Json(ApiResponse::<()>::error("列表获取失败", 201)));
        }
    }
}

#[derive(Debug, Deserialize)]
struct AddRequest {
    #[serde(default)]
    mid: Option<u64>,
    name: String,
    ver_key: String,
    ver_major: i32,
    ver_minor: i32,
    ver_patch: i32,
    #[serde(default)]
    ver_url: Option<String>,
    #[serde(default)]
    ver_content: Option<String>,
    ver_state: String,
    #[serde(default)]
    ver_off_msg: Option<String>,
}

impl AddRequest {
    fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("版本名不能为空".to_string());
        }
        if self.name.len() > 64 {
            return Err("版本名长度不能超过64个字符".to_string());
        }
        if self.ver_key.trim().is_empty() {
            return Err("版本索引不能为空".to_string());
        }
        if self.ver_key.len() > 12 {
            return Err("版本索引长度不能超过12个字符".to_string());
        }
        if self.ver_major < 1 || self.ver_major > 999 {
            return Err("版本主号必须在1-999之间".to_string());
        }
        if self.ver_minor < 0 || self.ver_minor > 999 {
            return Err("版本次号必须在0-999之间".to_string());
        }
        if self.ver_patch < 0 || self.ver_patch > 999 {
            return Err("版本补丁号必须在0-999之间".to_string());
        }
        if !["on", "off"].contains(&self.ver_state.as_str()) {
            return Err("版本状态必须是on或off".to_string());
        }
        if let Some(ref url) = self.ver_url {
            if !url.is_empty() && !url.starts_with("http://") && !url.starts_with("https://") {
                return Err("版本URL必须以http://或https://开头".to_string());
            }
        }
        Ok(())
    }
}

#[handler]
pub async fn add(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let app_state = depot.obtain::<Arc<AppState>>().unwrap();
    
    let add_req = match req.parse_json::<AddRequest>().await {
        Ok(data) => data,
        Err(_) => {
            res.render(Json(ApiResponse::<()>::error("参数解析失败", 201)));
            return;
        }
    };

    // 数据校验
    if let Err(e) = add_req.validate() {
        res.render(Json(ApiResponse::<()>::error(e, 201)));
        return;
    }

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

    // 检查版本是否已存在
    let check_query = "SELECT id FROM u_app_ver WHERE ver_key = ? AND ver_major = ? AND ver_minor = ? AND ver_patch = ? AND appid = ?";
    let check_result = sqlx::query_as::<_, (u64,)>(check_query)
        .bind(&add_req.ver_key)
        .bind(add_req.ver_major)
        .bind(add_req.ver_minor)
        .bind(add_req.ver_patch)
        .bind(appid)
        .fetch_optional(app_state.get_db())
        .await;

    match check_result {
        Ok(Some(_)) => {
            res.render(Json(ApiResponse::<()>::error("版本号已存在", 201)));
            return;
        }
        Ok(None) => {}
        Err(e) => {
            tracing::error!("数据库查询失败: {}", e);
            res.render(Json(ApiResponse::<()>::error("数据库查询失败", 201)));
            return;
        }
    }

    let insert_query = "INSERT INTO u_app_ver (mid, name, ver_key, ver_major, ver_minor, ver_patch, ver_url, ver_content, ver_state, ver_off_msg, appid) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";
    
    let result = sqlx::query(insert_query)
        .bind(add_req.mid)
        .bind(&add_req.name)
        .bind(&add_req.ver_key)
        .bind(add_req.ver_major)
        .bind(add_req.ver_minor)
        .bind(add_req.ver_patch)
        .bind(&add_req.ver_url)
        .bind(&add_req.ver_content)
        .bind(&add_req.ver_state)
        .bind(&add_req.ver_off_msg)
        .bind(appid)
        .execute(app_state.get_db())
        .await;

    match result {
        Ok(_) => {
            res.render(Json(ApiResponse::success_msg("添加成功")));
        }
        Err(e) => {
            tracing::error!("添加失败: {}", e);
            res.render(Json(ApiResponse::<()>::error("添加失败", 201)));
        }
    }
}

#[derive(Debug, Deserialize)]
struct EditRequest {
    id: u64,
    #[serde(default)]
    mid: Option<u64>,
    name: String,
    ver_key: String,
    ver_major: i32,
    ver_minor: i32,
    ver_patch: i32,
    #[serde(default)]
    ver_url: Option<String>,
    #[serde(default)]
    ver_content: Option<String>,
    ver_state: String,
    #[serde(default)]
    ver_off_msg: Option<String>,
}

impl EditRequest {
    fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("版本名不能为空".to_string());
        }
        if self.name.len() > 64 {
            return Err("版本名长度不能超过64个字符".to_string());
        }
        if self.ver_key.trim().is_empty() {
            return Err("版本索引不能为空".to_string());
        }
        if self.ver_key.len() > 12 {
            return Err("版本索引长度不能超过12个字符".to_string());
        }
        if self.ver_major < 1 || self.ver_major > 999 {
            return Err("版本主号必须在1-999之间".to_string());
        }
        if self.ver_minor < 0 || self.ver_minor > 999 {
            return Err("版本次号必须在0-999之间".to_string());
        }
        if self.ver_patch < 0 || self.ver_patch > 999 {
            return Err("版本补丁号必须在0-999之间".to_string());
        }
        if !["on", "off"].contains(&self.ver_state.as_str()) {
            return Err("版本状态必须是on或off".to_string());
        }
        if let Some(ref url) = self.ver_url {
            if !url.is_empty() && !url.starts_with("http://") && !url.starts_with("https://") {
                return Err("版本URL必须以http://或https://开头".to_string());
            }
        }
        Ok(())
    }
}

#[handler]
pub async fn edit(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let app_state = depot.obtain::<Arc<AppState>>().unwrap();
    
    let edit_req = match req.parse_json::<EditRequest>().await {
        Ok(data) => data,
        Err(_) => {
            res.render(Json(ApiResponse::<()>::error("参数解析失败", 201)));
            return;
        }
    };

    // 数据校验
    if let Err(e) = edit_req.validate() {
        res.render(Json(ApiResponse::<()>::error(e, 201)));
        return;
    }

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

    let update_query = "UPDATE u_app_ver SET mid = ?, name = ?, ver_key = ?, ver_major = ?, ver_minor = ?, ver_patch = ?, ver_url = ?, ver_content = ?, ver_state = ?, ver_off_msg = ?, appid = ? WHERE id = ?";
    
    let result = sqlx::query(update_query)
        .bind(edit_req.mid)
        .bind(&edit_req.name)
        .bind(&edit_req.ver_key)
        .bind(edit_req.ver_major)
        .bind(edit_req.ver_minor)
        .bind(edit_req.ver_patch)
        .bind(&edit_req.ver_url)
        .bind(&edit_req.ver_content)
        .bind(&edit_req.ver_state)
        .bind(&edit_req.ver_off_msg)
        .bind(appid)
        .bind(edit_req.id)
        .execute(app_state.get_db())
        .await;

    match result {
        Ok(_) => {
            res.render(Json(ApiResponse::success_msg("编辑成功")));
        }
        Err(e) => {
            tracing::error!("编辑失败: {}", e);
            res.render(Json(ApiResponse::<()>::error("编辑失败", 201)));
        }
    }
}

#[derive(Debug, Deserialize)]
struct DelRequest {
    id: u64,
}

#[handler]
pub async fn del(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let app_state = depot.obtain::<Arc<AppState>>().unwrap();
    
    let del_req = match req.parse_json::<DelRequest>().await {
        Ok(data) => data,
        Err(_) => {
            res.render(Json(ApiResponse::<()>::error("参数解析失败", 201)));
            return;
        }
    };

    let result = sqlx::query("DELETE FROM u_app_ver WHERE id = ?")
        .bind(del_req.id)
        .execute(app_state.get_db())
        .await;

    match result {
        Ok(r) => {
            if r.rows_affected() > 0 {
                res.render(Json(ApiResponse::success_msg("删除成功")));
            } else {
                res.render(Json(ApiResponse::<()>::error("删除失败", 201)));
            }
        }
        Err(e) => {
            tracing::error!("删除失败: {}", e);
            res.render(Json(ApiResponse::<()>::error("删除失败", 201)));
        }
    }
}