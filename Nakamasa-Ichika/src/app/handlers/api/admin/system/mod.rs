//! System management handlers
//! Auto-organized module - system, set, dict, encryption, extend

pub mod dict;
pub mod encryption;
pub mod extend;
pub mod set;
pub mod system;

// Re-export all public items for backward compatibility
pub use dict::*;
pub use encryption::*;
pub use extend::*;
pub use set::*;
pub use system::*;
