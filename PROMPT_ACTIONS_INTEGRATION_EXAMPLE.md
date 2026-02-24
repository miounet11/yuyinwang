# AI Prompt Actions - Frontend Integration Example

## Quick Start

The AI action execution engine is now fully implemented and ready to use from the frontend.

## TypeScript Type Definitions

Add these types to your frontend code:

```typescript
// src/types/prompt-actions.ts

export type PromptAction =
  | { type: 'google-search'; query: string }
  | { type: 'youtube-search'; query: string }
  | { type: 'open-website'; url: string }
  | { type: 'launch-app'; appName: string }
  | { type: 'close-app'; appName: string }
  | { type: 'apple-shortcut'; shortcutName: string }
  | { type: 'shell-command'; command: string }
  | { type: 'ask-chatgpt'; prompt: string }  // Not yet implemented
  | { type: 'ask-claude'; prompt: string }   // Not yet implemented
  | { type: 'keypress'; keys: string };      // Not yet implemented

export interface AIPrompt {
  id: string;
  name: string;
  shortcut?: CustomShortcut;
  instruction: string;
  actions: PromptAction[];
  advancedSettings?: AdvancedPromptSettings;
  enabled: boolean;
}

export interface CustomShortcut {
  type: 'custom';
  modifiers: string[];
  key: string;
  displayLabel: string;
}

export interface AdvancedPromptSettings {
  temperature?: number;
  maxTokens?: number;
  systemPrompt?: string;
}
```

## Usage Examples

### 1. Execute a Single Action

```typescript
import { invoke } from '@tauri-apps/api';
import type { PromptAction } from './types/prompt-actions';

// Google Search
async function searchGoogle(query: string) {
  try {
    const result = await invoke<string>('execute_prompt_action', {
      action: {
        type: 'google-search',
        query
      } as PromptAction
    });
    console.log('✅', result);
    return result;
  } catch (error) {
    console.error('❌ Search failed:', error);
    throw error;
  }
}

// Launch Application
async function launchApp(appName: string) {
  try {
    const result = await invoke<string>('execute_prompt_action', {
      action: {
        type: 'launch-app',
        appName
      } as PromptAction
    });
    console.log('✅', result);
    return result;
  } catch (error) {
    console.error('❌ Launch failed:', error);
    throw error;
  }
}

// Open Website
async function openWebsite(url: string) {
  try {
    const result = await invoke<string>('execute_prompt_action', {
      action: {
        type: 'open-website',
        url
      } as PromptAction
    });
    console.log('✅', result);
    return result;
  } catch (error) {
    console.error('❌ Open failed:', error);
    throw error;
  }
}
```

### 2. Execute Multiple Actions from AI Prompt

```typescript
import { invoke } from '@tauri-apps/api';
import type { AIPrompt, PromptAction } from './types/prompt-actions';

async function executeAIPrompt(prompt: AIPrompt, transcription: string) {
  if (!prompt.enabled) {
    console.log('Prompt is disabled');
    return;
  }

  const results: string[] = [];
  const errors: Error[] = [];

  for (const action of prompt.actions) {
    try {
      // Replace {{transcription}} placeholder with actual text
      const processedAction = replaceTranscriptionPlaceholder(action, transcription);

      const result = await invoke<string>('execute_prompt_action', {
        action: processedAction
      });

      results.push(result);
      console.log(`✅ Action ${action.type}:`, result);
    } catch (error) {
      errors.push(error as Error);
      console.error(`❌ Action ${action.type} failed:`, error);
    }
  }

  return { results, errors };
}

function replaceTranscriptionPlaceholder(
  action: PromptAction,
  transcription: string
): PromptAction {
  const actionStr = JSON.stringify(action);
  const replaced = actionStr.replace(/\{\{transcription\}\}/g, transcription);
  return JSON.parse(replaced);
}
```

### 3. React Hook for Action Execution

```typescript
import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api';
import type { PromptAction } from './types/prompt-actions';

interface UsePromptActionResult {
  execute: (action: PromptAction) => Promise<string>;
  isExecuting: boolean;
  error: Error | null;
  result: string | null;
}

export function usePromptAction(): UsePromptActionResult {
  const [isExecuting, setIsExecuting] = useState(false);
  const [error, setError] = useState<Error | null>(null);
  const [result, setResult] = useState<string | null>(null);

  const execute = useCallback(async (action: PromptAction) => {
    setIsExecuting(true);
    setError(null);
    setResult(null);

    try {
      const res = await invoke<string>('execute_prompt_action', { action });
      setResult(res);
      return res;
    } catch (err) {
      const error = err as Error;
      setError(error);
      throw error;
    } finally {
      setIsExecuting(false);
    }
  }, []);

  return { execute, isExecuting, error, result };
}

// Usage in component
function ActionButton() {
  const { execute, isExecuting, error } = usePromptAction();

  const handleSearch = async () => {
    try {
      await execute({
        type: 'google-search',
        query: 'Tauri framework'
      });
    } catch (err) {
      console.error('Search failed:', err);
    }
  };

  return (
    <button onClick={handleSearch} disabled={isExecuting}>
      {isExecuting ? 'Searching...' : 'Search Google'}
      {error && <span className="error">{error.message}</span>}
    </button>
  );
}
```

### 4. AI Prompt Manager Component

```typescript
import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api';
import type { AIPrompt, PromptAction } from './types/prompt-actions';

function AIPromptManager() {
  const [prompts, setPrompts] = useState<AIPrompt[]>([]);
  const [transcription, setTranscription] = useState('');

  // Load prompts on mount
  useEffect(() => {
    loadPrompts();
  }, []);

  async function loadPrompts() {
    try {
      const loaded = await invoke<AIPrompt[]>('load_ai_prompts');
      setPrompts(loaded);
    } catch (error) {
      console.error('Failed to load prompts:', error);
    }
  }

  async function savePrompts() {
    try {
      await invoke('save_ai_prompts', { prompts });
      console.log('✅ Prompts saved');
    } catch (error) {
      console.error('❌ Failed to save prompts:', error);
    }
  }

  async function executePrompt(prompt: AIPrompt) {
    for (const action of prompt.actions) {
      try {
        // Replace placeholder with transcription
        const processedAction = JSON.parse(
          JSON.stringify(action).replace(/\{\{transcription\}\}/g, transcription)
        );

        const result = await invoke<string>('execute_prompt_action', {
          action: processedAction
        });

        console.log(`✅ ${prompt.name}:`, result);
      } catch (error) {
        console.error(`❌ ${prompt.name} failed:`, error);
      }
    }
  }

  return (
    <div className="prompt-manager">
      <h2>AI Prompts</h2>

      <input
        type="text"
        placeholder="Enter transcription..."
        value={transcription}
        onChange={(e) => setTranscription(e.target.value)}
      />

      <div className="prompts-list">
        {prompts.map((prompt) => (
          <div key={prompt.id} className="prompt-item">
            <h3>{prompt.name}</h3>
            <p>{prompt.instruction}</p>
            <button
              onClick={() => executePrompt(prompt)}
              disabled={!prompt.enabled || !transcription}
            >
              Execute
            </button>
          </div>
        ))}
      </div>

      <button onClick={savePrompts}>Save Prompts</button>
    </div>
  );
}
```

### 5. Predefined Action Templates

```typescript
import type { AIPrompt } from './types/prompt-actions';

export const PREDEFINED_PROMPTS: AIPrompt[] = [
  {
    id: 'search-google',
    name: 'Search Google',
    instruction: 'Search on Google',
    actions: [
      {
        type: 'google-search',
        query: '{{transcription}}'
      }
    ],
    enabled: true
  },
  {
    id: 'search-youtube',
    name: 'Search YouTube',
    instruction: 'Search on YouTube',
    actions: [
      {
        type: 'youtube-search',
        query: '{{transcription}}'
      }
    ],
    enabled: true
  },
  {
    id: 'open-safari',
    name: 'Open Safari',
    instruction: 'Launch Safari browser',
    actions: [
      {
        type: 'launch-app',
        appName: 'Safari'
      }
    ],
    enabled: true
  },
  {
    id: 'open-website',
    name: 'Open Website',
    instruction: 'Open a website',
    actions: [
      {
        type: 'open-website',
        url: 'https://{{transcription}}'
      }
    ],
    enabled: true
  },
  {
    id: 'multi-action',
    name: 'Search and Open',
    instruction: 'Search Google and open first result',
    actions: [
      {
        type: 'google-search',
        query: '{{transcription}}'
      },
      {
        type: 'launch-app',
        appName: 'Safari'
      }
    ],
    enabled: true
  }
];
```

## Error Handling

```typescript
import { invoke } from '@tauri-apps/api';
import type { PromptAction } from './types/prompt-actions';

async function executeActionWithErrorHandling(action: PromptAction) {
  try {
    const result = await invoke<string>('execute_prompt_action', { action });
    return { success: true, result };
  } catch (error) {
    const errorMessage = error as string;

    // Handle specific error types
    if (errorMessage.includes('Permission denied')) {
      return {
        success: false,
        error: 'Permission denied. Please grant necessary permissions.',
        type: 'permission'
      };
    }

    if (errorMessage.includes('Dangerous command')) {
      return {
        success: false,
        error: 'This command is blocked for security reasons.',
        type: 'security'
      };
    }

    if (errorMessage.includes('Invalid URL')) {
      return {
        success: false,
        error: 'Invalid URL format. Please use http:// or https://',
        type: 'validation'
      };
    }

    if (errorMessage.includes('not yet implemented')) {
      return {
        success: false,
        error: 'This action is not yet available.',
        type: 'not-implemented'
      };
    }

    return {
      success: false,
      error: errorMessage,
      type: 'unknown'
    };
  }
}
```

## Testing

```typescript
// Test all implemented actions
async function testAllActions() {
  const tests = [
    {
      name: 'Google Search',
      action: { type: 'google-search', query: 'test' } as PromptAction
    },
    {
      name: 'YouTube Search',
      action: { type: 'youtube-search', query: 'test' } as PromptAction
    },
    {
      name: 'Open Website',
      action: { type: 'open-website', url: 'https://example.com' } as PromptAction
    },
    {
      name: 'Launch App',
      action: { type: 'launch-app', appName: 'Safari' } as PromptAction
    }
  ];

  for (const test of tests) {
    console.log(`Testing: ${test.name}`);
    try {
      const result = await invoke<string>('execute_prompt_action', {
        action: test.action
      });
      console.log(`✅ ${test.name}: ${result}`);
    } catch (error) {
      console.error(`❌ ${test.name}: ${error}`);
    }
  }
}
```

## Security Considerations

1. **Always validate user input** before passing to actions
2. **Never allow direct shell command execution** from user input without review
3. **Use the built-in security features**:
   - URL validation
   - Path traversal protection
   - Dangerous command detection
4. **Consider implementing action permissions** for sensitive operations
5. **Log all action executions** for audit purposes

## Next Steps

1. Implement `save_ai_prompts` and `load_ai_prompts` commands (see BACKEND_INTEGRATION.md)
2. Add UI for managing AI prompts
3. Integrate with voice transcription workflow
4. Implement ChatGPT and Claude API integrations
5. Add keypress simulation support

---

**Status**: ✅ Ready for frontend integration
**Last Updated**: 2026-02-24
