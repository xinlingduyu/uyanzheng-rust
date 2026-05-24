//! Admin Download controller
//! 管理员下载控制器

use salvo::prelude::*;
use serde::Deserialize;

use crate::app::utils::response::ApiResponse;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct KamiRequestParam {
    path: String,
    sign: String,
    time: i64,
}

#[handler]
pub async fn index(req: &mut Request, _depot: &mut Depot, res: &mut Response) {
    let param = req.query::<String>("param").unwrap_or_default();

    if param.is_empty() {
        res.render(Json(ApiResponse::<()>::error("缺少参数", 201)));
        return;
    }

    let author_data = match hex::decode(&param) {
        Ok(data) => match String::from_utf8(data) {
            Ok(s) => s,
            Err(_) => {
                res.render(Json(ApiResponse::<()>::error("参数有误", 201)));
                return;
            }
        },
        Err(_) => {
            res.render(Json(ApiResponse::<()>::error("参数有误", 201)));
            return;
        }
    };

    let kami_param: KamiRequestParam = match serde_json::from_str(&author_data) {
        Ok(data) => data,
        Err(_) => {
            res.render(Json(ApiResponse::<()>::error("参数有误", 201)));
            return;
        }
    };

    let current_time = chrono::Utc::now().timestamp();
    if kami_param.time + 600 < current_time {
        res.render(Json(ApiResponse::<()>::error("参数过期", 201)));
        return;
    }

    res.render(Json(ApiResponse::success_msg("下载功能需要进一步实现")));
}
