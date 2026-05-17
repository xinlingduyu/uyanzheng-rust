//! Oauth handlers
//! Grouped: wxlogon, wxlogonCallback, wxlogonQuery, wxBindSDK, wxloginSDK, qqBindSDK, qqlogonCallback, qqloginWeb, qqloginQuery, qqloginSDK

pub mod qqBindSDK;
mod http_client;
pub mod qqloginQuery;
pub mod qqloginSDK;
pub mod qqloginWeb;
pub mod qqlogonCallback;
pub mod wxBindSDK;
pub mod wxloginSDK;
pub mod wxlogon;
pub mod wxlogonCallback;
pub mod wxlogonQuery;

// Re-export for backward compatibility
pub use qqBindSDK::*;
pub use qqloginQuery::*;
pub use qqloginSDK::*;
pub use qqloginWeb::*;
pub use qqlogonCallback::*;
pub use wxBindSDK::*;
pub use wxloginSDK::*;
pub use wxlogon::*;
pub use wxlogonCallback::*;
pub use wxlogonQuery::*;
