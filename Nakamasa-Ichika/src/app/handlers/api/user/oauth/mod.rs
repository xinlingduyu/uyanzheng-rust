//! Oauth handlers
//! Grouped: wxlogon, wxlogonCallback, wxlogonQuery, wxBindSDK, wxloginSDK, qqBindSDK, qqlogonCallback, qqloginWeb, qqloginQuery, qqloginSDK

pub mod wxlogon;
pub mod wxlogonCallback;
pub mod wxlogonQuery;
pub mod wxBindSDK;
pub mod wxloginSDK;
pub mod qqBindSDK;
pub mod qqlogonCallback;
pub mod qqloginWeb;
pub mod qqloginQuery;
pub mod qqloginSDK;

// Re-export for backward compatibility
pub use wxlogon::*;
pub use wxlogonCallback::*;
pub use wxlogonQuery::*;
pub use wxBindSDK::*;
pub use wxloginSDK::*;
pub use qqBindSDK::*;
pub use qqlogonCallback::*;
pub use qqloginWeb::*;
pub use qqloginQuery::*;
pub use qqloginSDK::*;
