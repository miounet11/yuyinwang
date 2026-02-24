//! Recording King Library
//!
//! This library exposes the core functionality of Recording King
//! for use in integration tests and potential future library usage.

pub mod commands;
pub mod core;
pub mod services;

// Re-export commonly used types for convenience
pub use core::error::{AppError, Result};
pub use core::types::*;
