pub mod text_injector;
pub mod permission_manager;
pub mod optimized_text_injector;

pub use text_injector::*;
pub use permission_manager::*;
pub use optimized_text_injector::{SmartTextInjector, OptimizedTextInjectionConfig};

// Re-export commonly used types for convenience
pub use text_injector::ApplicationInfo as AppInfo;