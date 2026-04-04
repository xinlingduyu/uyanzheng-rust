//! 留言记录列表
//! 
//! 功能说明：
//! 获取用户的留言工单列表，支持分页。

use salvo::prelude::*;
use std::sync::Arc;

use crate::core::AppState;
use crate::app::utils::response::SignedApiResponse;
use crate::app::utils::validator::Validator;
use crate::app::models::requests::MessageListRequest;
use crate::app::models::responses::{MessageItem, MessageListResponse};
use crate::app::middleware::user_auth::UserInfo;
use crate::app::middleware::app_context::AppInfo;

const PAGE_SIZE: u32 = 10;

#[handler]
pub async fn message_list(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let app_state = depot.obtain::<Arc<AppState>>().unwrap();
    
    // 获取 app_key（零拷贝）
    let app_key = depot.get::<AppInfo>("app_info").map(|i| i.app_key.as_str()).unwrap_or("");
    
    let list_req = match req.parse_json::<MessageListRequest>().await {
        Ok(data) => data,
        Err(_) => {
            res.render(Json(SignedApiResponse::<()>::error("参数解析失败", 201, app_key)));
            return;
        }
    };

    // 验证参数
    let mut validator = Validator::new();
    validator.wordnum("token", &list_req.token, 32, 32);
    
    if let Err(msg) = validator.validate() {
        res.render(Json(SignedApiResponse::<()>::error(msg, 201, app_key)));
        return;
    }

    // 从 depot 获取用户信息（避免 clone）
    let user_info = match depot.get::<UserInfo>("user_info") {
        Ok(info) => info,
        Err(_) => {
            res.render(Json(SignedApiResponse::<()>::error("未授权", 201, app_key)));
            return;
        }
    };

    let uid = user_info.uid;
    let appid = user_info.appid;

    // 页码处理
    let page = list_req.pg.unwrap_or(1).max(1);
    let offset = page.saturating_sub(1) * PAGE_SIZE;

    // 查询数据总量
    let count_result = sqlx::query_as::<_, (i64,)>(
        "SELECT COUNT(*) FROM u_message WHERE uid = ? AND reply_id IS NULL AND appid = ?"
    )
    .bind(uid).bind(appid)
    .fetch_one(app_state.get_db())
    .await;

    let data_total = match count_result {
        Ok(row) => row.0 as u32,
        Err(e) => {
            tracing::error!("获取留言总数失败: {}", e);
            res.render(Json(SignedApiResponse::<()>::error("获取失败", 201, app_key)));
            return;
        }
    };

    // 计算总页数
    let page_total = data_total.div_ceil(PAGE_SIZE);

    // 查询列表数据
    let result = sqlx::query_as::<_, (i64, String, i64, i64, i32)>(
        "SELECT id, title, time, last_time, state FROM u_message WHERE uid = ? AND reply_id IS NULL AND appid = ? ORDER BY id DESC LIMIT ? OFFSET ?"
    )
    .bind(uid).bind(appid).bind(PAGE_SIZE).bind(offset)
    .fetch_all(app_state.get_db())
    .await;

    match result {
        Ok(rows) => {
            let msg_list: Vec<MessageItem> = rows.into_iter().map(|(id, title, time, last_time, state)| {
                MessageItem { id, title, time, last_time, state }
            }).collect();
            
            let response = MessageListResponse {
                currentPage: page,
                dataTotal: data_total,
                list: msg_list,
                pageTotal: page_total,
            };
            
            res.render(Json(SignedApiResponse::success(app_key, Some(response))));
        }
        Err(e) => {
            tracing::error!("获取留言列表失败: {}", e);
            res.render(Json(SignedApiResponse::<()>::error("获取失败", 201, app_key)));
        }
    }
}