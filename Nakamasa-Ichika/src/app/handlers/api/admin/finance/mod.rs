//! Finance management handlers
//! Auto-organized module - goods, order, statistics, fen_event, fen_order

pub mod goods;
pub mod order;
pub mod statistics;
pub mod fen_event;
pub mod fen_order;

// Re-export all public items for backward compatibility
pub use goods::*;
pub use order::*;
pub use statistics::*;
pub use fen_event::*;
pub use fen_order::*;
