//! Profile handlers
//! Grouped: info, modifyName, modifyPic, setAcctno, setEmail, setExtend, setPhone, reEmail, rePhone

pub mod info;
pub mod modifyName;
pub mod modifyPic;
pub mod setAcctno;
pub mod setEmail;
pub mod setExtend;
pub mod setPhone;
pub mod reEmail;
pub mod rePhone;

// Re-export for backward compatibility
pub use info::*;
pub use modifyName::*;
pub use modifyPic::*;
pub use setAcctno::*;
pub use setEmail::*;
pub use setExtend::*;
pub use setPhone::*;
pub use reEmail::*;
pub use rePhone::*;
