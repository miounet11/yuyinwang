pub mod permission_manager;
pub mod text_injector;
pub mod unified_permission_manager_simple;

pub use permission_manager::*;
pub use text_injector::*;
pub use unified_permission_manager_simple::*;

// Re-export commonly used types for convenience
pub use text_injector::ApplicationInfo as AppInfo;
