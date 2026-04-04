//! 绑定邮箱
//! 
//! 功能说明：
//! 用户绑定邮箱，需要邮箱验证码验证。
//! 绑定后可用邮箱+密码登录。
//!
//! 处理流程：
//! 1. 验证token、邮箱、验证码参数
//! 2. 验证验证码是否正确
//! 3. 检查邮箱是否已被其他用户绑定
//! 4. 更新用户email字段
//! 5. 返回成功

use salvo::prelude::*;
use std::sync::Arc;
use chrono::Utc;

use crate::core::AppState;
use crate::core::middleware::get_client_ip;
use crate::app::utils::response::SignedApiResponse;
use crate::app::utils::validator::Validator;
use crate::app::models::requests::SetEmailRequest;
use crate::app::middleware::user_auth::UserInfo;
use crate::app::middleware::app_context::AppInfo;

#[handler]
pub async fn set_email(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let app_state = depot.obtain::<Arc<AppState>>().unwrap();
    
    // 获取 app_key 和 vc_time（零拷贝）
    let (app_key, vc_time) = match depot.get::<AppInfo>("app_info") {
        Ok(info) => (info.app_key.as_str(), info.vc_time),
        Err(_) => {
            res.render(Json(SignedApiResponse::<()>::error("应用信息不存在", 201, "")));
            return;
        }
    };
    
    let set_req = match req.parse_json::<SetEmailRequest>().await {
        Ok(data) => data,
        Err(_) => {
            res.render(Json(SignedApiResponse::<()>::error("参数解析失败", 201, app_key)));
            return;
        }
    };

    // PHP: 'token' => ['wordnum','32,32','TOKEN有误'],
    // PHP: 'email' => ['email','','邮箱账号有误']
    // PHP: 'code'  => ['int','4,6','验证码填写有误']
    let mut validator = Validator::new();
    validator
        .wordnum("token", &set_req.token, 32, 32)
        .email("email", &set_req.email)
        .int("code", set_req.code as i64, 4, 6);
    
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
    let user_type = user_info.user_type.as_str();
    let current_time = Utc::now().timestamp();
    let ip = get_client_ip(req);

    // PHP: $dtime = time() - (60*$this->app['vc_time']);
    let dtime = current_time - (vc_time * 60) as i64;

    // PHP: if(!empty($this->user['email']))$this->out->e(124);
    // 检查用户是否已绑定邮箱
    if user_info.email.is_some() && !user_info.email.as_ref().unwrap().is_empty() {
        res.render(Json(SignedApiResponse::<()>::error("已绑定邮箱", 124, app_key)));
        return;
    }

    // PHP: $vcDB = db('vcode');
    // PHP: $res_code = $vcDB->where('eorp = ? and code = ? and type = ? and usable = ? and time > ? and appid = ?', [$_POST['email'],$_POST['code'],'ubind','y',$dtime,$this->app['id']])->update(['usable'=>'n']);
    // PHP: if(!$res_code || $vcDB->rowCount() < 1)$this->out->e(119);
    // 验证验证码并标记为已使用
    let verify_result = sqlx::query(
        "UPDATE u_vcode SET usable = 'n' WHERE eorp = ? AND code = ? AND type = ? AND usable = 'y' AND time > ? AND appid = ?"
    )
    .bind(&set_req.email)
    .bind(set_req.code)
    .bind("ubind")
    .bind(dtime)
    .bind(appid)
    .execute(app_state.get_db())
    .await;
    
    match verify_result {
        Ok(result) => {
            if result.rows_affected() < 1 {
                res.render(Json(SignedApiResponse::<()>::error("验证码不正确", 119, app_key)));
                return;
            }
        }
        Err(e) => {
            tracing::error!("验证码验证失败: {}", e);
            res.render(Json(SignedApiResponse::<()>::error("数据库错误", 201, app_key)));
            return;
        }
    }

    // PHP: $emailRes = $this->db->where('email = ? and appid = ?',[$_POST['email'],$this->app['id']])->fetch('id');
    // PHP: if($emailRes)$this->out->e(120,'该邮箱已被绑定');
    // 检查邮箱是否已被其他用户绑定
    let email_check = sqlx::query_as::<_, (i64,)>(
        "SELECT id FROM u_user WHERE email = ? AND appid = ?"
    )
    .bind(&set_req.email)
    .bind(appid)
    .fetch_optional(app_state.get_db())
    .await;

    if let Ok(Some(_)) = email_check {
        res.render(Json(SignedApiResponse::<()>::error("该邮箱已被绑定", 120, app_key)));
        return;
    }

    // PHP: $res = $this->db->where('id = ?',[$this->user['id']])->update(['email'=>$_POST['email']]);
    // 更新邮箱
    let result = sqlx::query(
        "UPDATE u_user SET email = ? WHERE id = ? AND appid = ?"
    )
    .bind(&set_req.email)
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
                .bind("setEmail")
                .bind(true)
                .bind(current_time)
                .bind(ip)
                .bind(appid)
                .execute(app_state.get_db())
                .await;

                // PHP: $this->out->e(200,"绑定成功");
                res.render(Json(SignedApiResponse::success(app_key, None::<()>)));
            } else {
                // PHP: if(!$res)$this->out->e(201,"绑定失败");
                res.render(Json(SignedApiResponse::<()>::error("绑定失败", 201, app_key)));
            }
        }
        Err(e) => {
            tracing::error!("绑定邮箱失败: {}", e);
            res.render(Json(SignedApiResponse::<()>::error("绑定失败", 201, app_key)));
        }
    }
}
