# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Recording King is a Tauri-based desktop application for AI-powered speech-to-text transcription. It features real-time recording, multiple AI model support, global shortcuts, and macOS system integration.

## Architecture

### Core Technology Stack
- **Frontend**: React + TypeScript + Vite
- **Backend**: Rust + Tauri
- **UI Framework**: Tailwind CSS + Radix UI components
- **State Management**: Zustand
- **Database**: SQLite (via rusqlite)
- **Audio Processing**: cpal, whisper-rs

### Key Architectural Patterns
- **Modular Rust Backend**: Commands, services, and managers are separated into focused modules
- **Command Pattern**: All frontend-backend communication uses Tauri commands
- **State Management**: Centralized app state in `AppState` struct with Arc<Mutex<T>> for thread safety
- **Database Layer**: Abstracted through `DatabaseManager` and `HistoryManager`

## Development Commands

### Essential Commands
```bash
# Development
npm run tauri:dev          # Start development server with hot reload
npm run dev               # Frontend only development

# Building
npm run build             # Build frontend
npm run tauri:build       # Build complete application
cargo check              # Quick Rust syntax check (in src-tauri/)
cargo build               # Build Rust backend only
```

### Testing & Validation
```bash
# The project follows these validation patterns:
# - Frontend: No specific test runner configured
# - Backend: Use `cargo test` in src-tauri/ directory
# - Integration: Manual testing through the app interface
```

## Code Organization

### Frontend Structure (`src/`)
- `components/` - All React components (UI, settings, recording controls)
- `stores/` - Zustand stores for state management
- `utils/` - Utility functions and helpers
- `types/` - TypeScript type definitions
- `hooks/` - Custom React hooks

### Backend Structure (`src-tauri/src/`)
- `commands/` - Tauri command handlers (API layer)
- `audio/` - Audio recording and processing logic
- `database/` - Database management and models
- `transcription/` - AI transcription services
- `ai_agent/` - AI agent processing pipeline
- `system/` - System integration (permissions, text injection)
- `shortcuts/` - Global shortcut management

## Key Components and Services

### Audio System
- **AudioRecorder** (`audio/recorder.rs`) - Core recording functionality
- **AudioDeviceManager** (`audio/devices.rs`) - Device enumeration and management
- **RealtimeStreamer** (`audio/realtime_streamer.rs`) - Streaming audio processing

### Transcription Pipeline
- **TranscriptionService** (`transcription/service.rs`) - API client wrapper
- **WhisperService** (`transcription/whisper.rs`) - Local Whisper model integration
- **TranscriptionEditor** (`transcription/editor.rs`) - Text editing and manipulation

### State Management
- **AppState** (`main.rs`) - Central application state
- **Settings** (`config/settings.rs`) - Configuration management
- **Database Integration** - SQLite with migrations support

## Development Guidelines

### Adding New Features
1. **Always check PROJECT_MANAGEMENT.md first** - Never duplicate existing functionality
2. **Frontend**: Create components in `src/components/`, add types to `src/types/`
3. **Backend**: Add commands to appropriate module in `src-tauri/src/commands/`
4. **Database**: Add migrations to `src-tauri/src/database/migrations.rs`

### Command Handler Pattern
All backend functionality is exposed through Tauri commands:
```rust
#[tauri::command]
pub async fn your_command(
    app_state: tauri::State<'_, AppState>,
    // parameters
) -> Result<ResponseType, String> {
    // implementation
}
```

### Error Handling
- Use `AppResult<T>` and `AppError` types for consistent error handling
- Frontend commands should return `Result<T, String>` for Tauri compatibility
- Log errors appropriately for debugging

### macOS Integration Specifics
- **Permissions**: Check accessibility, microphone, and file system permissions
- **Global Shortcuts**: Use `rdev` for cross-platform key listening
- **Text Injection**: System integration through `objc` and `cocoa` crates
- **Floating Windows**: Specialized window management for voice input

## Important Files

### Configuration
- `src-tauri/tauri.conf.json` - Tauri app configuration and permissions
- `src-tauri/Cargo.toml` - Rust dependencies and features
- `package.json` - Frontend dependencies and scripts

### Core Modules
- `src-tauri/src/main.rs` - Application entry point and state setup
- `src-tauri/src/commands/mod.rs` - Command registration and exports
- `src/App.tsx` - Main React application component

### State and Storage
- App settings stored in system directories (macOS: `~/Library/Application Support/spokenly-clone`)
- Database: SQLite file in app data directory
- Audio files: Managed through Tauri's file system API

## Tauri-Specific Considerations

### Security Model
- CSP configured for API access to OpenAI, Deepgram, and other services
- File system access limited to specific directories via allowlist
- Commands must be explicitly registered in `invoke_handler!`

### Window Management
- Main window: Primary application interface
- Floating input: Overlay for voice input (created dynamically)
- System tray: Background application management

### Build Configuration
- Cross-platform builds supported (macOS, Windows, Linux)
- macOS: Requires code signing for distribution
- Bundle identifier: `com.recordingking.app`

## Common Development Patterns

### Adding a New Command
1. Implement in appropriate `commands/` module
2. Add to `invoke_handler!` in `main.rs`
3. Create corresponding frontend function in utilities
4. Add TypeScript types if needed

### Database Changes
1. Add migration to `migrations.rs`
2. Update models in `models.rs`
3. Test migration with existing data

### UI Components
1. Use Tailwind for styling consistency
2. Follow existing component patterns
3. Implement proper error boundaries
4. Add to appropriate store if state is needed

## Performance Considerations

- Audio processing runs in separate threads
- Database operations use connection pooling
- Frontend state updates are optimized with Zustand
- Large file operations use streaming where possible

## Security Notes

- API keys stored in secure system locations
- Input validation on all command parameters
- File access restricted through Tauri allowlist
- No direct shell access - use Tauri APIs