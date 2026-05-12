// 通用结构体
use serde::{Serialize, Deserialize};

/// 设备信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub udid: String,
    pub time: i64,
}