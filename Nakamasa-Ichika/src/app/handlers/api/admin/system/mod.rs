//! System management handlers
//! Auto-organized module - system, set, dict, encryption, extend

pub mod system;
pub mod set;
pub mod dict;
pub mod encryption;
pub mod extend;

// Re-export all public items for backward compatibility
pub use system::*;
pub use set::*;
pub use dict::*;
pub use encryption::*;
pub use extend::*;
