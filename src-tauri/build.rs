fn main() {
    // whisper.cpp 需要 macOS 10.15+ 的 std::filesystem
    #[cfg(target_os = "macos")]
    if std::env::var("MACOSX_DEPLOYMENT_TARGET").is_err() {
        std::env::set_var("MACOSX_DEPLOYMENT_TARGET", "10.15");
    }

    tauri_build::build()
}
