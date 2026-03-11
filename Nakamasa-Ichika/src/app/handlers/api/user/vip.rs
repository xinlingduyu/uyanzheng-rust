//! 会员验证
//! 
//! 功能说明：
//! 验证用户的VIP状态，返回VIP到期时间。
//!
//! 处理流程：
//! 1. 验证token参数
//! 2. 获取用户VIP到期时间
//! 3. 判断VIP是否有效
//! 4. 返回VIP状态和到期时间

use salvo::prelude::*;
use chrono::Utc;

use crate::app::utils::response::SignedApiResponse;
use crate::app::utils::validator::Validator;
use crate::app::models::requests::VipRequest;
use crate::app::middleware::user_auth::UserInfo;
use crate::app::middleware::app_context::AppInfo;

#[handler]
pub async fn check_vip(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    // 获取 app_key 用于签名（零拷贝）
    let app_key = match depot.get::<AppInfo>("app_info") {
        Ok(info) => info.app_key.as_str(),
        Err(_) => {
            res.render(Json(SignedApiResponse::<()>::error("应用信息不存在", 201, "")));
            return;
        }
    };

    // 解析请求参数
    let vip_req = match req.parse_json::<VipRequest>().await {
        Ok(data) => data,
        Err(_) => {
            res.render(Json(SignedApiResponse::<()>::error("参数解析失败", 201, app_key)));
            return;
        }
    };

    // PHP: 'token' => ['wordnum','32,32','TOKEN有误']
    let mut validator = Validator::new();
    validator.wordnum("token", &vip_req.token, 32, 32);
    
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

    let user_type = user_info.user_type.as_str();
    let current_time = Utc::now().timestamp();

    // PHP: if($this->app['app_type'] == 'user'){
    // PHP:     if($this->user['vip'] < time())$this->out->e(201,'验证失败');
    // PHP: }else{
    // PHP:     if($this->user['vip_exp'] < time())$this->out->e(201,'验证失败');
    // PHP: }
    // 根据用户类型检查不同的VIP字段
    if user_type == "user" {
        // 普通用户检查vip字段
        let vip_time = user_info.vip.unwrap_or(0);
        if vip_time < current_time {
            res.render(Json(SignedApiResponse::<()>::error("验证失败", 201, app_key)));
            return;
        }
    } else if user_type == "kami" {
        // 卡密用户检查vip_exp字段 - 一比一还原PHP
        let vip_exp_time = user_info.vip_exp.unwrap_or(0);
        if vip_exp_time < current_time {
            res.render(Json(SignedApiResponse::<()>::error("验证失败", 201, app_key)));
            return;
        }
    } else {
        res.render(Json(SignedApiResponse::<()>::error("用户类型错误", 201, app_key)));
        return;
    }
    
    // PHP: $this->out->e(200,'验证成功');
    res.render(Json(SignedApiResponse::success_with_msg("验证成功", app_key)));
}
