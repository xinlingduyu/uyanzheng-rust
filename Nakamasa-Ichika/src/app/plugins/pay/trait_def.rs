//! 支付插件trait定义

use serde::{Deserialize, Serialize};

/// 支付订单信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct PayOrder {
    pub order_no: String,
    pub name: String,
    pub money: f64,
    pub notify_url: String,
    pub return_url: String,
    /// 支付方式: pc, h5, app, native, jsapi
    #[serde(default = "default_pay_type")]
    pub pay_type: String,
    /// 客户端IP (H5支付必需)
    pub client_ip: Option<String>,
    /// 场景信息 (H5支付必需)
    pub scene_info: Option<serde_json::Value>,
}

fn default_pay_type() -> String {
    "h5".to_string()
}

/// 支付结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct PayResult {
    pub success: bool,
    pub pay_url: Option<String>,
    pub qrcode: Option<String>,
    pub message: String,
}

/// 支付异步通知验签后的标准结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct NotifyVerifyResult {
    /// 商户订单号
    pub order_no: String,
    /// 支付平台交易号
    pub trade_no: String,
    /// 支付金额（分）
    pub amount: Option<i64>,
}

/// 支付插件trait
/// 所有支付插件都需要实现这个trait
pub trait PayPlugin: Send + Sync {
    /// 获取插件名称
    fn name(&self) -> &str;

    /// 获取插件类型
    fn plugin_type(&self) -> &str;

    /// 获取插件配置表单
    fn config_form(&self) -> serde_json::Value;

    /// 初始化插件
    fn init(&mut self, config: serde_json::Value) -> Result<(), String>;

    /// 创建支付
    fn create(&self, order: &PayOrder) -> Result<PayResult, String>;

    /// 验证异步通知
    fn verify_notify(&self, data: serde_json::Value) -> Result<NotifyVerifyResult, String>;

    /// 查询订单
    fn query(&self, data: serde_json::Value) -> Result<serde_json::Value, String>;
}

/// 插件元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct PluginMeta {
    pub name: String,
    pub plugin_type: String,
    pub form: serde_json::Value,
}
