# Model Configuration Components Documentation

## Overview

This documentation describes the implementation of two React components for managing transcription models in the Spokenly application:

1. **ModelConfigDialog** - Configure API keys and model parameters for online services
2. **LocalModelManager** - Manage local model downloads and storage

Both components integrate seamlessly with the existing `useModelsStore` Zustand store for state management.

## Components

### 1. ModelConfigDialog

A dialog component for configuring API keys and parameters for online transcription models.

#### Features
- Dynamic form fields based on API provider configuration
- Connection testing functionality
- Support for different field types (password, select, toggle, slider)
- Real-time validation
- Integration with the models store

#### Props
```typescript
interface ModelConfigDialogProps {
  isVisible: boolean;
  model: TranscriptionModel | null;
  onClose: () => void;
}
```

#### Usage Example
```tsx
import ModelConfigDialog from './components/ModelConfigDialog';
import { useModelsStore } from './stores/modelsStore';

function App() {
  const { models } = useModelsStore();
  const [showConfig, setShowConfig] = useState(false);
  const [selectedModel, setSelectedModel] = useState<TranscriptionModel | null>(null);

  const handleConfigureModel = (model: TranscriptionModel) => {
    setSelectedModel(model);
    setShowConfig(true);
  };

  return (
    <div>
      {/* Your model list */}
      {models.filter(m => m.requiresApiKey).map(model => (
        <button key={model.id} onClick={() => handleConfigureModel(model)}>
          Configure {model.name}
        </button>
      ))}

      <ModelConfigDialog
        isVisible={showConfig}
        model={selectedModel}
        onClose={() => setShowConfig(false)}
      />
    </div>
  );
}
```

#### Configuration Fields

The component automatically renders different field types based on the provider configuration:

- **password**: For API keys (masked input)
- **select**: For model version selection
- **toggle**: For boolean options like real-time transcription
- **slider**: For numerical values like temperature
- **text**: For general text input

#### API Integration

The component uses the `ModelAPI.testApiConnection()` method to test configurations:

```typescript
const result = await ModelAPI.testApiConnection(model.id, config);
```

### 2. LocalModelManager

A comprehensive dialog for managing local model downloads, installation, and storage.

#### Features
- Model storage path management
- Download progress tracking with pause/resume/cancel
- Storage space monitoring
- Model installation/uninstallation
- Integration with Tauri file system APIs

#### Props
```typescript
interface LocalModelManagerProps {
  isVisible: boolean;
  onClose: () => void;
}
```

#### Usage Example
```tsx
import LocalModelManager from './components/LocalModelManager';

function App() {
  const [showLocalManager, setShowLocalManager] = useState(false);

  return (
    <div>
      <button onClick={() => setShowLocalManager(true)}>
        Manage Local Models
      </button>

      <LocalModelManager
        isVisible={showLocalManager}
        onClose={() => setShowLocalManager(false)}
      />
    </div>
  );
}
```

#### Features Detail

1. **Storage Management**
   - Display current model storage path
   - Change storage location with folder picker
   - Show storage space usage with visual progress bar
   - Display downloaded models size

2. **Download Management**
   - Start/pause/resume/cancel downloads
   - Real-time progress tracking
   - Download speed and time remaining
   - Error handling

3. **Model Operations**
   - Install/uninstall local models
   - Open model folder in file explorer
   - Delete model files

## Store Integration

Both components integrate with the `useModelsStore` Zustand store:

```typescript
const {
  models,                    // All available models
  modelConfigs,             // Saved API configurations
  downloadTasks,            // Active download tasks
  saveModelConfig,          // Save API configuration
  startDownload,            // Start model download
  pauseDownload,            // Pause download
  resumeDownload,           // Resume download
  cancelDownload,           // Cancel download
  uninstallModel           // Remove installed model
} = useModelsStore();
```

## API Methods

The components use the following `ModelAPI` methods:

### Configuration APIs
- `ModelAPI.testApiConnection(modelId, config)` - Test API connection
- `ModelAPI.saveApiConfig(modelId, config)` - Save API configuration
- `ModelAPI.loadApiConfig(modelId)` - Load saved configuration

### Local Model APIs
- `ModelAPI.downloadLocalModel(modelId)` - Start download
- `ModelAPI.pauseDownload(modelId)` - Pause download
- `ModelAPI.resumeDownload(modelId)` - Resume download
- `ModelAPI.cancelDownload(modelId)` - Cancel download
- `ModelAPI.deleteLocalModel(modelId)` - Delete model
- `ModelAPI.getLocalModelsPath()` - Get storage path
- `ModelAPI.setLocalModelsPath(path)` - Set storage path
- `ModelAPI.openModelFolder()` - Open in file explorer
- `ModelAPI.getStorageInfo()` - Get disk space info

## Type Definitions

Key types used by the components:

```typescript
interface ModelConfig {
  modelId: string;
  apiKey?: string;
  apiUrl?: string;
  language?: string;
  realtimeTranscription?: boolean;
  detectSpeaker?: boolean;
  temperature?: number;
}

interface APIProvider {
  id: string;
  name: string;
  icon: string;
  description: string;
  requiresApiKey: boolean;
  configFields: ConfigField[];
  testEndpoint?: string;
}

interface ConfigField {
  name: string;
  label: string;
  type: 'text' | 'password' | 'select' | 'toggle' | 'number' | 'slider';
  placeholder?: string;
  options?: { value: string; label: string }[];
  required?: boolean;
}
```

## Styling

Both components use CSS custom properties for theming:

```css
:root {
  --bg-primary: #ffffff;
  --bg-secondary: #f8fafc;
  --bg-tertiary: #f1f5f9;
  --text-primary: #0f172a;
  --text-secondary: #64748b;
  --accent-color: #3b82f6;
  --border-color: #e2e8f0;
  --success-color: #10b981;
  --error-color: #ef4444;
}
```

The components are fully responsive and include mobile-optimized layouts.

## Error Handling

Both components include comprehensive error handling:

- Network errors during API testing
- File system errors during path selection
- Download failures with retry options
- Graceful fallbacks for missing data

## Accessibility

The components follow accessibility best practices:

- Keyboard navigation support
- ARIA labels and descriptions
- Focus management
- Screen reader compatibility
- Color contrast compliance

## Browser Compatibility

The components are compatible with modern browsers and use:

- ES2020+ JavaScript features
- CSS Grid and Flexbox
- CSS Custom Properties
- Modern React patterns (hooks, functional components)

## Performance Considerations

- Components use React.memo for optimization where appropriate
- Large lists are virtualized when necessary
- API calls are debounced to prevent excessive requests
- Download progress updates are throttled

## Testing

Example usage component is provided at `src/components/ComponentUsageExample.tsx` which demonstrates:

- Proper component integration
- State management patterns
- Error handling examples
- User interaction flows