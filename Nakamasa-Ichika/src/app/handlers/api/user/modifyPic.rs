//! 修改头像
//! 
//! 功能说明：
//! 已登录用户修改自己的头像。
//! 接收已上传文件的URL路径，更新数据库中的avatars字段。
//!
//! 处理流程：
//! 1. 验证token和file参数
//! 2. 检查应用类型为用户版
//! 3. 更新用户avatars字段
//! 4. 返回成功

use salvo::prelude::*;
use std::sync::Arc;
use chrono::Utc;

use crate::core::AppState;
use crate::core::middleware::get_client_ip;
use crate::app::utils::response::SignedApiResponse;
use crate::app::utils::validator::Validator;
use crate::app::models::requests::ModifyPicRequest;
use crate::app::middleware::user_auth::UserInfo;
use crate::app::middleware::app_context::AppInfo;

#[handler]
pub async fn modify_pic(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let app_state = depot.obtain::<Arc<AppState>>().unwrap();
    
    // 获取 app_key 用于签名
    let (app_key, app_type) = match depot.get::<AppInfo>("app_info") {
        Ok(info) => (info.app_key.as_str(), info.app_type.as_str()),
        Err(_) => {
            res.render(Json(SignedApiResponse::<()>::error("应用信息不存在", 201, "")));
            return;
        }
    };
    
    // PHP: if($this->app['app_type'] != 'user')$this->out->e(115);
    // 检查应用类型
    if app_type != "user" {
        res.render(Json(SignedApiResponse::<()>::error("当前应用不支持调用该接口", 115, app_key)));
        return;
    }

    let modify_req = match req.parse_json::<ModifyPicRequest>().await {
        Ok(data) => data,
        Err(_) => {
            res.render(Json(SignedApiResponse::<()>::error("参数解析失败", 201, app_key)));
            return;
        }
    };

    // PHP: $checkRules = ['token' => ['wordnum','32,32','TOKEN有误'], 'file' => ['wordnum','32,32','file有误']]
    // 验证参数
    let mut validator = Validator::new();
    validator.wordnum("token", &modify_req.token, 32, 32);
    // file 是URL路径，验证长度范围
    validator.string("file", &modify_req.file, 1, 255);
    
    if let Err(msg) = validator.validate() {
        res.render(Json(SignedApiResponse::<()>::error(msg, 201, app_key)));
        return;
    }

    // 从 depot 获取用户信息（由 UserAuth 中间件提供）
    let user_info = match depot.get::<UserInfo>("user_info") {
        Ok(info) => info,
        Err(_) => {
            res.render(Json(SignedApiResponse::<()>::error("未授权", 201, app_key)));
            return;
        }
    };

    let (uid, appid) = (user_info.uid, user_info.appid);
    let current_time = Utc::now().timestamp();
    let ip = get_client_ip(req);

    // PHP: if(isset($post["file"]))
    // 准备头像URL - 确保以 / 开头
    let avatars_owned;
    let avatars: &str = if modify_req.file.starts_with('/') {
        &modify_req.file
    } else {
        avatars_owned = format!("/{}", modify_req.file);
        &avatars_owned
    };

    // PHP: $res = $this->db->where('id = ? and appid = ?',[$this->user['id'],$this->app['id']])->update(['avatars'=>'/'.$uploadedFile]);
    let result = sqlx::query(
        "UPDATE u_user SET avatars = ? WHERE id = ? AND appid = ?"
    )
    .bind(avatars)
    .bind(uid as i64)
    .bind(appid as i64)
    .execute(app_state.get_db())
    .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            // PHP: $this->log->u('user',$this->user['id'])->add($res);
            // 记录日志
            let _ = sqlx::query(
                "INSERT INTO u_logs (ug, uid, type, time, ip, ip_address, appid) VALUES (?, ?, ?, ?, ?, NULL, ?)"
            )
            .bind("user")
            .bind(uid as i64)
            .bind("modifyPic")
            .bind(current_time)
            .bind(ip)
            .bind(appid as i64)
            .execute(app_state.get_db())
            .await;

            tracing::info!("用户 {} 修改头像成功: {}", uid, avatars);
            res.render(Json(SignedApiResponse::success(app_key, None::<()>)));
        }
        Ok(_) => {
            res.render(Json(SignedApiResponse::<()>::error("头像修改失败", 201, app_key)));
        }
        Err(e) => {
            tracing::error!("更新头像失败: {}", e);
            res.render(Json(SignedApiResponse::<()>::error("数据库错误", 201, app_key)));
        }
    }
}
