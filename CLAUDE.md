# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Recording King (Spokenly Clone)** is a real-time speech-to-text desktop application built with Tauri (Rust backend) and React (TypeScript frontend). It features AI-powered transcription, multi-model support, and advanced audio processing capabilities.

**Current Status**: v3.0.1, ~80% complete, actively developed

## Development Commands

### Frontend Development
```bash
# Install dependencies
npm install

# Development server (frontend only)
npm run dev

# Build frontend
npm run build
```

### Tauri Desktop App
```bash
# Development with hot reload
npm run tauri:dev

# Build production app
npm run tauri:build

# Tauri CLI commands
npm run tauri -- [command]
```

### Testing
```bash
# TypeScript compilation check
npx tsc --noEmit

# No unified test command configured - check individual test directories:
# tests/unit/ tests/integration/ tests/e2e/
```

## Architecture Overview

### High-Level Structure
- **Frontend**: React 18 + TypeScript + Zustand (state management)
- **Backend**: Rust + Tauri (cross-platform desktop framework)
- **Audio Processing**: CPAL (Cross-Platform Audio Library) + Whisper integration
- **AI Integration**: Multi-model support (GPT, Claude, Gemini, local Whisper)
- **Database**: SQLite via Rusqlite for transcription history

### Key Directories
```
src/                    # React frontend
├── components/         # React components (UI pages and widgets)
├── data/              # Static data and model configurations
├── utils/             # Frontend utilities and managers
└── stores/            # Zustand state management

src-tauri/             # Rust backend
├── src/
│   ├── main.rs        # Main application entry (28k+ lines - complex)
│   ├── audio/         # Audio recording and processing
│   ├── api/           # External API integrations
│   └── whisper/       # Local Whisper model integration
└── Cargo.toml         # Rust dependencies
```

### Core Modules

**Frontend State Management (Zustand)**:
- Main app state in `src/App.tsx` (~2000+ lines)
- Audio devices, transcription history, AI prompts
- Recording state, model selection, permissions

**Backend Systems**:
- `audio_recorder.rs`: Real-time audio capture with CPAL
- `ai_agent.rs`: Multi-model AI processing chains
- `database.rs`: SQLite transcription history storage
- `whisper/`: Local Whisper model integration

### Critical Integration Points

**Tauri IPC Commands**: Frontend calls Rust backend via `invoke()`:
- Audio device management
- Recording start/stop
- File transcription
- AI agent processing
- Permission checks (macOS focus)

**Permission System**: macOS-focused accessibility and microphone permissions with runtime checks and user guidance.

## Development Patterns

### Code Organization
- **Component-per-file**: Each React component has dedicated `.tsx` and `.css` files
- **Modular Rust**: Backend organized into feature modules
- **Type Safety**: Comprehensive TypeScript usage with shared interfaces

### State Management
- **Zustand Store**: Single store pattern in main App component
- **Rust State**: Arc<Mutex<T>> for thread-safe shared state
- **IPC Communication**: Structured message passing between frontend/backend

### AI Model Integration
- **Multi-Provider Support**: OpenAI, Anthropic, Google, local models
- **Agent Chains**: Complex multi-step AI processing workflows
- **Model Switching**: Runtime model selection with configuration persistence

## Important Implementation Notes

### Audio Processing
- Uses CPAL for cross-platform audio capture
- WAV format for local storage and processing
- Real-time audio streaming to Whisper models

### Platform-Specific Code
- macOS permission handling with Objective-C bridging
- System tray integration with platform-specific menus
- Global shortcut management

### Performance Considerations
- Large main.rs file (28k+ lines) - consider refactoring for maintainability
- Audio processing runs in separate threads
- Database operations are async with Tokio runtime

### Security & Permissions
- Comprehensive macOS permission system
- API key management for AI services
- No hardcoded credentials (environment/config driven)

## Key Dependencies

**Frontend**:
- React 18, TypeScript 5.0
- Zustand 4.4 (state management)
- @tauri-apps/api (IPC communication)
- Radix UI components

**Backend**:
- Tauri 1.5, Tokio async runtime
- CPAL 0.15 (audio), Whisper-rs (speech recognition)
- Rusqlite (database), Reqwest (HTTP client)
- serde/serde_json (serialization)

## Project Management

**IMPORTANT**: Always check `PROJECT_MANAGEMENT.md` before starting development work to understand current status and avoid duplicate implementation.

Key principles:
- Never reimplement existing functionality
- Update PROJECT_MANAGEMENT.md after completing features
- Maintain 80%+ completion status accuracy
- Follow modular development with integration testing

## Configuration Files

- `tauri.conf.json`: Tauri app configuration and permissions
- `vite.config.ts`: Frontend build configuration (port 1420)
- `Cargo.toml`: Rust dependencies and features
- `tsconfig.json`: TypeScript compilation settings (strict: false)