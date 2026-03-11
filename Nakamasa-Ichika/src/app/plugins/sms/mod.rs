//! 短信插件模块
//! 支持热插拔的短信插件系统

pub mod trait_def;
pub mod manager;
pub mod jie;
pub mod ali;
pub mod tencent;

pub use trait_def::SmsPlugin;
pub use jie::JieSmsPlugin;
pub use ali::AliSmsPlugin;
pub use tencent::TencentSmsPlugin;
