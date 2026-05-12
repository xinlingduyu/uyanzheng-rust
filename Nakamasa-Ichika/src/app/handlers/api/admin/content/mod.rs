//! Content management handlers
//! Auto-organized module - notice, message

pub mod notice;
pub mod message;

// Re-export all public items for backward compatibility
pub use notice::*;
pub use message::*;
