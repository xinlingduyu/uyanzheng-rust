//! Admin API handlers
//! 管理员API处理器

pub mod login;
pub mod app;
pub mod user;
pub mod system;
pub mod cdk_kami;
pub mod agent_list;
pub mod agent_group;
pub mod cdk_group;
pub mod cdk_user;
pub mod goods;
pub mod pay;
pub mod order;
pub mod statistics;
pub mod logs;
pub mod fen_order;
pub mod fen_event;
pub mod notice;
pub mod message;
pub mod set;
pub mod ver;
pub mod extend;
pub mod agent_cash;
pub mod download;
pub mod upload;
pub mod encryption;
pub mod function;
pub mod send;
pub mod adm_list;
pub mod blocklist;

use salvo::Router;
use crate::app::middleware::admin_auth::AdminAuth;

pub fn admin_routes() -> Router {
    Router::with_path("/api/admin")
        // Login - 不需要认证
        .push(Router::with_path("/login").post(login::login))
        // tokenVerify - 使用skip_verify模式，返回验证结果
        .push(Router::with_path("/admin/verify").hoop(AdminAuth::new().skip_verify()).get(login::token_verify))
        // System - 需要认证
        .push(Router::with_path("/system/ulogin").hoop(AdminAuth::new()).post(system::ulogin))
        .push(Router::with_path("/system/urefresh").hoop(AdminAuth::new()).get(system::urefresh))
        .push(Router::with_path("/system/uquit").hoop(AdminAuth::new()).post(system::uquit))
        .push(Router::with_path("/system/uinfo").hoop(AdminAuth::new()).get(system::uinfo))
        .push(Router::with_path("/system/clearCache").hoop(AdminAuth::new()).post(system::clear_cache))
        .push(Router::with_path("/system/getSet").hoop(AdminAuth::new()).post(system::get_set))
        .push(Router::with_path("/system/editSet").hoop(AdminAuth::new()).post(system::edit_set))
        .push(Router::with_path("/system/getUserApiRouter").hoop(AdminAuth::new()).post(system::get_user_api_router))
        .push(Router::with_path("/system/editUserApiRouter").hoop(AdminAuth::new()).post(system::edit_user_api_router))
        .push(Router::with_path("/system/switchUserApiRouter").hoop(AdminAuth::new()).post(system::switch_user_api_router))
        .push(Router::with_path("/system/getUserApiCode").hoop(AdminAuth::new()).post(system::get_user_api_code))
        .push(Router::with_path("/system/editUserApiCode").hoop(AdminAuth::new()).post(system::edit_user_api_code))
        .push(Router::with_path("/system/switchUserApiCode").hoop(AdminAuth::new()).post(system::switch_user_api_code))
        // App - 需要认证
        .push(Router::with_path("/app/get").hoop(AdminAuth::new()).post(app::get_info))
        .push(Router::with_path("/app/getInfo").hoop(AdminAuth::new()).post(app::get_info))
        .push(Router::with_path("/app/getUrl").hoop(AdminAuth::new()).post(app::get_url))
        .push(Router::with_path("/app/list").hoop(AdminAuth::new()).post(app::get_list))
        .push(Router::with_path("/app/getInherit").hoop(AdminAuth::new()).post(app::get_inherit))
        .push(Router::with_path("/app/all").hoop(AdminAuth::new()).get(app::get_all))
        .push(Router::with_path("/app/add").hoop(AdminAuth::new()).post(app::add))
        .push(Router::with_path("/app/edit").hoop(AdminAuth::new()).post(app::edit))
        .push(Router::with_path("/app/del").hoop(AdminAuth::new()).post(app::del))
        .push(Router::with_path("/app/send").hoop(AdminAuth::new()).
        get(send::get_info))
        .push(Router::with_path("/app/pay").hoop(AdminAuth::new()).get(pay::get_info))
        .push(Router::with_path("/app/pay/edit").hoop(AdminAuth::new()).post(pay::edit))
        // User - 需要认证
        .push(Router::with_path("/user/list").hoop(AdminAuth::new()).post(user::get_list))
        .push(Router::with_path("/user/get").hoop(AdminAuth::new()).post(user::get))
        .push(Router::with_path("/user/add").hoop(AdminAuth::new()).post(user::add))
        .push(Router::with_path("/user/award").hoop(AdminAuth::new()).post(user::award))
        .push(Router::with_path("/user/edit").hoop(AdminAuth::new()).post(user::edit))
        .push(Router::with_path("/user/del").hoop(AdminAuth::new()).post(user::del))
        .push(Router::with_path("/user/delall").hoop(AdminAuth::new()).post(user::del_all))
        .push(Router::with_path("/user/getLog").hoop(AdminAuth::new()).post(user::get_log))
        .push(Router::with_path("/user/unbindSn").hoop(AdminAuth::new()).post(user::unbind_sn))
        // CDK Kami - 需要认证
        .push(Router::with_path("/cdkKami/list").hoop(AdminAuth::new()).post(cdk_kami::get_list))
        .push(Router::with_path("/cdkKami/add").hoop(AdminAuth::new()).post(cdk_kami::add))
        .push(Router::with_path("/cdkKami/edit").hoop(AdminAuth::new()).post(cdk_kami::edit))
        .push(Router::with_path("/cdkKami/del").hoop(AdminAuth::new()).post(cdk_kami::del))
        .push(Router::with_path("/cdkKami/delall").hoop(AdminAuth::new()).post(cdk_kami::del_all))
        .push(Router::with_path("/cdkKami/outall").hoop(AdminAuth::new()).post(cdk_kami::out_all))
        // Agent - 需要认证
        .push(Router::with_path("/agentList/list").hoop(AdminAuth::new()).post(agent_list::get_list))
        .push(Router::with_path("/agentList/add").hoop(AdminAuth::new()).post(agent_list::add))
        .push(Router::with_path("/agentList/edit").hoop(AdminAuth::new()).post(agent_list::edit))
        .push(Router::with_path("/agentList/del").hoop(AdminAuth::new()).post(agent_list::del))
        .push(Router::with_path("/agentGroup/list").hoop(AdminAuth::new()).post(agent_group::get_list))
        .push(Router::with_path("/agentGroup/add").hoop(AdminAuth::new()).post(agent_group::add))
        .push(Router::with_path("/agentGroup/edit").hoop(AdminAuth::new()).post(agent_group::edit))
        .push(Router::with_path("/agentGroup/del").hoop(AdminAuth::new()).post(agent_group::del))
        // CDK Group - 需要认证
        .push(Router::with_path("/cdkGroup/get").hoop(AdminAuth::new()).get(cdk_group::get_all_list))
        .push(Router::with_path("/cdkGroup/list").hoop(AdminAuth::new()).post(cdk_group::get_list))
        .push(Router::with_path("/cdkGroup/add").hoop(AdminAuth::new()).post(cdk_group::add))
        .push(Router::with_path("/cdkGroup/edit").hoop(AdminAuth::new()).post(cdk_group::edit))
        .push(Router::with_path("/cdkGroup/del").hoop(AdminAuth::new()).post(cdk_group::del))
        .push(Router::with_path("/cdkGroup/delall").hoop(AdminAuth::new()).post(cdk_group::del_all))
        // CDK User - 需要认证
        .push(Router::with_path("/cdkUser/list").hoop(AdminAuth::new()).post(cdk_user::get_list))
        .push(Router::with_path("/cdkUser/add").hoop(AdminAuth::new()).post(cdk_user::add))
        .push(Router::with_path("/cdkUser/edit").hoop(AdminAuth::new()).post(cdk_user::edit))
        .push(Router::with_path("/cdkUser/editState").hoop(AdminAuth::new()).post(cdk_user::edit_state))
        .push(Router::with_path("/cdkUser/del").hoop(AdminAuth::new()).post(cdk_user::del))
        // Goods - 需要认证
        .push(Router::with_path("/goods/list").hoop(AdminAuth::new()).post(goods::get_list))
        .push(Router::with_path("/goods/add").hoop(AdminAuth::new()).post(goods::add))
        .push(Router::with_path("/goods/edit").hoop(AdminAuth::new()).post(goods::edit))
        .push(Router::with_path("/goods/editState").hoop(AdminAuth::new()).post(goods::edit_state))
        .push(Router::with_path("/goods/del").hoop(AdminAuth::new()).post(goods::del))
        // Order - 需要认证
        .push(Router::with_path("/order/list").hoop(AdminAuth::new()).post(order::get_list))
        .push(Router::with_path("/order/statistics").hoop(AdminAuth::new()).post(order::statistics))
        .push(Router::with_path("/order/edit").hoop(AdminAuth::new()).post(order::edit))
        .push(Router::with_path("/order/del").hoop(AdminAuth::new()).post(order::del))
        // Statistics - 需要认证
        .push(Router::with_path("/statistics/get").hoop(AdminAuth::new()).get(statistics::get))
        // Logs - 需要认证
        .push(Router::with_path("/logs/list").hoop(AdminAuth::new()).post(logs::get_list))
        .push(Router::with_path("/logs/list/user").hoop(AdminAuth::new()).post(logs::get_list_user))
        .push(Router::with_path("/logs/list/admin").hoop(AdminAuth::new()).post(logs::get_list_admin))
        .push(Router::with_path("/logs/type/user").hoop(AdminAuth::new()).get(logs::get_type_user))
        .push(Router::with_path("/logs/type/admin").hoop(AdminAuth::new()).get(logs::get_type_admin))
        .push(Router::with_path("/logs/del").hoop(AdminAuth::new()).post(logs::del))
        // Fen Order - 需要认证
        .push(Router::with_path("/fenOrder/list").hoop(AdminAuth::new()).post(fen_order::get_list))
        .push(Router::with_path("/fenOrder/edit").hoop(AdminAuth::new()).post(fen_order::edit))
        .push(Router::with_path("/fenOrder/del").hoop(AdminAuth::new()).post(fen_order::del))
        .push(Router::with_path("/fenOrder/delall").hoop(AdminAuth::new()).post(fen_order::delall))
        // Fen Event - 需要认证
        .push(Router::with_path("/fenEvent/list").hoop(AdminAuth::new()).post(fen_event::get_list))
        .push(Router::with_path("/fenEvent/add").hoop(AdminAuth::new()).post(fen_event::add))
        .push(Router::with_path("/fenEvent/edit").hoop(AdminAuth::new()).post(fen_event::edit))
        .push(Router::with_path("/fenEvent/editState").hoop(AdminAuth::new()).post(fen_event::edit_state))
        .push(Router::with_path("/fenEvent/del").hoop(AdminAuth::new()).post(fen_event::del))
        .push(Router::with_path("/fenEvent/delall").hoop(AdminAuth::new()).post(fen_event::del_all))
        // Notice - 需要认证
        .push(Router::with_path("/notice/list").hoop(AdminAuth::new()).post(notice::get_list))
        .push(Router::with_path("/notice/add").hoop(AdminAuth::new()).post(notice::add))
        .push(Router::with_path("/notice/edit").hoop(AdminAuth::new()).post(notice::edit))
        .push(Router::with_path("/notice/del").hoop(AdminAuth::new()).post(notice::del))
        // Message - 需要认证
        .push(Router::with_path("/message/list").hoop(AdminAuth::new()).post(message::get_list))
        .push(Router::with_path("/message/edit").hoop(AdminAuth::new()).post(message::edit))
        .push(Router::with_path("/message/del").hoop(AdminAuth::new()).post(message::del))
        // Set - 需要认证
        .push(Router::with_path("/set/list").hoop(AdminAuth::new()).post(set::get_list))
        .push(Router::with_path("/set/edit").hoop(AdminAuth::new()).post(set::edit))
        // Ver - 需要认证
        .push(Router::with_path("/ver/list").hoop(AdminAuth::new()).post(ver::get_list))
        .push(Router::with_path("/ver/add").hoop(AdminAuth::new()).post(ver::add))
        .push(Router::with_path("/ver/edit").hoop(AdminAuth::new()).post(ver::edit))
        .push(Router::with_path("/ver/del").hoop(AdminAuth::new()).post(ver::del))
        // Extend - 需要认证
        .push(Router::with_path("/extend/list").hoop(AdminAuth::new()).post(extend::get_list))
        .push(Router::with_path("/extend/add").hoop(AdminAuth::new()).post(extend::add))
        .push(Router::with_path("/extend/edit").hoop(AdminAuth::new()).post(extend::edit))
        .push(Router::with_path("/extend/del").hoop(AdminAuth::new()).post(extend::del))
        // Agent Cash - 需要认证
        .push(Router::with_path("/agentCash/list").hoop(AdminAuth::new()).post(agent_cash::get_list))
        .push(Router::with_path("/agentCash/edit").hoop(AdminAuth::new()).post(agent_cash::edit))
        .push(Router::with_path("/agentCash/del").hoop(AdminAuth::new()).post(agent_cash::del))
        // Download - 需要认证
        .push(Router::with_path("/download/index").hoop(AdminAuth::new()).post(download::index))
        // Upload - 需要认证
        .push(Router::with_path("/upload/index").hoop(AdminAuth::new()).post(upload::index))
        .push(Router::with_path("/upload/img").hoop(AdminAuth::new()).post(upload::img))
        // Encryption - 需要认证
        .push(Router::with_path("/encryption/plug").hoop(AdminAuth::new()).get(encryption::get_plug))
        .push(Router::with_path("/encryption/list").hoop(AdminAuth::new()).post(encryption::get_list))
        .push(Router::with_path("/encryption/add").hoop(AdminAuth::new()).post(encryption::add))
        .push(Router::with_path("/encryption/edit").hoop(AdminAuth::new()).post(encryption::edit))
        .push(Router::with_path("/encryption/editSign").hoop(AdminAuth::new()).post(encryption::edit_sign))
        .push(Router::with_path("/encryption/del").hoop(AdminAuth::new()).post(encryption::del))
        // Function - 需要认证
        .push(Router::with_path("/functions/list").hoop(AdminAuth::new()).post(function::get_list))
        .push(Router::with_path("/functions/getInfo").hoop(AdminAuth::new()).post(function::get_info))
        .push(Router::with_path("/functions/getCode").hoop(AdminAuth::new()).post(function::get_code))
        .push(Router::with_path("/functions/add").hoop(AdminAuth::new()).post(function::add))
        .push(Router::with_path("/functions/edit").hoop(AdminAuth::new()).post(function::edit))
        .push(Router::with_path("/functions/editState").hoop(AdminAuth::new()).post(function::edit_state))
        .push(Router::with_path("/functions/del").hoop(AdminAuth::new()).post(function::del))
        // Send - 需要认证
        .push(Router::with_path("/send").hoop(AdminAuth::new()).get(send::get_info))
        .push(Router::with_path("/send/getInfo").hoop(AdminAuth::new()).post(send::get_info))
        .push(Router::with_path("/send/edit").hoop(AdminAuth::new()).post(send::edit))
        // Admin List - 需要认证
        .push(Router::with_path("/admList/list").hoop(AdminAuth::new()).post(adm_list::get_list))
        .push(Router::with_path("/admList/add").hoop(AdminAuth::new()).post(adm_list::add))
        .push(Router::with_path("/admList/edit").hoop(AdminAuth::new()).post(adm_list::edit))
        .push(Router::with_path("/admList/del").hoop(AdminAuth::new()).post(adm_list::del))
        .push(Router::with_path("/admin/setAvatars").hoop(AdminAuth::new()).post(adm_list::set_avatars))
        // Blocklist - 需要认证
        .push(Router::with_path("/blocklist/list").hoop(AdminAuth::new()).post(blocklist::get_list))
        .push(Router::with_path("/blocklist/add").hoop(AdminAuth::new()).post(blocklist::add))
        .push(Router::with_path("/blocklist/edit").hoop(AdminAuth::new()).post(blocklist::edit))
        .push(Router::with_path("/blocklist/del").hoop(AdminAuth::new()).post(blocklist::del))
        .push(Router::with_path("/blocklist/delall").hoop(AdminAuth::new()).post(blocklist::del_all))
}