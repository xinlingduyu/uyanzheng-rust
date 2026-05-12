//! 用户API模块 - Organized by business domain
//!
//! ## Groups
//! - auth: logon.rs, logout.rs, reg.rs, resetPwd.rs, modifyPwd.rs
//! - oauth: wxlogon.rs, wxlogonCallback.rs, wxlogonQuery.rs, wxBindSDK.rs, wxloginSDK.rs, qqBindSDK.rs, qqlogonCallback.rs, qqloginWeb.rs, qqloginQuery.rs, qqloginSDK.rs
//! - profile: info.rs, modifyName.rs, modifyPic.rs, setAcctno.rs, setEmail.rs, setExtend.rs, setPhone.rs, reEmail.rs, rePhone.rs
//! - device: getUdid.rs, bindUdid.rs, reUdid.rs, heartbeat.rs, ban.rs
//! - trade: pay.rs, order.rs, orderQuery.rs, goods.rs, vip.rs, kamiTopup.rs, fen.rs
//! - message: messageAdd.rs, messageContent.rs, messageEnd.rs, messageList.rs, messageReply.rs
//! - misc: getCode.rs, signIn.rs, cloudFunction.rs, ai.rs, ini.rs, upload.rs

pub mod auth;
pub mod oauth;
pub mod profile;
pub mod device;
pub mod trade;
pub mod message;
pub mod misc;
use salvo::prelude::*;
use crate::app::middleware::user_auth::UserAuth;
use crate::app::middleware::app_context::AppContext;

pub fn routes() -> Router {
    Router::with_path("/api/user/{appid}/{ver_key}/{ver_val}")
        // 应用上下文中间件 - 从路径提取 appid、ver_key 和 ver_val 并查询 app 数据
        .hoop(AppContext::new())
        
        // ============== 不需要token认证的接口 ==============
        
        // 登录接口
        .push(Router::with_path("/logon").post(auth::logon::login))
        .push(Router::with_path("/kamiLogin").post(auth::logon::kami_login))
        
        // 注册
        .push(Router::with_path("/reg").post(auth::reg::register))
        
        // 重置密码
        .push(Router::with_path("/resetPwd").post(auth::resetPwd::reset_pwd))
        
        // 解绑邮箱 - 需要token
        .push(Router::with_path("/reEmail").hoop(UserAuth::new()).post(profile::reEmail::re_email))
        
        // 解绑手机 - 需要token
        .push(Router::with_path("/rePhone").hoop(UserAuth::new()).post(profile::rePhone::re_phone))
        
        // 获取验证码
        .push(Router::with_path("/getCode").post(misc::getCode::get_code))
        
        // 心跳（内部验证token）
        .push(Router::with_path("/heartbeat").post(device::heartbeat::heartbeat))
        
        // 商品列表
        .push(Router::with_path("/goods").post(trade::goods::goods))
        
        // 在线充值
        .push(Router::with_path("/pay").post(trade::pay::pay))
        
        // 获取配置
        .push(Router::with_path("/ini").get(misc::ini::ini))
        
        // 微信登录相关
        .push(Router::with_path("/wxlogon").post(oauth::wxlogon::wx_logon))
        .push(Router::with_path("/wxlogonCallback").get(oauth::wxlogonCallback::wx_logon_callback))
        .push(Router::with_path("/wxlogonQuery").post(oauth::wxlogonQuery::wx_logon_query))
        
        // QQ登录相关
        // QQ互联回调路径 - 用于QQ授权后回调
        .push(Router::with_path("/qqlogonCallback").get(oauth::qqlogonCallback::qq_logon_callback))
        
        // QQ网页登录 - 获取扫码登录URL
        .push(Router::with_path("/qqloginWeb").post(oauth::qqloginWeb::qq_login_web))
        
        // QQ网页登录状态查询 - 轮询登录结果
        .push(Router::with_path("/qqloginQuery").post(oauth::qqloginQuery::qq_login_query))
        
        // QQ SDK登录
        .push(Router::with_path("/qqloginSDK").post(oauth::qqloginSDK::qq_login_sdk))
        
        // 微信SDK绑定 - 需要token
        .push(Router::with_path("/wxBindSDK").hoop(UserAuth::new()).post(oauth::wxBindSDK::wx_bind_sdk))
        
        // QQ SDK绑定 - 需要token
        .push(Router::with_path("/qqBindSDK").hoop(UserAuth::new()).post(oauth::qqBindSDK::qq_bind_sdk))
        
        // 微信SDK登录
        .push(Router::with_path("/wxloginSDK").post(oauth::wxloginSDK::wx_login_sdk))
        
        // 退出登录（内部验证token）
        .push(Router::with_path("/logout").post(auth::logout::logout))
        
        // ============== 需要token认证的接口 ==============
        
        // 绑定设备 - 需要token
        .push(Router::with_path("/bindUdid").hoop(UserAuth::new()).post(device::bindUdid::bind_udid))
        
        // 获取设备列表 - 需要token
        .push(Router::with_path("/getUdid").hoop(UserAuth::new().allow_udid()).post(device::getUdid::get_udid))
        
        // 解绑设备 - 需要token
        .push(Router::with_path("/reUdid").hoop(UserAuth::new().allow_udid()).post(device::reUdid::re_udid))
        
        // 获取个人信息 - 需要token
        .push(Router::with_path("/info").hoop(UserAuth::new()).post(profile::info::get_info))
        
        // 每日签到 - 需要token
        .push(Router::with_path("/signIn").hoop(UserAuth::new()).post(misc::signIn::sign_in))
        
        // 订单列表 - 需要token
        .push(Router::with_path("/order").hoop(UserAuth::new()).post(trade::order::order))
        
        // 订单查询 - 需要token
        .push(Router::with_path("/orderQuery").hoop(UserAuth::new()).post(trade::orderQuery::order_query))
        
        // 会员验证 - 需要token
        .push(Router::with_path("/vip").hoop(UserAuth::new()).post(trade::vip::check_vip))
        
        // 修改昵称 - 需要token
        .push(Router::with_path("/modifyName").hoop(UserAuth::new()).post(profile::modifyName::modify_name))
        
        // 修改头像 - 需要token
        .push(Router::with_path("/modifyPic").hoop(UserAuth::new()).post(profile::modifyPic::modify_pic))
        
        // 修改密码 - 需要token
        .push(Router::with_path("/modifyPwd").hoop(UserAuth::new()).post(auth::modifyPwd::modify_pwd))
        
        // 设置账号 - 需要token
        .push(Router::with_path("/setAcctno").hoop(UserAuth::new()).post(profile::setAcctno::set_acctno))
        
        // 绑定邮箱 - 需要token
        .push(Router::with_path("/setEmail").hoop(UserAuth::new()).post(profile::setEmail::set_email))
        
        // 设置扩展信息 - 需要token
        .push(Router::with_path("/setExtend").hoop(UserAuth::new()).post(profile::setExtend::set_extend))
        
        // 绑定手机号 - 需要token
        .push(Router::with_path("/setPhone").hoop(UserAuth::new()).post(profile::setPhone::set_phone))
        
        // 积分验证 - 需要token
        .push(Router::with_path("/fen").hoop(UserAuth::new()).post(trade::fen::fen_verify))
        
        // 卡密充值 - 需要token
        .push(Router::with_path("/kamiTopup").hoop(UserAuth::new()).post(trade::kamiTopup::kami_topup))
        
        // 云函数 - 需要token
        .push(Router::with_path("/cloudFunction").hoop(UserAuth::new()).post(misc::cloudFunction::cloud_function))
        
        // 上传文件 - 需要token
        .push(Router::with_path("/upload").hoop(UserAuth::new()).post(misc::upload::upload))
        
        // 留言相关 - 需要token
        .push(Router::with_path("/messageAdd").hoop(UserAuth::new()).post(message::messageAdd::message_add))
        .push(Router::with_path("/messageContent").hoop(UserAuth::new()).post(message::messageContent::message_content))
        .push(Router::with_path("/messageEnd").hoop(UserAuth::new()).post(message::messageEnd::message_end))
        .push(Router::with_path("/messageList").hoop(UserAuth::new()).post(message::messageList::message_list))
        .push(Router::with_path("/messageReply").hoop(UserAuth::new()).post(message::messageReply::message_reply))
        
        // 账户禁用 - 需要token
        .push(Router::with_path("/ban").hoop(UserAuth::new()).post(device::ban::ban_user))
        
        // AI 对话 - 需要token
        .push(Router::with_path("/ai").hoop(UserAuth::new()).post(misc::ai::ai_chat))
}
