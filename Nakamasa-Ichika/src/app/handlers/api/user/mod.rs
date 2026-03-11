//! 用户API模块
//! 
//! 模块说明：
//! 包含所有用户相关的API接口，包括登录注册、信息管理、支付充值等功能。
//! 
//! 路由结构：
//! - 所有路由前缀：/api/user/{appid}/{ver_key}/{ver_val}
//! - 需要token认证的接口使用UserAuth中间件
//! - 不需要token的接口直接暴露

pub mod bindUdid;
pub mod getUdid;
pub mod reUdid;
pub mod logon;
pub mod info;
pub mod logout;
pub mod ini;
pub mod heartbeat;
pub mod signIn;
pub mod goods;
pub mod order;
pub mod orderQuery;
pub mod vip;
pub mod modifyName;
pub mod modifyPic;
pub mod modifyPwd;
pub mod setAcctno;
pub mod setEmail;
pub mod setExtend;
pub mod setPhone;
pub mod reg;
pub mod resetPwd;
pub mod reEmail;
pub mod rePhone;
pub mod fen;
pub mod getCode;
pub mod kamiTopup;
pub mod pay;
pub mod cloudFunction;
pub mod messageAdd;
pub mod messageContent;
pub mod messageEnd;
pub mod messageList;
pub mod messageReply;
pub mod upload;
pub mod wxlogon;
pub mod wxlogonCallback;
pub mod wxlogonQuery;
pub mod wxBindSDK;
pub mod wxloginSDK;
pub mod qqBindSDK;
pub mod qqlogonCallback;
pub mod qqloginWeb;
pub mod qqloginQuery;
pub mod qqloginSDK;
pub mod ban;

use salvo::prelude::*;
use crate::app::middleware::user_auth::UserAuth;
use crate::app::middleware::app_context::AppContext;

pub fn routes() -> Router {
    Router::with_path("/api/user/{appid}/{ver_key}/{ver_val}")
        // 应用上下文中间件 - 从路径提取 appid、ver_key 和 ver_val 并查询 app 数据
        .hoop(AppContext::new())
        
        // ============== 不需要token认证的接口 ==============
        
        // 登录接口
        .push(Router::with_path("/logon").post(logon::login))
        .push(Router::with_path("/kamiLogin").post(logon::kami_login))
        
        // 注册
        .push(Router::with_path("/reg").post(reg::register))
        
        // 重置密码
        .push(Router::with_path("/resetPwd").post(resetPwd::reset_pwd))
        
        // 解绑邮箱 - 需要token
        .push(Router::with_path("/reEmail").hoop(UserAuth::new()).post(reEmail::re_email))
        
        // 解绑手机 - 需要token
        .push(Router::with_path("/rePhone").hoop(UserAuth::new()).post(rePhone::re_phone))
        
        // 获取验证码
        .push(Router::with_path("/getCode").post(getCode::get_code))
        
        // 心跳（内部验证token）
        .push(Router::with_path("/heartbeat").post(heartbeat::heartbeat))
        
        // 商品列表
        .push(Router::with_path("/goods").post(goods::goods))
        
        // 在线充值
        .push(Router::with_path("/pay").post(pay::pay))
        
        // 获取配置
        .push(Router::with_path("/ini").get(ini::ini))
        
        // 微信登录相关
        .push(Router::with_path("/wxlogon").post(wxlogon::wx_logon))
        .push(Router::with_path("/wxlogonCallback").get(wxlogonCallback::wx_logon_callback))
        .push(Router::with_path("/wxlogonQuery").post(wxlogonQuery::wx_logon_query))
        
        // QQ登录相关
        // QQ互联回调路径 - 用于QQ授权后回调
        .push(Router::with_path("/qqlogonCallback").get(qqlogonCallback::qq_logon_callback))
        
        // QQ网页登录 - 获取扫码登录URL
        .push(Router::with_path("/qqloginWeb").post(qqloginWeb::qq_login_web))
        
        // QQ网页登录状态查询 - 轮询登录结果
        .push(Router::with_path("/qqloginQuery").post(qqloginQuery::qq_login_query))
        
        // QQ SDK登录
        .push(Router::with_path("/qqloginSDK").post(qqloginSDK::qq_login_sdk))
        
        // 微信SDK绑定 - 需要token
        .push(Router::with_path("/wxBindSDK").hoop(UserAuth::new()).post(wxBindSDK::wx_bind_sdk))
        
        // QQ SDK绑定 - 需要token
        .push(Router::with_path("/qqBindSDK").hoop(UserAuth::new()).post(qqBindSDK::qq_bind_sdk))
        
        // 微信SDK登录
        .push(Router::with_path("/wxloginSDK").post(wxloginSDK::wx_login_sdk))
        
        // 退出登录（内部验证token）
        .push(Router::with_path("/logout").post(logout::logout))
        
        // ============== 需要token认证的接口 ==============
        
        // 绑定设备 - 需要token
        .push(Router::with_path("/bindUdid").hoop(UserAuth::new()).post(bindUdid::bind_udid))
        
        // 获取设备列表 - 需要token
        .push(Router::with_path("/getUdid").hoop(UserAuth::new().allow_udid()).post(getUdid::get_udid))
        
        // 解绑设备 - 需要token
        .push(Router::with_path("/reUdid").hoop(UserAuth::new().allow_udid()).post(reUdid::re_udid))
        
        // 获取个人信息 - 需要token
        .push(Router::with_path("/info").hoop(UserAuth::new()).post(info::get_info))
        
        // 每日签到 - 需要token
        .push(Router::with_path("/signIn").hoop(UserAuth::new()).post(signIn::sign_in))
        
        // 订单列表 - 需要token
        .push(Router::with_path("/order").hoop(UserAuth::new()).post(order::order))
        
        // 订单查询 - 需要token
        .push(Router::with_path("/orderQuery").hoop(UserAuth::new()).post(orderQuery::order_query))
        
        // 会员验证 - 需要token
        .push(Router::with_path("/vip").hoop(UserAuth::new()).post(vip::check_vip))
        
        // 修改昵称 - 需要token
        .push(Router::with_path("/modifyName").hoop(UserAuth::new()).post(modifyName::modify_name))
        
        // 修改头像 - 需要token
        .push(Router::with_path("/modifyPic").hoop(UserAuth::new()).post(modifyPic::modify_pic))
        
        // 修改密码 - 需要token
        .push(Router::with_path("/modifyPwd").hoop(UserAuth::new()).post(modifyPwd::modify_pwd))
        
        // 设置账号 - 需要token
        .push(Router::with_path("/setAcctno").hoop(UserAuth::new()).post(setAcctno::set_acctno))
        
        // 绑定邮箱 - 需要token
        .push(Router::with_path("/setEmail").hoop(UserAuth::new()).post(setEmail::set_email))
        
        // 设置扩展信息 - 需要token
        .push(Router::with_path("/setExtend").hoop(UserAuth::new()).post(setExtend::set_extend))
        
        // 绑定手机号 - 需要token
        .push(Router::with_path("/setPhone").hoop(UserAuth::new()).post(setPhone::set_phone))
        
        // 积分验证 - 需要token
        .push(Router::with_path("/fen").hoop(UserAuth::new()).post(fen::fen_verify))
        
        // 卡密充值 - 需要token
        .push(Router::with_path("/kamiTopup").hoop(UserAuth::new()).post(kamiTopup::kami_topup))
        
        // 云函数 - 需要token
        .push(Router::with_path("/cloudFunction").hoop(UserAuth::new()).post(cloudFunction::cloud_function))
        
        // 上传文件 - 需要token
        .push(Router::with_path("/upload").hoop(UserAuth::new()).post(upload::upload))
        
        // 留言相关 - 需要token
        .push(Router::with_path("/messageAdd").hoop(UserAuth::new()).post(messageAdd::message_add))
        .push(Router::with_path("/messageContent").hoop(UserAuth::new()).post(messageContent::message_content))
        .push(Router::with_path("/messageEnd").hoop(UserAuth::new()).post(messageEnd::message_end))
        .push(Router::with_path("/messageList").hoop(UserAuth::new()).post(messageList::message_list))
        .push(Router::with_path("/messageReply").hoop(UserAuth::new()).post(messageReply::message_reply))
        
        // 账户禁用 - 需要token
        .push(Router::with_path("/ban").hoop(UserAuth::new()).post(ban::ban_user))
}
