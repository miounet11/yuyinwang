# AI Prompt Actions - Quick Start Guide

## 🚀 What Was Implemented

AI action execution engine for Recording King with 5 priority actions + 2 bonus actions.

## 📁 Files Created

```
✅ src-tauri/src/commands/prompt_actions.rs (12KB)
   - Core implementation with security features
   - 7 working actions, 3 planned
   - Unit tests included

✅ src-tauri/src/commands/PROMPT_ACTIONS_README.md (8.2KB)
   - Technical documentation
   - API reference
   - Security features

✅ PROMPT_ACTIONS_INTEGRATION_EXAMPLE.md (12KB)
   - Frontend integration guide
   - TypeScript examples
   - React hooks

✅ PROMPT_ACTIONS_IMPLEMENTATION_SUMMARY.md (9.4KB)
   - Complete implementation summary
   - Test results
   - Next steps
```

## 📝 Files Modified

```
✅ src-tauri/Cargo.toml
   + Added: urlencoding = "2.1"

✅ src-tauri/src/commands/mod.rs
   + Added: pub mod prompt_actions;

✅ src-tauri/src/main.rs
   + Registered: execute_prompt_action command
```

## ✅ Working Actions

### 1. Google Search
```typescript
await invoke('execute_prompt_action', {
  action: { type: 'google-search', query: 'Tauri' }
});
```

### 2. YouTube Search
```typescript
await invoke('execute_prompt_action', {
  action: { type: 'youtube-search', query: 'Rust tutorial' }
});
```

### 3. Open Website
```typescript
await invoke('execute_prompt_action', {
  action: { type: 'open-website', url: 'https://tauri.app' }
});
```

### 4. Launch App
```typescript
await invoke('execute_prompt_action', {
  action: { type: 'launch-app', appName: 'Safari' }
});
```

### 5. Close App
```typescript
await invoke('execute_prompt_action', {
  action: { type: 'close-app', appName: 'Safari' }
});
```

### 6. Apple Shortcuts (macOS)
```typescript
await invoke('execute_prompt_action', {
  action: { type: 'apple-shortcut', shortcutName: 'My Shortcut' }
});
```

### 7. Shell Command (Advanced)
```typescript
await invoke('execute_prompt_action', {
  action: { type: 'shell-command', command: 'echo hello' }
});
```

## ⏳ Planned Actions

- `ask-chatgpt` - Requires OpenAI API
- `ask-claude` - Requires Anthropic API
- `keypress` - Requires keyboard simulation library

## 🔒 Security Features

✅ Path traversal protection
✅ URL validation (http/https only)
✅ Dangerous command detection (25+ patterns)
✅ Optional whitelist support

## 🧪 Testing

```bash
# Run tests
cd src-tauri
cargo test --bin recording-king prompt_actions

# Build release
cargo build --release
```

**Status**: ✅ All tests passing, build successful

## 🌍 Cross-Platform Support

| Action | macOS | Windows | Linux |
|--------|-------|---------|-------|
| All basic actions | ✅ | ✅ | ✅ |
| Apple Shortcuts | ✅ | ❌ | ❌ |

## 📚 Documentation

1. **Technical Docs**: `src-tauri/src/commands/PROMPT_ACTIONS_README.md`
2. **Frontend Guide**: `PROMPT_ACTIONS_INTEGRATION_EXAMPLE.md`
3. **Implementation Summary**: `PROMPT_ACTIONS_IMPLEMENTATION_SUMMARY.md`
4. **This Quick Start**: `PROMPT_ACTIONS_QUICK_START.md`

## 🎯 Next Steps

### For Backend
1. Implement `save_ai_prompts` command
2. Implement `load_ai_prompts` command
3. Add ChatGPT/Claude API integration

### For Frontend
1. Add TypeScript types from integration example
2. Create AI prompt management UI
3. Integrate with voice transcription workflow
4. Add error handling and user feedback

## 💡 Quick Example

```typescript
import { invoke } from '@tauri-apps/api';

// Simple usage
async function searchGoogle(query: string) {
  try {
    const result = await invoke<string>('execute_prompt_action', {
      action: { type: 'google-search', query }
    });
    console.log('✅', result);
  } catch (error) {
    console.error('❌', error);
  }
}

// With AI prompt
const prompt = {
  id: '1',
  name: 'Search Google',
  instruction: 'Search on Google',
  actions: [
    { type: 'google-search', query: '{{transcription}}' }
  ],
  enabled: true
};

// Execute with transcription
const transcription = 'Tauri framework';
for (const action of prompt.actions) {
  const processedAction = JSON.parse(
    JSON.stringify(action).replace('{{transcription}}', transcription)
  );
  await invoke('execute_prompt_action', { action: processedAction });
}
```

## 🐛 Troubleshooting

### Action fails with "Permission denied"
- Check system permissions (Accessibility, etc.)
- Verify app has necessary entitlements

### Action fails with "Dangerous command detected"
- Command contains blocked patterns
- Review security documentation
- Consider using safer alternatives

### Action fails with "Invalid URL"
- URL must start with http:// or https://
- Check for typos in URL

### App launch fails
- Verify app name is correct
- Check app is installed
- Try full app name (e.g., "Google Chrome" not "Chrome")

## 📞 Support

For questions or issues:
1. Check documentation files listed above
2. Review BACKEND_INTEGRATION.md for API specs
3. Check inline code comments in prompt_actions.rs

---

**Status**: ✅ Production Ready
**Version**: 1.0.0
**Last Updated**: 2026-02-24
