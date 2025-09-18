// Story 1.4: Network Detection and Mode Switching Module

pub mod network_monitor;
pub mod transcription_mode_manager;

pub use network_monitor::{NetworkMonitor, NetworkStatus};
pub use transcription_mode_manager::{TranscriptionMode, TranscriptionModeManager};
