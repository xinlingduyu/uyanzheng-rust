//! 设置账号
//! 
//! 功能说明：
//! 用户设置自定义账号，设置后可用账号+密码登录。
//! 账号必须以字母开头，5-12位字母数字。
//!
//! 处理流程：
//! 1. 验证token和账号参数
//! 2. 检查账号是否已被使用
//! 3. 更新用户acctno字段
//! 4. 返回成功

use salvo::prelude::*;
use std::sync::Arc;
use chrono::Utc;

use crate::core::AppState;
use crate::core::middleware::get_client_ip;
use crate::app::utils::response::SignedApiResponse;
use crate::app::utils::validator::Validator;
use crate::app::models::requests::SetAcctnoRequest;
use crate::app::middleware::user_auth::UserInfo;
use crate::app::middleware::app_context::AppInfo;

#[handler]
pub async fn set_acctno(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let app_state = depot.obtain::<Arc<AppState>>().unwrap();
    
    // 获取 app_key 和 app_type（零拷贝）
    let (app_key, app_type) = match depot.get::<AppInfo>("app_info") {
        Ok(info) => (info.app_key.as_str(), info.app_type.as_str()),
        Err(_) => {
            res.render(Json(SignedApiResponse::<()>::error("应用信息不存在", 201, "")));
            return;
        }
    };
    
    let set_req = match req.parse_json::<SetAcctnoRequest>().await {
        Ok(data) => data,
        Err(_) => {
            res.render(Json(SignedApiResponse::<()>::error("参数解析失败", 201, app_key)));
            return;
        }
    };

    // PHP: 'token' => ['wordnum','32,32','TOKEN有误'],
    // PHP: 'acctno' => ['wordnum','5,12','自定义账号有误，必须以字母开头5~12位']
    let mut validator = Validator::new();
    validator
        .wordnum("token", &set_req.token, 32, 32)
        .wordnum("acctno", &set_req.acctno, 5, 12);
    
    if let Err(msg) = validator.validate() {
        res.render(Json(SignedApiResponse::<()>::error(msg, 201, app_key)));
        return;
    }

    // PHP: if($this->app['app_type'] != 'user')$this->out->e(115);
    // 只支持用户版应用
    if app_type != "user" {
        res.render(Json(SignedApiResponse::<()>::error("当前应用不支持调用该接口", 115, app_key)));
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
    let user_type = user_info.user_type.as_str();
    let current_time = Utc::now().timestamp();
    let ip = get_client_ip(req);

    // PHP: if(!empty($this->user['acctno']))$this->out->e(123);
    // 检查用户是否已设置账号
    if user_info.acctno.is_some() && !user_info.acctno.as_ref().unwrap().is_empty() {
        res.render(Json(SignedApiResponse::<()>::error("已设置账号", 123, app_key)));
        return;
    }

    // PHP: $Anores = $this->db->where('(phone = ? or acctno = ?) and appid = ?',[$_POST['acctno'],$_POST['acctno'],$this->app['id']])->fetch('id');
    // PHP: if($Anores)$this->out->e(120);
    // 检查账号是否已被使用（同时检查手机号）
    let acctno_check = sqlx::query_as::<_, (i64,)>(
        "SELECT id FROM u_user WHERE (phone = ? OR acctno = ?) AND appid = ?"
    )
    .bind(&set_req.acctno)
    .bind(&set_req.acctno)
    .bind(appid)
    .fetch_optional(app_state.get_db())
    .await;

    if let Ok(Some(_)) = acctno_check {
        res.render(Json(SignedApiResponse::<()>::error("账号已存在", 120, app_key)));
        return;
    }

    // PHP: $res = $this->db->where('id = ?',[$this->user['id']])->update(['acctno'=>$_POST['acctno']]);
    // 更新账号
    let result = sqlx::query(
        "UPDATE u_user SET acctno = ? WHERE id = ? AND appid = ?"
    )
    .bind(&set_req.acctno)
    .bind(uid)
    .bind(appid)
    .execute(app_state.get_db())
    .await;

    match result {
        Ok(r) => {
            if r.rows_affected() > 0 {
                // PHP: $this->log->u($this->app['app_type'],$this->user['id'])->add($res);
                // 记录日志
                let _ = sqlx::query(
                    "INSERT INTO u_logs (ug, uid, type, state, time, ip, appid) VALUES (?, ?, ?, ?, ?, ?, ?)"
                )
                .bind(user_type)
                .bind(uid)
                .bind("setAcctno")
                .bind(true)
                .bind(current_time)
                .bind(ip)
                .bind(appid)
                .execute(app_state.get_db())
                .await;

                // PHP: $this->out->e(200,"设置成功");
                res.render(Json(SignedApiResponse::success(app_key, None::<()>)));
            } else {
                // PHP: if(!$res)$this->out->e(201,"设置失败");
                res.render(Json(SignedApiResponse::<()>::error("设置失败", 201, app_key)));
            }
        }
        Err(e) => {
            tracing::error!("设置账号失败: {}", e);
            res.render(Json(SignedApiResponse::<()>::error("设置失败", 201, app_key)));
        }
    }
}
