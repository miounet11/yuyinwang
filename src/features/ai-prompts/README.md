# AI Prompts Feature

This feature provides AI-powered prompt management with automated actions for the Recording King application.

## Overview

The AI Prompts feature allows users to:
- Create custom AI prompts with keyboard shortcuts
- Define automated actions (Google search, launch apps, ChatGPT/Claude queries, etc.)
- Configure advanced AI settings (model, temperature, max tokens)
- Execute actions with confirmation dialogs for sensitive operations (shell commands)

## Components

### AIPromptsPage

Main page component that displays the list of AI prompts and manages their lifecycle.

**Location**: `/Users/lu/Documents/yuyinwang/src/features/ai-prompts/AIPromptsPage.tsx`

**Features**:
- Display all configured AI prompts in a card layout
- Enable/disable prompts with toggle switches
- Edit and delete prompts
- Execute prompt actions with visual feedback
- Shell command confirmation dialog for security
- Empty state with call-to-action

**State Management**:
- Uses `useAppStore` for global state (aiPrompts, addAIPrompt, updateAIPrompt, deleteAIPrompt)
- Local state for modal visibility and pending actions

### EditPromptModal

Step-by-step modal for creating and editing AI prompts.

**Location**: `/Users/lu/Documents/yuyinwang/src/features/ai-prompts/EditPromptModal.tsx`

**Features**:
- Two-step wizard interface:
  - Step 1: Configure activation (name, keyboard shortcut)
  - Step 2: Define AI instruction and actions
- Keyboard shortcut recording with real-time validation
- Action button grid with 10 action types
- Advanced settings panel (collapsible)
- Form validation before saving

**Action Types**:
1. Google Search - Search query on Google
2. Launch App - Open macOS application
3. Close App - Close macOS application
4. ChatGPT - Send prompt to ChatGPT
5. Claude - Send prompt to Claude
6. YouTube Search - Search on YouTube
7. Open Website - Open URL in browser
8. Apple Shortcut - Execute Apple Shortcuts
9. Shell Command - Execute terminal command (requires confirmation)
10. Keypress - Simulate keyboard input

## Data Types

All types are defined in `/Users/lu/Documents/yuyinwang/src/shared/types.ts`:

```typescript
interface AIPrompt {
  id: string;
  name: string;
  shortcut?: CustomShortcut;
  instruction: string;
  actions: PromptAction[];
  advancedSettings?: AdvancedPromptSettings;
  enabled: boolean;
}

type PromptAction =
  | { type: 'google-search'; query: string }
  | { type: 'launch-app'; appName: string }
  | { type: 'close-app'; appName: string }
  | { type: 'ask-chatgpt'; prompt: string }
  | { type: 'ask-claude'; prompt: string }
  | { type: 'youtube-search'; query: string }
  | { type: 'open-website'; url: string }
  | { type: 'apple-shortcut'; shortcutName: string }
  | { type: 'shell-command'; command: string }
  | { type: 'keypress'; keys: string };

interface AdvancedPromptSettings {
  model?: string;
  temperature?: number;
  maxTokens?: number;
}
```

## Store Integration

The feature integrates with Zustand store (`useAppStore`):

```typescript
// State
aiPrompts: AIPrompt[];

// Actions
addAIPrompt: (prompt: AIPrompt) => Promise<void>;
updateAIPrompt: (id: string, updates: Partial<AIPrompt>) => Promise<void>;
deleteAIPrompt: (id: string) => Promise<void>;
```

## Backend Integration

The following Tauri commands need to be implemented in the Rust backend:

```rust
#[tauri::command]
async fn save_ai_prompts(prompts: Vec<AIPrompt>) -> Result<(), String> {
    // Save prompts to local storage/database
}

#[tauri::command]
async fn execute_prompt_action(prompt_id: String, action: PromptAction) -> Result<String, String> {
    // Execute the specified action
    // Return result or error message
}
```

## Styling

Styles follow the Spokenly design system with dark theme:

- **AIPromptsPage.css**: Main page styles (cards, buttons, grid layouts)
- **EditPromptModal.css**: Modal styles (steps, forms, advanced settings)

**CSS Variables Used**:
- `--accent`: Primary accent color (#3b82f6)
- `--bg-primary`: Main background (#1a1a1a)
- `--bg-secondary`: Secondary background (#1e1e1e)
- `--bg-tertiary`: Tertiary background (#2a2a2a)
- `--border`: Border color (#3a3a3a)
- `--text-primary`: Primary text (#fff)
- `--text-secondary`: Secondary text (#ccc)
- `--text-muted`: Muted text (#888)
- `--danger`: Danger color (#ef4444)
- `--success`: Success color (#10b981)

## Security Considerations

1. **Shell Command Confirmation**: Shell commands require explicit user confirmation before execution
2. **XSS Prevention**: All user input is escaped using `escapeHtml()` utility
3. **Validation**: Keyboard shortcuts are validated to prevent conflicts with system shortcuts
4. **Error Handling**: All actions have try-catch blocks with user-friendly error messages

## Usage Example

```typescript
import { AIPromptsPage } from './features/ai-prompts';

// In your router/App component
{currentPage === 'ai-prompts' && <AIPromptsPage />}
```

## Future Enhancements

1. **Action Execution**: Implement backend handlers for all action types
2. **Retry Logic**: Add retry/skip options for failed actions
3. **Action Chaining**: Support sequential execution of multiple actions
4. **Templates**: Provide pre-built prompt templates
5. **Import/Export**: Allow users to share prompts
6. **Analytics**: Track prompt usage and success rates
7. **AI Integration**: Connect to actual AI APIs (OpenAI, Anthropic)
8. **Voice Activation**: Trigger prompts via voice commands

## Testing

To test the components:

```bash
# Run the development server
npm run dev

# Navigate to AI Prompts page
# Click "+ 添加提示" to create a new prompt
# Test keyboard shortcut recording
# Add various action types
# Test enable/disable toggle
# Test edit and delete operations
```

## Troubleshooting

**Issue**: Keyboard shortcuts not recording
- **Solution**: Ensure the input field has focus and you're pressing modifier keys + regular key

**Issue**: Actions not executing
- **Solution**: Check that backend Tauri commands are implemented and registered

**Issue**: Styles not applying
- **Solution**: Verify CSS files are imported and CSS variables are defined in App.css

## Related Files

- `/Users/lu/Documents/yuyinwang/src/shared/types.ts` - Type definitions
- `/Users/lu/Documents/yuyinwang/src/shared/stores/useAppStore.ts` - State management
- `/Users/lu/Documents/yuyinwang/src/shared/utils.ts` - Utility functions
- `/Users/lu/Documents/yuyinwang/src/App.tsx` - Main app integration
- `/Users/lu/Documents/yuyinwang/.kiro/specs/spokenly-ui-redesign/design.md` - Design specifications
