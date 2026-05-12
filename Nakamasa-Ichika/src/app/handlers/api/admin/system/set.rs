//! Admin Set controller
//! 管理员设置控制器

use salvo::prelude::*;

use crate::app::utils::response::ApiResponse;

#[handler]
pub async fn get_list(_req: &mut Request, _depot: &mut Depot, res: &mut Response) {
    let settings = serde_json::json!({
        "app_url": "http://localhost:8080",
        "app_adm_log": "on",
        "app_user_log": "on",
        "user_upfile_size": 10485760,
        "api_run_cost": "on",
        "api_out_type": "json"
    });
    res.render(Json(ApiResponse::success("成功", Some(settings))));
}

#[handler]
pub async fn edit(_req: &mut Request, _depot: &mut Depot, res: &mut Response) {
    res.render(Json(ApiResponse::success_msg("编辑成功")));
}