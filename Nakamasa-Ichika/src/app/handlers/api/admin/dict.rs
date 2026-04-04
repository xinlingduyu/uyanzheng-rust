//! Admin Dict controller
//! 字典管理控制器 - 返回系统字典数据

use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::app::utils::response::ApiResponse;

/// 字典项
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DictItem {
    pub label: String,
    pub value: serde_json::Value,
}

/// 获取所有字典数据
#[handler]
pub async fn dict_all(_req: &mut Request, _depot: &mut Depot, res: &mut Response) {
    let mut dict: HashMap<String, Vec<DictItem>> = HashMap::new();
    
    // 数据状态
    dict.insert("data_status".to_string(), vec![
        DictItem { label: "启用".to_string(), value: serde_json::json!(1) },
        DictItem { label: "禁用".to_string(), value: serde_json::json!(0) },
    ]);
    
    // 性别
    dict.insert("sex".to_string(), vec![
        DictItem { label: "男".to_string(), value: serde_json::json!(1) },
        DictItem { label: "女".to_string(), value: serde_json::json!(2) },
    ]);
    
    // 用户状态
    dict.insert("user_status".to_string(), vec![
        DictItem { label: "正常".to_string(), value: serde_json::json!(0) },
        DictItem { label: "封禁".to_string(), value: serde_json::json!(1) },
    ]);
    
    // 应用状态
    dict.insert("app_state".to_string(), vec![
        DictItem { label: "开启".to_string(), value: serde_json::json!("on") },
        DictItem { label: "关闭".to_string(), value: serde_json::json!("off") },
    ]);
    
    // 支付类型
    dict.insert("pay_type".to_string(), vec![
        DictItem { label: "支付宝".to_string(), value: serde_json::json!("ali") },
        DictItem { label: "微信".to_string(), value: serde_json::json!("wx") },
    ]);
    
    // 卡密类型
    dict.insert("cdk_type".to_string(), vec![
        DictItem { label: "VIP时长".to_string(), value: serde_json::json!("vip") },
        DictItem { label: "积分".to_string(), value: serde_json::json!("fen") },
        DictItem { label: "设备额度".to_string(), value: serde_json::json!("addsn") },
    ]);
    
    // 商品类型
    dict.insert("goods_type".to_string(), vec![
        DictItem { label: "VIP时长".to_string(), value: serde_json::json!("vip") },
        DictItem { label: "积分".to_string(), value: serde_json::json!("fen") },
        DictItem { label: "代理".to_string(), value: serde_json::json!("agent") },
        DictItem { label: "设备额度".to_string(), value: serde_json::json!("addsn") },
    ]);
    
    // 黑名单类型
    dict.insert("blocklist_type".to_string(), vec![
        DictItem { label: "IP".to_string(), value: serde_json::json!("ip") },
        DictItem { label: "设备".to_string(), value: serde_json::json!("sn") },
    ]);
    
    // 日志类型 - 用户
    dict.insert("log_type_user".to_string(), vec![
        DictItem { label: "登录".to_string(), value: serde_json::json!("login") },
        DictItem { label: "注册".to_string(), value: serde_json::json!("register") },
        DictItem { label: "充值".to_string(), value: serde_json::json!("recharge") },
        DictItem { label: "卡密充值".to_string(), value: serde_json::json!("kami") },
    ]);
    
    // 日志类型 - 管理员
    dict.insert("log_type_admin".to_string(), vec![
        DictItem { label: "登录".to_string(), value: serde_json::json!("login") },
        DictItem { label: "用户管理".to_string(), value: serde_json::json!("user") },
        DictItem { label: "应用管理".to_string(), value: serde_json::json!("app") },
        DictItem { label: "卡密管理".to_string(), value: serde_json::json!("cdk") },
        DictItem { label: "订单管理".to_string(), value: serde_json::json!("order") },
    ]);
    
    // 加密类型
    dict.insert("encrypt_type".to_string(), vec![
        DictItem { label: "RC4".to_string(), value: serde_json::json!("rc4") },
        DictItem { label: "AES".to_string(), value: serde_json::json!("aes") },
        DictItem { label: "DES".to_string(), value: serde_json::json!("des") },
        DictItem { label: "RSA".to_string(), value: serde_json::json!("rsa") },
    ]);
    
    // VIP权限要求
    dict.insert("vip_require".to_string(), vec![
        DictItem { label: "无要求".to_string(), value: serde_json::json!(0) },
        DictItem { label: "需要VIP".to_string(), value: serde_json::json!(1) },
    ]);

    res.render(Json(ApiResponse::success("成功", Some(dict))));
}
