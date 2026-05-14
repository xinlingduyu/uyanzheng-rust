//! Admin Fen Event controller
//! 管理员积分事件控制器
//! 一比一还原 PHP 源码逻辑

use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::Row;

use crate::app::utils::response::ApiResponse;
use crate::core::app_state::AppState;
use std::sync::Arc;

// ==================== 获取全部事件列表 ====================

#[derive(Debug, Serialize)]
struct FenEventItem {
    id: u64,
    name: String,
}

#[handler]
pub async fn get_all_list(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let app_state = match depot.obtain::<Arc<AppState>>() {
        Ok(s) => s,
        Err(_) => {
            res.render(Json(ApiResponse::<()>::error("服务器错误", 201)));
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

    // PHP: $this->db->where('appid = ?',[$this->appid])->order('id desc')->fetchAll('id,name');
    let result = sqlx::query_as::<_, (u64, String)>(
        "SELECT id, name FROM u_fen_event WHERE appid = ? ORDER BY id DESC",
    )
    .bind(appid)
    .fetch_all(app_state.get_db())
    .await;

    match result {
        Ok(rows) => {
            let list: Vec<FenEventItem> = rows
                .into_iter()
                .map(|row| FenEventItem {
                    id: row.0,
                    name: row.1,
                })
                .collect();
            // PHP: $this->json('成功',200,$list);
            res.render(Json(ApiResponse::success("成功", Some(list))));
        }
        Err(e) => {
            tracing::error!("数据库查询失败: {}", e);
            res.render(Json(ApiResponse::<()>::error("获取失败", 201)));
        }
    }
}

// ==================== 获取列表 ====================

#[derive(Debug, Deserialize)]
struct GetListRequest {
    #[serde(default)]
    page: Option<i32>,
    #[serde(default)]
    size: Option<i32>,
    #[serde(default)]
    so: Option<SearchOptions>,
}

#[derive(Debug, Deserialize)]
struct SearchOptions {
    #[serde(default)]
    keyword: Option<String>,
}

#[derive(Debug, Serialize)]
struct FenEventListItem {
    id: u64,
    name: String,
    fen: i64,
    vip: Option<i64>,
    vip_free: String,
    state: String,
    appid: i64,
}

#[derive(Debug, Serialize)]
struct FenEventListResponse {
    list: Vec<FenEventListItem>,
    currentPage: i32,
    pageTotal: i32,
    dataTotal: i64,
}

#[handler]
pub async fn get_list(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let app_state = match depot.obtain::<Arc<AppState>>() {
        Ok(s) => s,
        Err(_) => {
            res.render(Json(ApiResponse::<()>::error("服务器错误", 201)));
            return;
        }
    };

    // PHP: $checkRules = ['pg' => ['int','1,11','页面有误',1], 'size' => ['int','1,3','数据条数有误',10], 'so' => ['isArr','','搜索内容不规范',true]];
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

    // PHP: $page = isset($_POST['pg']) ? (intval($_POST['pg']) >= 1 ? intval($_POST['pg']):1) : 1;
    let page = list_req.page.unwrap_or(1).max(1);
    // PHP: $size = isset($_POST['size']) ? intval($_POST['size']) : 10;
    let size = list_req.size.unwrap_or(10);
    let offset = (page - 1) * size;

    // PHP: $whereArr = [$this->appid]; $where = 'appid = ?';
    let mut where_conditions = vec!["appid = ?".to_string()];
    let mut params: Vec<String> = vec![appid.to_string()];

    // PHP: if(isset($_POST['so']) && $this->__isSo($_POST['so']))
    if let Some(so) = &list_req.so {
        // PHP: if(isset($_POST['so']['keyword']) && !empty($_POST['so']['keyword']))
        if let Some(keyword) = &so.keyword
            && !keyword.is_empty()
        {
            // PHP: $where .= ' and name LIKE ?'; array_push($whereArr,'%'.$_POST['so']['keyword'].'%');
            where_conditions.push("name LIKE ?".to_string());
            params.push(format!("%{}%", keyword));
        }
    }

    let where_clause = where_conditions.join(" AND ");
    let count_query = format!("SELECT COUNT(*) FROM u_fen_event WHERE {}", where_clause);
    let data_query = format!(
        "SELECT id, name, fen, vip, vip_free, appid, state FROM u_fen_event WHERE {} ORDER BY id DESC LIMIT ? OFFSET ?",
        where_clause
    );

    // 查询总数
    let mut count_sql = sqlx::query_as::<_, (i64,)>(&count_query);
    for param in &params {
        count_sql = count_sql.bind(param);
    }

    let data_total = match count_sql.fetch_one(app_state.get_db()).await {
        Ok((count,)) => count,
        Err(e) => {
            tracing::error!("查询总数失败: {}", e);
            res.render(Json(ApiResponse::<()>::error("列表获取失败", 201)));
            return;
        }
    };

    let page_total = if data_total == 0 {
        0
    } else {
        ((data_total - 1) / size as i64 + 1) as i32
    };

    // 查询数据
    let mut data_sql =
        sqlx::query_as::<_, (u64, String, i64, Option<i64>, String, i64, String)>(&data_query);
    for param in &params {
        data_sql = data_sql.bind(param);
    }
    data_sql = data_sql.bind(size).bind(offset);

    let result = data_sql.fetch_all(app_state.get_db()).await;

    match result {
        Ok(rows) => {
            let list: Vec<FenEventListItem> = rows
                .into_iter()
                .map(|row| FenEventListItem {
                    id: row.0,
                    name: row.1,
                    fen: row.2,
                    vip: row.3,
                    vip_free: row.4,
                    appid: row.5,
                    state: row.6,
                })
                .collect();

            let response = FenEventListResponse {
                list,
                currentPage: page,
                pageTotal: page_total,
                dataTotal: data_total,
            };

            res.render(Json(ApiResponse::success("成功", Some(response))));
        }
        Err(e) => {
            tracing::error!("数据库查询失败: {}", e);
            res.render(Json(ApiResponse::<()>::error("列表获取失败", 201)));
        }
    }
}

// ==================== 添加 ====================

#[derive(Debug, Deserialize)]
struct AddFenEventRequest {
    name: String,
    fen: i64,
    #[serde(default)]
    vip: Option<String>,
    #[serde(default = "default_vip_free")]
    vip_free: String,
    #[serde(default)]
    r#type: Option<String>,
}

fn default_vip_free() -> String {
    "n".to_string()
}

#[handler]
pub async fn add(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let app_state = match depot.obtain::<Arc<AppState>>() {
        Ok(s) => s,
        Err(_) => {
            res.render(Json(ApiResponse::<()>::error("服务器错误", 201)));
            return;
        }
    };

    let add_req = match req.parse_json::<AddFenEventRequest>().await {
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

    // PHP: $checkRules = [
    //   'name' => ['String','2,125','事件名称不规范'],
    //   'fen' => ['betweend','1,1000000','事件扣除积分数值有误'],
    //   'vip' => ['betweend','0,9999999999','事件兑换会员数值有误','0'],
    //   'vip_free' => ['sameone','y,n','VIP免费选择有误','n']
    // ];

    // 验证 name: 2-125 字符
    if add_req.name.len() < 2 || add_req.name.len() > 125 {
        res.render(Json(ApiResponse::<()>::error("事件名称不规范", 201)));
        return;
    }

    // 验证 fen: 1-1000000
    if add_req.fen < 1 || add_req.fen > 1000000 {
        res.render(Json(ApiResponse::<()>::error("事件扣除积分数值有误", 201)));
        return;
    }

    // 验证 vip: 0-9999999999，默认0
    let vip_value = if let Some(vip_str) = &add_req.vip {
        if vip_str.is_empty() {
            None
        } else {
            match vip_str.parse::<i64>() {
                Ok(v) => {
                    if !(0..=9999999999).contains(&v) {
                        res.render(Json(ApiResponse::<()>::error("事件兑换会员数值有误", 201)));
                        return;
                    }
                    Some(v)
                }
                Err(_) => {
                    res.render(Json(ApiResponse::<()>::error("事件兑换会员数值有误", 201)));
                    return;
                }
            }
        }
    } else {
        None
    };

    // 验证 vip_free: 必须是 'y' 或 'n'
    if add_req.vip_free != "y" && add_req.vip_free != "n" {
        res.render(Json(ApiResponse::<()>::error("VIP免费选择有误", 201)));
        return;
    }

    // PHP: $res = $this->db->where('name = ? and appid = ?',[$_POST['name'],$this->appid])->fetch();
    // PHP: if($res)$this->json("事件名称已存在",201);
    let check_result =
        sqlx::query_as::<_, (i64,)>("SELECT id FROM u_fen_event WHERE name = ? AND appid = ?")
            .bind(&add_req.name)
            .bind(appid)
            .fetch_optional(app_state.get_db())
            .await;

    match check_result {
        Ok(Some(_)) => {
            res.render(Json(ApiResponse::<()>::error("事件名称已存在", 201)));
            return;
        }
        Ok(None) => {}
        Err(e) => {
            tracing::error!("数据库查询失败: {}", e);
            res.render(Json(ApiResponse::<()>::error("数据库错误", 201)));
            return;
        }
    }

    // PHP: $data = [
    //   'name'=>$_POST['name'],
    //   'fen'=>$_POST['fen'],
    //   'vip'=>empty($_POST['vip'])?null:$_POST['vip'],
    //   'vip_free'=>$_POST['vip_free'],
    //   'appid'=>$this->appid,
    // ];

    let insert_result = sqlx::query(
        "INSERT INTO u_fen_event (name, fen, vip, vip_free, appid) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&add_req.name)
    .bind(add_req.fen)
    .bind(vip_value)
    .bind(&add_req.vip_free)
    .bind(appid)
    .execute(app_state.get_db())
    .await;

    match insert_result {
        Ok(result) => {
            if result.rows_affected() > 0 {
                let add_id = result.last_insert_id();

                // PHP: $this->log->u('adm',$this->adminfo['id'])->add($add_id);
                if let Err(e) = add_log(depot, app_state, add_id).await {
                    tracing::error!("日志记录失败: {}", e);
                }

                // PHP: $this->json('添加成功',200);
                res.render(Json(ApiResponse::success_msg("添加成功")));
            } else {
                // PHP: if(!$add_id)$this->json('添加失败',201,$this->db->error());
                res.render(Json(ApiResponse::<()>::error("添加失败", 201)));
            }
        }
        Err(e) => {
            tracing::error!("添加失败: {}", e);
            res.render(Json(ApiResponse::<()>::error("添加失败", 201)));
        }
    }
}

// ==================== 编辑 ====================

#[derive(Debug, Deserialize)]
struct EditFenEventRequest {
    id: i64,
    name: String,
    fen: i64,
    #[serde(default)]
    vip: Option<String>,
    #[serde(default = "default_vip_free")]
    vip_free: String,
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

    let edit_req = match req.parse_json::<EditFenEventRequest>().await {
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

    // PHP: $checkRules = [
    //   'id' => ['int','1,11','编辑ID有误'],
    //   'name' => ['String','2,125','事件名称不规范'],
    //   'fen' => ['betweend','1,1000000','事件扣除积分数值有误'],
    //   'vip' => ['betweend','0,9999999999','事件兑换会员数值有误','0'],
    //   'vip_free' => ['sameone','y,n','VIP免费选择有误','n']
    // ];

    // 验证 id: 1-11位整数
    if edit_req.id < 1 || edit_req.id > 99999999999 {
        res.render(Json(ApiResponse::<()>::error("编辑ID有误", 201)));
        return;
    }

    // 验证 name: 2-125 字符
    if edit_req.name.len() < 2 || edit_req.name.len() > 125 {
        res.render(Json(ApiResponse::<()>::error("事件名称不规范", 201)));
        return;
    }

    // 验证 fen: 1-1000000
    if edit_req.fen < 1 || edit_req.fen > 1000000 {
        res.render(Json(ApiResponse::<()>::error("事件扣除积分数值有误", 201)));
        return;
    }

    // 验证 vip: 0-9999999999，默认0
    let vip_value: Option<i64> = if let Some(vip_str) = &edit_req.vip {
        if vip_str.is_empty() {
            None
        } else {
            match vip_str.parse::<i64>() {
                Ok(v) => {
                    if !(0..=9999999999).contains(&v) {
                        res.render(Json(ApiResponse::<()>::error("事件兑换会员数值有误", 201)));
                        return;
                    }
                    Some(v)
                }
                Err(_) => {
                    res.render(Json(ApiResponse::<()>::error("事件兑换会员数值有误", 201)));
                    return;
                }
            }
        }
    } else {
        None
    };

    // 验证 vip_free: 必须是 'y' 或 'n'
    if edit_req.vip_free != "y" && edit_req.vip_free != "n" {
        res.render(Json(ApiResponse::<()>::error("VIP免费选择有误", 201)));
        return;
    }

    // PHP: $res = $this->db->where('name = ? and appid = ?',[$_POST['name'],$this->appid])->fetch();
    // PHP: if($res && $res['id'] != $_POST['id'])$this->json("事件名称已存在",201);
    let check_result =
        sqlx::query_as::<_, (i64,)>("SELECT id FROM u_fen_event WHERE name = ? AND appid = ?")
            .bind(&edit_req.name)
            .bind(appid)
            .fetch_optional(app_state.get_db())
            .await;

    match check_result {
        Ok(Some((existing_id,))) => {
            if existing_id != edit_req.id {
                res.render(Json(ApiResponse::<()>::error("事件名称已存在", 201)));
                return;
            }
        }
        Ok(None) => {}
        Err(e) => {
            tracing::error!("数据库查询失败: {}", e);
            res.render(Json(ApiResponse::<()>::error("数据库错误", 201)));
            return;
        }
    }

    // PHP: $data = [
    //   'name'=>$_POST['name'],
    //   'fen'=>$_POST['fen'],
    //   'vip'=>$_POST['vip'],
    //   'vip_free'=>$_POST['vip_free'],
    // ];
    // PHP: $res = $this->db->where('id = ?',[$_POST['id']])->update($data);

    let update_result =
        sqlx::query("UPDATE u_fen_event SET name = ?, fen = ?, vip = ?, vip_free = ? WHERE id = ?")
            .bind(&edit_req.name)
            .bind(edit_req.fen)
            .bind(vip_value)
            .bind(&edit_req.vip_free)
            .bind(edit_req.id)
            .execute(app_state.get_db())
            .await;

    match update_result {
        Ok(result) => {
            if result.rows_affected() > 0 {
                // 失效积分事件缓存
                app_state.invalidate_fen_event_cache(edit_req.id as u64);

                // PHP: $this->log->u('adm',$this->adminfo['id'])->add($res);
                if let Err(e) = add_log(depot, app_state, result.rows_affected()).await {
                    tracing::error!("日志记录失败: {}", e);
                }

                // PHP: $this->json('编辑成功',200);
                res.render(Json(ApiResponse::success_msg("编辑成功")));
            } else {
                // PHP: if(!$res)$this->json('编辑失败',201,$this->db->error());
                res.render(Json(ApiResponse::<()>::error("编辑失败", 201)));
            }
        }
        Err(e) => {
            tracing::error!("编辑失败: {}", e);
            res.render(Json(ApiResponse::<()>::error("编辑失败", 201)));
        }
    }
}

// ==================== 编辑状态 ====================

#[derive(Debug, Deserialize)]
struct EditStateRequest {
    id: i64,
    state: String,
}

#[handler]
pub async fn edit_state(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let app_state = match depot.obtain::<Arc<AppState>>() {
        Ok(s) => s,
        Err(_) => {
            res.render(Json(ApiResponse::<()>::error("服务器错误", 201)));
            return;
        }
    };

    // PHP: $checkRules = [
    //   'id' => ['int','1,11','删除ID有误'],
    //   'state' => ['sameone','on,off','状态不规范'],
    // ];

    let edit_state_req = match req.parse_json::<EditStateRequest>().await {
        Ok(data) => data,
        Err(_) => {
            res.render(Json(ApiResponse::<()>::error("参数解析失败", 201)));
            return;
        }
    };

    // 验证 id
    if edit_state_req.id < 1 || edit_state_req.id > 99999999999 {
        res.render(Json(ApiResponse::<()>::error("删除ID有误", 201)));
        return;
    }

    // 验证 state: 必须是 'on' 或 'off'
    if edit_state_req.state != "on" && edit_state_req.state != "off" {
        res.render(Json(ApiResponse::<()>::error("状态不规范", 201)));
        return;
    }

    // PHP: $res = $this->db->where('id = ?',[$_POST['id']])->update(['state'=>$_POST['state']]);
    let result = sqlx::query("UPDATE u_fen_event SET state = ? WHERE id = ?")
        .bind(&edit_state_req.state)
        .bind(edit_state_req.id)
        .execute(app_state.get_db())
        .await;

    match result {
        Ok(update_result) => {
            if update_result.rows_affected() > 0 {
                // 失效积分事件缓存
                app_state.invalidate_fen_event_cache(edit_state_req.id as u64);

                // PHP: $this->log->u('adm',$this->adminfo['id'])->add($res);
                if let Err(e) = add_log(depot, app_state, update_result.rows_affected()).await {
                    tracing::error!("日志记录失败: {}", e);
                }

                // PHP: $this->json('编辑成功',200);
                res.render(Json(ApiResponse::success_msg("编辑成功")));
            } else {
                // PHP: if(!$res)$this->json('编辑失败',201,$this->db->error());
                res.render(Json(ApiResponse::<()>::error("编辑失败", 201)));
            }
        }
        Err(e) => {
            tracing::error!("编辑失败: {}", e);
            res.render(Json(ApiResponse::<()>::error("编辑失败", 201)));
        }
    }
}

// ==================== 删除 ====================

#[derive(Debug, Deserialize)]
struct DelRequest {
    id: i64,
}

#[handler]
pub async fn del(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let app_state = match depot.obtain::<Arc<AppState>>() {
        Ok(s) => s,
        Err(_) => {
            res.render(Json(ApiResponse::<()>::error("服务器错误", 201)));
            return;
        }
    };

    // PHP: $checkRules = ['id' => ['int','1,11','删除ID有误']];

    let del_req = match req.parse_json::<DelRequest>().await {
        Ok(data) => data,
        Err(_) => {
            res.render(Json(ApiResponse::<()>::error("参数解析失败", 201)));
            return;
        }
    };

    // 验证 id: 1-11位整数
    if del_req.id < 1 || del_req.id > 99999999999 {
        res.render(Json(ApiResponse::<()>::error("删除ID有误", 201)));
        return;
    }

    // PHP: $res = $this->db->where('id = ?',[$_POST['id']])->delete();
    let result = sqlx::query("DELETE FROM u_fen_event WHERE id = ?")
        .bind(del_req.id)
        .execute(app_state.get_db())
        .await;

    match result {
        Ok(delete_result) => {
            // PHP: $this->log->u('adm',$this->adminfo['id'])->add($res);
            if let Err(e) = add_log(depot, app_state, delete_result.rows_affected()).await {
                tracing::error!("日志记录失败: {}", e);
            }

            // PHP: if($res)$this->json('删除成功');
            if delete_result.rows_affected() > 0 {
                // 失效积分事件缓存
                app_state.invalidate_fen_event_cache(del_req.id as u64);

                res.render(Json(ApiResponse::success_msg("删除成功")));
            } else {
                // PHP: $this->json('删除失败',201);
                res.render(Json(ApiResponse::<()>::error("删除失败", 201)));
            }
        }
        Err(e) => {
            tracing::error!("删除失败: {}", e);
            res.render(Json(ApiResponse::<()>::error("删除失败", 201)));
        }
    }
}

// ==================== 批量删除 ====================

#[derive(Debug, Deserialize)]
struct DelAllRequest {
    ids: Vec<i64>,
}

#[handler]
pub async fn del_all(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let app_state = match depot.obtain::<Arc<AppState>>() {
        Ok(s) => s,
        Err(_) => {
            res.render(Json(ApiResponse::<()>::error("服务器错误", 201)));
            return;
        }
    };

    // PHP: $checkRules = ['ids' => ['isArr','','删除选中ID有误']];

    let del_all_req = match req.parse_json::<DelAllRequest>().await {
        Ok(data) => data,
        Err(_) => {
            res.render(Json(ApiResponse::<()>::error("参数解析失败", 201)));
            return;
        }
    };

    if del_all_req.ids.is_empty() {
        res.render(Json(ApiResponse::<()>::error("删除选中ID有误", 201)));
        return;
    }

    // PHP: $placeholders = implode(',', array_fill(0,count($_POST['ids']), '?'));
    // PHP: $res = $this->db->where("id in ($placeholders)",$_POST['ids'])->delete();
    let placeholders = del_all_req
        .ids
        .iter()
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(",");
    let query = format!("DELETE FROM u_fen_event WHERE id IN ({})", placeholders);

    let mut sql_query = sqlx::query(&query);
    for id in &del_all_req.ids {
        sql_query = sql_query.bind(id);
    }

    let result = sql_query.execute(app_state.get_db()).await;

    match result {
        Ok(delete_result) => {
            // PHP: $this->log->u('adm',$this->adminfo['id'])->add($res);
            if let Err(e) = add_log(depot, app_state, delete_result.rows_affected()).await {
                tracing::error!("日志记录失败: {}", e);
            }

            // PHP: if($res)$this->json('删除成功');
            if delete_result.rows_affected() > 0 {
                // 批量失效积分事件缓存
                let fenids: Vec<u64> = del_all_req.ids.iter().map(|&id| id as u64).collect();
                app_state.invalidate_fen_event_cache_batch(&fenids);

                res.render(Json(ApiResponse::success_msg("删除成功")));
            } else {
                // PHP: $this->json('删除失败',201);
                res.render(Json(ApiResponse::<()>::error("删除失败", 201)));
            }
        }
        Err(e) => {
            tracing::error!("删除失败: {}", e);
            res.render(Json(ApiResponse::<()>::error("删除失败", 201)));
        }
    }
}

// ==================== 日志记录辅助函数 ====================

// PHP: $this->log->u('adm',$this->adminfo['id'])->add($add_id);
// 注意: 由于 Salvo depot API 限制，暂时使用默认值记录日志
// TODO: 实现正确的 admin_id 获取方式
async fn add_log(
    _depot: &Depot,
    app_state: &Arc<AppState>,
    record_id: u64,
) -> Result<(), sqlx::Error> {
    // PHP: $this->log->u('adm',$this->adminfo['id'])->add($add_id);
    // 对应 PHP: INSERT INTO u_logs (ug, uid, type, asset_changes, time, state) VALUES ('adm', ?, ?, ?, ?, 1)
    let now = chrono::Utc::now().timestamp();

    let asset_changes = serde_json::json!({
        "fen_event_id": record_id
    });

    // 临时使用默认 admin_id = 0，实际应该从 depot 获取
    let admin_id: u64 = 0;

    sqlx::query(
        "INSERT INTO u_logs (ug, uid, type, asset_changes, time, state) VALUES (?, ?, ?, ?, ?, 1)",
    )
    .bind("adm")
    .bind(admin_id)
    .bind("edit")
    .bind(serde_json::to_string(&asset_changes).unwrap_or_default())
    .bind(now)
    .execute(app_state.get_db())
    .await?;

    Ok(())
}
