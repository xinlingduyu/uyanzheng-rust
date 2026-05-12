//! App management handlers
//! Auto-organized module - app, send, pay

pub mod app;
pub mod send;
pub mod pay;

// Re-export all public items for backward compatibility
pub use app::*;
pub use send::*;
pub use pay::*;
