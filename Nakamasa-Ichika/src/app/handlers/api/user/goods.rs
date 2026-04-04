//! 商品列表
//! 
//! 功能说明：
//! 获取应用的商品列表，用于在线充值展示可选商品。
//!
//! 处理流程：
//! 1. 验证应用类型为用户版
//! 2. 分页查询商品表
//! 3. 返回商品名称、价格、时长等信息

use salvo::prelude::*;
use std::sync::Arc;

use crate::core::AppState;
use crate::app::utils::response::SignedApiResponse;
use crate::app::models::requests::GoodsRequest;
use crate::app::models::responses::{GoodsItem, GoodsListResponse};
use crate::app::middleware::app_context::AppInfo;

/// 默认每页数量
const PAGE_SIZE: u32 = 10;

#[handler]
pub async fn goods(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let app_state = depot.obtain::<Arc<AppState>>().unwrap();
    
    // 获取 app_key 和 app_info（零拷贝引用）
    let app_info = match depot.get::<AppInfo>("app_info") {
        Ok(info) => info,
        Err(_) => {
            res.render(Json(SignedApiResponse::<()>::error("应用信息获取失败", 201, "")));
            return;
        }
    };
    let app_key = &app_info.app_key;

    // 一比一还原PHP: if($this->app['app_type'] != 'user')$this->out->e(115);
    if app_info.app_type != "user" {
        res.render(Json(SignedApiResponse::<()>::error("当前应用不支持调用该接口", 115, app_key)));
        return;
    }

    let goods_req = match req.parse_json::<GoodsRequest>().await {
        Ok(data) => data,
        Err(_) => {
            res.render(Json(SignedApiResponse::<()>::error("参数解析失败", 201, app_key)));
            return;
        }
    };

    let appid = app_info.id;

    // 页码处理：默认为1，使用 saturating_sub 避免 underflow
    let page = goods_req.pg.unwrap_or(1).max(1);
    let offset = page.saturating_sub(1) * PAGE_SIZE;

    // 查询数据总量
    let count_result = sqlx::query_as::<_, (i64,)>(
        "SELECT COUNT(*) FROM u_goods WHERE state = 'y' AND appid = ?"
    )
    .bind(appid)
    .fetch_one(app_state.get_db())
    .await;

    let data_total = match count_result {
        Ok(row) => row.0 as u32,
        Err(e) => {
            tracing::error!("获取商品总数失败: {}", e);
            res.render(Json(SignedApiResponse::<()>::error("获取失败", 201, app_key)));
            return;
        }
    };

    // 计算总页数（向上取整）
    let page_total = data_total.div_ceil(PAGE_SIZE);

    // 查询列表数据
    let result = sqlx::query_as::<_, (i64, String, String, i64, String)>(
        "SELECT id, name, type, money, blurb FROM u_goods WHERE state = 'y' AND appid = ? ORDER BY id DESC LIMIT ? OFFSET ?"
    )
    .bind(appid)
    .bind(PAGE_SIZE)
    .bind(offset)
    .fetch_all(app_state.get_db())
    .await;

    match result {
        Ok(rows) => {
            let goods_list: Vec<GoodsItem> = rows.into_iter().map(|(id, name, r#type, money, blurb)| {
                GoodsItem { id, name, r#type, money, blurb }
            }).collect();
            
            let response = GoodsListResponse {
                currentPage: page,
                dataTotal: data_total,
                list: goods_list,
                pageTotal: page_total,
            };
            
            res.render(Json(SignedApiResponse::success(app_key, Some(response))));
        }
        Err(e) => {
            tracing::error!("获取商品列表失败: {}", e);
            res.render(Json(SignedApiResponse::<()>::error("获取失败", 201, app_key)));
        }
    }
}
