# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Recording King v7.0 is a Tauri-based desktop application providing AI-powered voice transcription with real-time audio processing, multiple model support (OpenAI, LuYinWang, local Whisper), and macOS text injection capabilities. The v7.0 rewrite reduced codebase by 80% while maintaining full functionality.

**Key Feature**: Global shortcut-triggered voice input with automatic text injection into any application (similar to macOS F5).

## Development Commands

### Frontend (React + TypeScript)
```bash
# Development
npm run dev                    # Vite dev server only
npm run tauri:dev             # Full Tauri app with hot reload

# Build
npm run build                 # Production frontend build
npm run tauri:build           # Complete app bundle (.dmg for macOS)

# Testing
npm run test                  # Vitest watch mode
npm run test:run              # Single test run
```

### Backend (Rust)
```bash
cd src-tauri

# Development
cargo build                   # Debug build
cargo build --release         # Optimized build

# Testing
cargo test                    # All tests
cargo test <test_name>        # Specific test
cargo test --lib              # Library tests only
cargo test --test <name>      # Integration test

# Run without Tauri
cargo run                     # Launch app directly
```

### Linting & Formatting
```bash
# Frontend
npm run lint                  # (if configured)

# Backend
cargo fmt                     # Format Rust code
cargo clippy                  # Lint checks
```

## Architecture

### Three-Layer Backend Structure

```
src-tauri/src/
├── main.rs              # Tauri app entry, system tray, window management
├── lib.rs               # Public library interface for tests
├── core/                # Business logic (no Tauri dependencies)
│   ├── audio.rs         # cpal recording, ringbuf circular buffer
│   ├── transcription.rs # Multi-provider routing (OpenAI/LuYin/Whisper)
│   ├── local_whisper.rs # whisper-rs Metal GPU inference
│   ├── injection.rs     # macOS Core Graphics text injection
│   ├── shortcuts.rs     # Global hotkey listener (rdev)
│   ├── global_listener.rs # Key event handling
│   ├── types.rs         # Shared data structures
│   └── error.rs         # Unified error handling
├── commands/            # Tauri IPC handlers (thin wrappers)
│   ├── recording.rs
│   ├── quick_input.rs
│   ├── settings.rs
│   ├── history.rs
│   ├── injection.rs
│   ├── models.rs
│   └── prompt_actions.rs
└── services/            # Stateful services
    ├── state.rs         # AppState with Arc<Mutex<T>> patterns
    ├── database.rs      # SQLite with r2d2 connection pooling
    └── quick_input.rs   # Orchestrates recording → transcription → injection
```

**Design Principle**: `core/` contains pure business logic testable without Tauri. `commands/` are thin IPC adapters. `services/` manage shared state.

### Frontend Structure

```
src/
├── App.tsx              # Main router, lazy-loaded pages
├── features/            # Feature-based modules
│   ├── recording/       # Recording UI
│   ├── settings/        # Settings panels
│   ├── shortcuts/       # Shortcut configuration
│   ├── models/          # Model management
│   ├── transcribe/      # File transcription
│   ├── history/         # History viewer
│   ├── ai-prompts/      # AI prompt actions
│   └── onboarding/      # First-run setup
└── shared/
    ├── stores/          # Zustand state management
    ├── components/      # Reusable components
    ├── hooks/           # Custom React hooks
    └── types.ts         # TypeScript definitions
```

### Critical Data Flow: Quick Voice Input

1. **Shortcut Press** → `core/shortcuts.rs` (rdev) detects key
2. **Start Recording** → `core/audio.rs` captures via cpal → ringbuf
3. **Shortcut Release** → Stop recording, get f32 samples
4. **Transcription** → `core/transcription.rs` routes to provider:
   - Local Whisper: Direct f32 → whisper-rs (Metal GPU)
   - Online APIs: f32 → WAV → HTTP upload
5. **Text Injection** → `core/injection.rs` uses Core Graphics to type text
6. **History Save** → `services/database.rs` stores entry

### State Management

**Backend**: `AppState` (services/state.rs)
- `Arc<Mutex<AppSettings>>`: Thread-safe settings
- `Arc<Database>`: Shared SQLite connection pool
- `mpsc::UnboundedSender<RecorderCommand>`: Audio thread communication

**Frontend**: Zustand store (shared/stores/useAppStore.ts)
- Settings, toasts, initialization state
- Direct Tauri IPC via `invoke()`

## Key Technical Details

### Audio Processing
- **Recording**: cpal → ringbuf (lock-free circular buffer) → f32 samples
- **Format**: 16kHz mono (resampled via rubato if needed)
- **Real-time**: Dedicated recorder thread to avoid blocking

### Transcription Providers
- **LuYinWang**: 3-step (upload → create task → poll)
- **OpenAI**: Direct multipart upload to `/audio/transcriptions`
- **Local Whisper**: whisper-rs with Metal acceleration (macOS GPU)
  - Models stored in `app_data_dir/models/`
  - Download via `commands/models.rs`

### macOS Integration
- **Text Injection**: Core Graphics CGEvent API (requires Accessibility permission)
- **Global Shortcuts**: rdev for system-wide key capture
- **Permissions**: Microphone + Accessibility checked at startup
- **System Tray**: Persistent tray icon with quick actions

### Database Schema (SQLite)
```sql
-- settings: JSON blob of AppSettings
-- transcriptions: id, text, timestamp, duration, model, confidence, audio_file_path
```

### Error Handling
- Custom `AppError` enum (core/error.rs)
- Unified `Result<T>` type throughout backend
- Frontend displays errors via toast notifications

## Testing Strategy

### Backend Tests
- Unit tests: `#[cfg(test)] mod <name>_test` in same file
- Integration tests: `tests/` directory
- Property-based: proptest for audio/transcription logic
- Run: `cargo test` (in src-tauri/)

### Frontend Tests
- Vitest + React Testing Library
- Property tests: fast-check
- Location: `src/**/*.test.tsx`
- Run: `npm run test:run`

## Common Development Tasks

### Adding a New Transcription Provider
1. Add variant to `ModelProvider` enum (core/types.rs)
2. Implement `transcribe_<provider>()` in `core/transcription.rs`
3. Update routing in `transcribe_audio()` and `transcribe_samples()`
4. Add API key field to `AppSettings` if needed
5. Update frontend model selection UI

### Adding a Tauri Command
1. Implement handler in `commands/<module>.rs`
2. Add to `invoke_handler![]` in `main.rs`
3. Call from frontend: `invoke('<command_name>', { args })`

### Modifying Settings
1. Update `AppSettings` struct (core/types.rs)
2. Add default value function if needed (for backward compatibility)
3. Update database save/load (services/database.rs)
4. Update frontend settings UI

### Working with Audio
- Audio recording runs in dedicated thread (services/state.rs)
- Use `RecorderCommand` enum for thread-safe communication
- Always resample to 16kHz before transcription
- Local Whisper expects f32 samples directly (no WAV conversion)

## Platform-Specific Notes

### macOS
- Primary target platform
- Requires entitlements: `com.apple.security.device.audio-input`, `com.apple.security.device.microphone`
- Text injection needs Accessibility permission (prompt at startup)
- Metal GPU acceleration for local Whisper models

### Windows/Linux
- Secondary support with graceful degradation
- Text injection may use different APIs (not fully implemented)
- Global shortcuts work via rdev (cross-platform)

## Performance Considerations

- **Startup Time**: ~1s (v7.0 optimization)
- **Memory**: ~80MB typical usage
- **Audio Latency**: Target <50ms for recording pipeline
- **Local Whisper**: GPU-accelerated, ~2-5s for 10s audio (tiny/base models)
- **Code Splitting**: Frontend uses lazy loading for all pages

## Build Configuration

### Release Profile (Cargo.toml)
```toml
[profile.release]
panic = "abort"
codegen-units = 1
lto = true
opt-level = "z"      # Size optimization
strip = true
```

### Tauri Features
- `global-shortcut-all`: System-wide hotkeys
- `system-tray`: Persistent tray icon
- `dialog-all`: File dialogs
- `fs-all`: File system access
- `shell-open`: Open external URLs

## Security

- API keys stored in SQLite (app data directory)
- No keys in code or version control
- CSP policies configured in tauri.conf.json
- IPC command validation in all handlers
- Accessibility permission required for text injection

## Troubleshooting

### "Recorder died" error
- Audio thread panicked, check device availability
- Restart app to reinitialize recorder thread

### Text injection not working
- Check Accessibility permission: System Preferences → Security & Privacy → Privacy → Accessibility
- Verify `auto_inject` setting is enabled

### Local Whisper model not found
- Models must be downloaded first via "听写模型" page
- Check `app_data_dir/models/` directory exists
- Model files: `ggml-tiny.bin`, `ggml-base.bin`, etc.

### Shortcut conflicts
- Check for conflicts with other apps
- Try different key combination
- Unregister and re-register shortcut

## Documentation References

- [Quick Input Guide](../docs/QUICK_INPUT_GUIDE.md) - Detailed usage
- [Features v7.0](../docs/FEATURES_v7.0.md) - Complete feature list
- [Development Guide](../docs/DEVELOPMENT_GUIDE.md) - Extended dev docs
- [Project Structure](../docs/PROJECT_STRUCTURE.md) - Detailed structure
