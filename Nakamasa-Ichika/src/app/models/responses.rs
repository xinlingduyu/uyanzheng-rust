// 用户API响应结构体
use serde::{Serialize, Deserialize};

// ========== 用户信息 ==========
#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub uid: u64,
    pub phone: Option<i64>,
    pub email: Option<String>,
    pub acctno: String,
    pub name: Option<String>,
    pub pic: String,
    #[serde(rename = "invID")]
    pub inv_id: i64,
    #[serde(rename = "invCode", skip_serializing_if = "Option::is_none")]
    pub inv_code: Option<String>,
    pub fen: i64,
    #[serde(rename = "vipExpTime")]
    pub vip_exp_time: i64,
    #[serde(rename = "vipExpDate")]
    pub vip_exp_date: String,
    pub extend: Option<serde_json::Value>,
    #[serde(rename = "openWx")]
    pub open_wx: Option<String>,
    #[serde(rename = "openQQ")]
    pub open_qq: Option<String>,
}

/// IP 地域信息
#[derive(Debug, Serialize, Clone, Default)]
pub struct IpLocation {
    /// 国家
    pub country: String,
    /// 省份
    pub province: String,
    /// 城市
    pub city: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub state: String,
    pub info: UserInfo,
    /// IP 地域信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_location: Option<IpLocation>,
}

// ========== 设备信息 ==========
#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub udid: String,
    pub time: i64,
}

// ========== 商品列表 ==========
#[derive(Debug, Serialize)]
pub struct GoodsItem {
    pub id: i64,
    pub name: String,
    pub r#type: String,
    pub money: i64,
    pub blurb: String,
}

#[derive(Debug, Serialize)]
pub struct GoodsListResponse {
    pub currentPage: u32,
    pub dataTotal: u32,
    pub list: Vec<GoodsItem>,
    pub pageTotal: u32,
}

// ========== 订单列表 ==========
#[derive(Debug, Serialize)]
pub struct OrderItem {
    pub order_no: String,
    pub trade_no: Option<String>,
    pub name: String,
    pub money: i64,
    pub ptype: String,
    pub add_time: i64,
    pub end_time: Option<i64>,
    pub state: String,
}

#[derive(Debug, Serialize)]
pub struct OrderListResponse {
    pub list: Vec<OrderItem>,
}

// ========== 消息列表 ==========
#[derive(Debug, Serialize)]
pub struct MessageItem {
    pub id: i64,
    pub title: String,
    pub time: i64,
    pub last_time: i64,
    pub state: i32,
}

#[derive(Debug, Serialize)]
pub struct MessageListResponse {
    pub currentPage: u32,
    pub dataTotal: u32,
    pub list: Vec<MessageItem>,
    pub pageTotal: u32,
}

#[derive(Debug, Serialize)]
pub struct MessageContentItem {
    pub content: String,
    pub files: Vec<String>,
    pub date: String,
    pub state: i32,
}

#[derive(Debug, Serialize)]
pub struct MessageContentResponse {
    pub info: MessageContentItem,
}

// ========== 支付信息 ==========
#[derive(Debug, Serialize)]
pub struct PayInfo {
    pub order_no: String,
    pub money: i64,
    pub name: String,
    pub pay_url: String,
}

// ========== 签到信息 ==========
#[derive(Debug, Serialize)]
pub struct SignInInfo {
    pub fen: i32,
    pub fen_time: i64,
    pub fen_count: i32,
}

// ========== VIP信息 ==========
#[derive(Debug, Serialize)]
pub struct VipInfo {
    pub vip: i64,
    pub vip_time: i64,
}

// ========== 初始化信息 ==========
#[derive(Debug, Serialize)]
pub struct AppInfo {
    pub name: String,
    pub logo: String,
    pub reg_state: String,
    pub reg_off_msg: String,
    pub reg_way: String,
    pub logon_state: String,
    pub logon_off_msg: String,
}

#[derive(Debug, Serialize)]
pub struct IniResponse {
    pub info: AppInfo,
}