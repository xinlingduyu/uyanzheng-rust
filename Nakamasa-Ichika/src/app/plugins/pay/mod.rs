//! 支付插件模块
//! 支持热插拔的支付插件系统

pub mod trait_def;
pub mod manager;
pub mod jie;
pub mod ali;
pub mod wx;
pub mod http_client;

pub use trait_def::{PayPlugin, PayOrder, PayResult};
pub use jie::JiePayPlugin;
pub use ali::AliPayPlugin;
pub use wx::WxPayPlugin;