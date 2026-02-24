# AI Prompt Actions - Implementation Guide

## Overview

This module implements the AI action execution engine for Recording King, allowing the application to execute various system actions based on AI prompts and voice commands.

## Implemented Actions

### Priority Actions (P0 - Fully Implemented)

#### 1. Google Search
```typescript
await invoke('execute_prompt_action', {
  action: { type: 'google-search', query: 'Tauri framework' }
});
```
- Opens Google search in default browser
- URL-encodes query parameters
- Cross-platform support (macOS/Windows/Linux)

#### 2. YouTube Search
```typescript
await invoke('execute_prompt_action', {
  action: { type: 'youtube-search', query: 'Rust programming' }
});
```
- Opens YouTube search in default browser
- URL-encodes query parameters
- Cross-platform support

#### 3. Open Website
```typescript
await invoke('execute_prompt_action', {
  action: { type: 'open-website', url: 'https://tauri.app' }
});
```
- Opens any URL in default browser
- Validates URL format (must start with http:// or https://)
- Blocks dangerous protocols (javascript:, file:, etc.)
- Cross-platform support

#### 4. Launch Application
```typescript
await invoke('execute_prompt_action', {
  action: { type: 'launch-app', appName: 'Safari' }
});
```
- Launches applications by name
- Platform-specific implementations:
  - **macOS**: Uses `open -a AppName`
  - **Windows**: Uses `start AppName`
  - **Linux**: Tries `xdg-open`, `gnome-open`, `kde-open`
- Security: Blocks path traversal attacks (../, /, \\)

#### 5. Close Application
```typescript
await invoke('execute_prompt_action', {
  action: { type: 'close-app', appName: 'Safari' }
});
```
- Closes running applications gracefully
- Platform-specific implementations:
  - **macOS**: Uses AppleScript `tell application to quit`
  - **Windows**: Uses `taskkill /IM AppName.exe /F`
  - **Linux**: Uses `pkill -f AppName`
- Security: Blocks path traversal attacks

### Additional Actions (Implemented)

#### 6. Apple Shortcuts (macOS only)
```typescript
await invoke('execute_prompt_action', {
  action: { type: 'apple-shortcut', shortcutName: 'My Shortcut' }
});
```
- Executes macOS Shortcuts via `shortcuts run` command
- Returns shortcut output
- Only available on macOS
- Security: Blocks path traversal attacks

#### 7. Shell Command (Advanced)
```typescript
await invoke('execute_prompt_action', {
  action: { type: 'shell-command', command: 'echo hello' }
});
```
- Executes arbitrary shell commands
- **CRITICAL SECURITY**: Implements strict dangerous command detection
- Blocked patterns include:
  - `rm -rf`, `sudo`, `chmod`, `chown`
  - `dd if=`, `mkfs`, `format`
  - Pipe to shell: `| sh`, `| bash`
  - System files: `/etc/passwd`, `/etc/shadow`
  - System control: `shutdown`, `reboot`, `systemctl`
  - And more...
- Optional whitelist validation (currently disabled)

### Not Yet Implemented (P2)

#### 8. Ask ChatGPT
```typescript
await invoke('execute_prompt_action', {
  action: { type: 'ask-chatgpt', prompt: 'Explain Rust ownership' }
});
```
- Status: TODO
- Requires OpenAI API integration

#### 9. Ask Claude
```typescript
await invoke('execute_prompt_action', {
  action: { type: 'ask-claude', prompt: 'Write a function' }
});
```
- Status: TODO
- Requires Anthropic API integration

#### 10. Keypress Simulation
```typescript
await invoke('execute_prompt_action', {
  action: { type: 'keypress', keys: 'Command+C' }
});
```
- Status: TODO
- Requires keyboard simulation library (e.g., enigo)

## Security Features

### 1. Path Traversal Protection
- All app names and shortcut names are validated
- Blocks: `../`, `/`, `\\` characters
- Prevents access to system files and directories

### 2. URL Validation
- Only allows `http://` and `https://` protocols
- Blocks dangerous protocols:
  - `javascript:` (XSS attacks)
  - `file://` (local file access)
  - `ftp://` (file transfer)
  - `data:` (data URIs)

### 3. Dangerous Command Detection
Comprehensive blacklist of dangerous shell commands:
- File system destruction: `rm -rf`, `dd`, `mkfs`
- Privilege escalation: `sudo`, `chmod`, `chown`
- Remote code execution: `curl | sh`, `wget | bash`
- System control: `shutdown`, `reboot`, `systemctl`
- Process termination: `kill -9`, `killall`

### 4. Optional Whitelist (Disabled by Default)
- Can be enabled for stricter security
- Only allows pre-approved commands
- Example whitelist: `echo`, `ls`, `pwd`, `date`

## Cross-Platform Support

### macOS
- ✅ Google Search
- ✅ YouTube Search
- ✅ Open Website
- ✅ Launch App (`open -a`)
- ✅ Close App (AppleScript)
- ✅ Apple Shortcuts
- ✅ Shell Commands

### Windows
- ✅ Google Search
- ✅ YouTube Search
- ✅ Open Website
- ✅ Launch App (`start`)
- ✅ Close App (`taskkill`)
- ❌ Apple Shortcuts (N/A)
- ✅ Shell Commands

### Linux
- ✅ Google Search
- ✅ YouTube Search
- ✅ Open Website
- ✅ Launch App (`xdg-open`, `gnome-open`, `kde-open`)
- ✅ Close App (`pkill`)
- ❌ Apple Shortcuts (N/A)
- ✅ Shell Commands

## Error Handling

All actions return `Result<String, AppError>` with descriptive error messages:

```rust
Ok("Opened Google search for: Tauri framework")
Err(AppError::Permission("Dangerous command detected"))
Err(AppError::Other("Failed to launch app: Application not found"))
```

## Testing

The module includes comprehensive unit tests:

```bash
cargo test --bin recording-king prompt_actions
```

Tests cover:
- ✅ Dangerous command detection
- ✅ URL validation
- ✅ Path traversal protection

## Usage Example

```typescript
import { invoke } from '@tauri-apps/api';

// Execute a Google search
try {
  const result = await invoke<string>('execute_prompt_action', {
    action: {
      type: 'google-search',
      query: 'Tauri framework'
    }
  });
  console.log(result); // "Opened Google search for: Tauri framework"
} catch (error) {
  console.error('Action failed:', error);
}

// Launch an application
try {
  const result = await invoke<string>('execute_prompt_action', {
    action: {
      type: 'launch-app',
      appName: 'Safari'
    }
  });
  console.log(result); // "Launched app: Safari"
} catch (error) {
  console.error('Failed to launch app:', error);
}

// Execute shell command (with security validation)
try {
  const result = await invoke<string>('execute_prompt_action', {
    action: {
      type: 'shell-command',
      command: 'echo "Hello World"'
    }
  });
  console.log(result);
} catch (error) {
  console.error('Command blocked:', error);
}
```

## Integration with AI Prompts

This module is designed to work with the AI prompt system defined in `BACKEND_INTEGRATION.md`:

```typescript
interface AIPrompt {
  id: string;
  name: string;
  instruction: string;
  actions: PromptAction[];
  enabled: boolean;
}

// Example: Search prompt
const searchPrompt: AIPrompt = {
  id: '1',
  name: 'Search Google',
  instruction: 'Search on Google',
  actions: [
    {
      type: 'google-search',
      query: '{{transcription}}' // Will be replaced with actual transcription
    }
  ],
  enabled: true
};

// Execute the action
for (const action of searchPrompt.actions) {
  await invoke('execute_prompt_action', { action });
}
```

## Future Enhancements

### P1 - High Priority
1. Implement ChatGPT integration (`ask-chatgpt`)
2. Implement Claude integration (`ask-claude`)
3. Add progress events for long-running actions

### P2 - Medium Priority
4. Implement keypress simulation (`keypress`)
5. Add action history and logging
6. Implement action chaining and workflows

### P3 - Low Priority
7. Add configurable whitelist for shell commands
8. Implement action rate limiting
9. Add action permissions system
10. Support for custom action plugins

## Dependencies

- `serde` - Serialization/deserialization
- `urlencoding` - URL encoding for search queries
- `std::process::Command` - System command execution

## File Structure

```
src-tauri/src/commands/
├── prompt_actions.rs          # Main implementation
├── mod.rs                     # Module exports
└── PROMPT_ACTIONS_README.md   # This file
```

## Related Documentation

- [Backend Integration Guide](../../../.kiro/specs/spokenly-ui-redesign/BACKEND_INTEGRATION.md)
- [Project README](../../../CLAUDE.md)
- [Tauri Command Documentation](https://tauri.app/v1/guides/features/command)

---

**Status**: ✅ Core functionality implemented and tested
**Last Updated**: 2026-02-24
**Version**: 1.0.0
