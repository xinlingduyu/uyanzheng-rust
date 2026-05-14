//! 支付插件模块
//! 支持热插拔的支付插件系统

pub mod ali;
pub mod http_client;
pub mod jie;
pub mod manager;
pub mod trait_def;
pub mod wx;

pub use ali::AliPayPlugin;
pub use jie::JiePayPlugin;
pub use trait_def::{NotifyVerifyResult, PayOrder, PayPlugin, PayResult};
pub use wx::WxPayPlugin;
