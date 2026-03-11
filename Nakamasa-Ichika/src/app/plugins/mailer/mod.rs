//! 邮件发送插件模块
//! 支持热插拔的邮件发送系统

pub mod trait_def;
pub mod smtp;

pub use trait_def::{MailerPlugin, MailerConfig, MailResult, MailerPluginMeta};
pub use smtp::SmtpMailer;
