//! 解绑手机号
//!
//! 功能说明：
//! 用户解绑已绑定的手机号，需要验证码验证。
//!
//! 处理流程：
//! 1. 验证token、手机号、验证码参数
//! 2. 验证验证码是否正确
//! 3. 验证手机号是否为当前用户绑定
//! 4. 清空用户phone字段
//! 5. 返回成功

use chrono::Utc;
use salvo::prelude::*;
use std::sync::Arc;

use crate::app::middleware::app_context::AppInfo;
use crate::app::middleware::user_auth::UserInfo;
use crate::app::models::requests::RePhoneRequest;
use crate::app::utils::response::{
    SignedApiResponse, render_error, render_success, render_success_msg, render_success_with_msg,
};
use crate::app::utils::validator::Validator;
use crate::core::AppState;
use crate::core::middleware::get_client_ip;

#[handler]
pub async fn re_phone(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let app_state = match depot.obtain::<Arc<AppState>>() {
        Ok(s) => s,
        Err(_) => {
            render_error(res, "服务器错误", 201, "");
            return;
        }
    };

    // 获取应用信息
    let app_info = match depot.get::<AppInfo>("app_info") {
        Ok(info) => info,
        Err(_) => {
            render_error(res, "应用信息不存在", 201, "");
            return;
        }
    };
    let app_key = &app_info.app_key;
    let vc_time = app_info.vc_time;

    let re_req = match req.parse_json::<RePhoneRequest>().await {
        Ok(data) => data,
        Err(_) => {
            render_error(res, "参数解析失败", 201, app_key);
            return;
        }
    };

    // PHP: 'token' => ['wordnum','32,32','TOKEN不规范'],
    // PHP: 'phone' => ['phone','','手机号不规范']
    // PHP: 'code'  => ['int','4,6','验证码填写不规范']
    let mut validator = Validator::new();
    validator
        .wordnum("token", &re_req.token, 32, 32)
        .phone("phone", &re_req.phone)
        .int("code", re_req.code as i64, 4, 6);

    if let Err(msg) = validator.validate() {
        render_error(res, msg, 201, app_key);
        return;
    }

    // 从 depot 获取用户信息（由 UserAuth 中间件提供）
    let user_info = match depot.get::<UserInfo>("user_info") {
        Ok(info) => info,
        Err(_) => {
            render_error(res, "未授权", 201, app_key);
            return;
        }
    };

    let (uid, appid) = (user_info.uid, user_info.appid);
    let user_type = user_info.user_type.as_str();
    let current_time = Utc::now().timestamp();
    let ip = get_client_ip(req);

    // PHP: $dtime = time() - (60*$this->app['vc_time']);
    let dtime = current_time - (vc_time * 60) as i64;

    // PHP: if($this->user['phone'] != $_POST['phone'])$this->out->e(201,'手机号有误');
    // 验证用户当前手机号是否与提交的手机号一致
    let current_phone = user_info.phone.as_deref().unwrap_or("");
    if current_phone != re_req.phone {
        render_error(res, "手机号有误", 201, app_key);
        return;
    }

    // PHP: $vcDB = db('vcode');
    // PHP: $res_code = $vcDB->where('eorp = ? and code = ? and type = ? and usable = ? and time > ? and appid = ?', [$_POST['phone'],$_POST['code'],'rePhone','y',$dtime,$this->app['id']])->update(['usable'=>'n']);
    // PHP: if(!$res_code || $vcDB->rowCount() < 1)$this->out->e(119);
    // 验证验证码并标记为已使用
    let verify_result = sqlx::query(
        "UPDATE u_vcode SET usable = 'n' WHERE eorp = ? AND code = ? AND type = ? AND usable = 'y' AND time > ? AND appid = ?"
    )
    .bind(&re_req.phone)
    .bind(re_req.code)
    .bind("rePhone")
    .bind(dtime)
    .bind(appid)
    .execute(app_state.get_db())
    .await;

    match verify_result {
        Ok(result) => {
            if result.rows_affected() < 1 {
                render_error(res, "验证码不正确", 119, app_key);
                return;
            }
        }
        Err(e) => {
            tracing::error!("验证码验证失败: {}", e);
            render_error(res, "数据库错误", 201, app_key);
            return;
        }
    }

    // PHP: $res = $this->db->where('id = ?',[$this->user['id']])->update(['phone'=>NULL]);
    // 更新手机号为NULL
    let result = sqlx::query("UPDATE u_user SET phone = NULL WHERE id = ? AND appid = ?")
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
                .bind("rePhone")
                .bind(true)
                .bind(current_time)
                .bind(ip)
                .bind(appid)
                .execute(app_state.get_db())
                .await;

                // PHP: $this->out->e(200,"解绑成功");
                render_success(res, app_key, None::<()>, app_info.mi.as_ref());
            } else {
                // PHP: if(!$res)$this->out->e(201,"解绑失败");
                render_error(res, "解绑失败", 201, app_key);
            }
        }
        Err(e) => {
            tracing::error!("解绑手机号失败: {}", e);
            render_error(res, "解绑失败", 201, app_key);
        }
    }
}
