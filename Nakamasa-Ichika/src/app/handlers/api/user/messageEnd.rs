//! 结束留言
//! 
//! 功能说明：
//! 用户主动结束留言工单，关闭后会话结束。

use salvo::prelude::*;
use std::sync::Arc;

use crate::core::AppState;
use crate::app::utils::response::SignedApiResponse;
use crate::app::utils::validator::Validator;
use crate::app::models::requests::MessageEndRequest;
use crate::app::middleware::user_auth::UserInfo;
use crate::app::middleware::app_context::AppInfo;

#[handler]
pub async fn message_end(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let app_state = depot.obtain::<Arc<AppState>>().unwrap();
    
    // 获取 app_key（零拷贝）
    let app_key = depot.get::<AppInfo>("app_info").map(|i| i.app_key.as_str()).unwrap_or("");
    
    let end_req = match req.parse_json::<MessageEndRequest>().await {
        Ok(data) => data,
        Err(_) => {
            res.render(Json(SignedApiResponse::<()>::error("参数解析失败", 201, app_key)));
            return;
        }
    };

    // 验证参数
    let mut validator = Validator::new();
    validator.wordnum("token", &end_req.token, 32, 32)
        .int("mid", end_req.mid, 1, 11);
    
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

    // 更新留言状态为已结束
    let result = sqlx::query(
        "UPDATE u_message SET state = 2 WHERE id = ? AND uid = ? AND appid = ?"
    )
    .bind(end_req.mid).bind(uid).bind(appid)
    .execute(app_state.get_db())
    .await;

    match result {
        Ok(r) => {
            if r.rows_affected() > 0 {
                res.render(Json(SignedApiResponse::success(app_key, None::<()>)));
            } else {
                res.render(Json(SignedApiResponse::<()>::error("操作失败", 201, app_key)));
            }
        }
        Err(e) => {
            tracing::error!("操作失败: {}", e);
            res.render(Json(SignedApiResponse::<()>::error("操作失败", 201, app_key)));
        }
    }
}
