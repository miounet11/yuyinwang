pub mod text_injector;
pub mod permission_manager;

pub use text_injector::*;
pub use permission_manager::*;

// Re-export commonly used types for convenience
pub use text_injector::ApplicationInfo as AppInfo;