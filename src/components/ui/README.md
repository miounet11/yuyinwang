# Spokenly Design System - Component Documentation

## Overview

The Spokenly Design System provides a comprehensive React component library built for the Recording King application. All components follow the established design tokens and patterns defined in the `spokenly-design-system.css` file.

## Features

- ✅ **Pixel-perfect design** - Matches Spokenly interface specifications
- ✅ **TypeScript support** - Fully typed interfaces and props
- ✅ **Accessibility first** - WCAG compliant with keyboard navigation
- ✅ **Motion design** - Smooth animations using Framer Motion
- ✅ **Responsive design** - Mobile-first approach with breakpoint support
- ✅ **Dark mode ready** - Supports system preference detection
- ✅ **Modular architecture** - Import only what you need

## Installation & Setup

```tsx
// Import the design system CSS (required)
import '../styles/spokenly-design-system.css';

// Import components
import { SpokenlyButton, SpokenlyCard } from './components/ui';

// Or import component groups
import { FormComponents, LayoutComponents } from './components/ui';
```

## Component Categories

### Layout Components

#### SpokenlyLayout
Main application container with proper structure and animations.

```tsx
<SpokenlyLayout>
  <SpokemlySidebar />
  <SpokenlyContent />
</SpokenlyLayout>
```

#### SpokemlySidebar
Collapsible navigation sidebar with smooth transitions.

```tsx
<SpokemlySidebar
  isCollapsed={false}
  onToggle={() => setSidebarOpen(!sidebarOpen)}
  width={250}
>
  <SpokenlyNavSection title="Main">
    <SpokenlyNavItem label="Dashboard" icon="📊" isActive />
  </SpokenlyNavSection>
</SpokemlySidebar>
```

#### SpokenlyContent
Main content area with proper spacing and scroll handling.

```tsx
<SpokenlyContent padding="lg">
  <h1>Page Content</h1>
</SpokenlyContent>
```

### Navigation Components

#### SpokenlyNavItem
Interactive navigation items with active states and badges.

```tsx
<SpokenlyNavItem
  label="Transcription"
  icon="🎤"
  badge="3"
  isActive={currentPage === 'transcription'}
  onClick={() => navigate('/transcription')}
/>
```

#### SpokenlyNavSection
Groups navigation items with optional section titles.

```tsx
<SpokenlyNavSection title="Tools">
  <SpokenlyNavItem label="Models" icon="🤖" />
  <SpokenlyNavItem label="History" icon="📚" />
</SpokenlyNavSection>
```

### Basic Components

#### SpokenlyButton
Versatile button component with multiple variants and states.

```tsx
// Basic usage
<SpokenlyButton variant="primary" onClick={handleSave}>
  Save Changes
</SpokenlyButton>

// With icons and loading
<SpokenlyButton
  variant="secondary"
  size="lg"
  leftIcon="📁"
  rightIcon="→"
  isLoading={uploading}
  loadingText="Uploading..."
>
  Upload File
</SpokenlyButton>
```

**Props:**
- `variant`: 'primary' | 'secondary' | 'ghost' | 'danger' | 'success'
- `size`: 'xs' | 'sm' | 'md' | 'lg'
- `isLoading`: boolean
- `leftIcon`, `rightIcon`: ReactNode
- `fullWidth`: boolean

#### SpokenlyCard
Flexible card container with header, body, and footer sections.

```tsx
<SpokenlyCard hover selected onClick={handleSelect}>
  <SpokenlyCardHeader
    title="Card Title"
    subtitle="Optional subtitle"
    actions={<SpokenlyButton size="sm">Edit</SpokenlyButton>}
  />
  <SpokenlyCardBody padding="lg">
    <p>Card content goes here...</p>
  </SpokenlyCardBody>
  <SpokenlyCardFooter justify="between">
    <span>Footer info</span>
    <SpokenlyButton variant="primary">Action</SpokenlyButton>
  </SpokenlyCardFooter>
</SpokenlyCard>
```

#### SpokenlyInput
Enhanced input field with icons, validation, and labels.

```tsx
<SpokenlyInput
  label="Email Address"
  placeholder="Enter your email"
  leftIcon="📧"
  rightIcon="✓"
  helperText="We'll never share your email"
  errorText={emailError}
  value={email}
  onChange={(e) => setEmail(e.target.value)}
/>
```

### Form Components

#### SpokenlySwitch
iOS-style toggle switch with smooth animations.

```tsx
<SpokenlySwitch
  checked={notifications}
  onCheckedChange={setNotifications}
  label="Email Notifications"
  description="Receive updates about transcription progress"
  size="md"
/>
```

#### SpokenlySelect
Dropdown select with search and custom options.

```tsx
<SpokenlySelect
  label="AI Model"
  placeholder="Choose a model..."
  options={[
    { value: 'gpt-4', label: 'GPT-4 Turbo', icon: '🚀' },
    { value: 'whisper', label: 'Whisper Large', icon: '🎤' }
  ]}
  value={selectedModel}
  onValueChange={setSelectedModel}
/>
```

#### SpokenlyTag
Colored tags with optional remove functionality.

```tsx
<SpokenlyTag variant="success" icon="✅" removable onRemove={handleRemove}>
  Online
</SpokenlyTag>

<SpokenlyTag variant="info" size="sm">
  High Quality
</SpokenlyTag>
```

### Specialized Components

#### SpokenlyModelCard
AI model selection card with status indicators.

```tsx
<SpokenlyModelCard
  title="GPT-4 Turbo"
  description="Most capable model with improved performance"
  provider="OpenAI"
  status={{ type: 'online', message: 'Ready' }}
  isSelected={selectedModel === 'gpt-4'}
  onSelect={() => setSelectedModel('gpt-4')}
  tags={['Latest', 'Fast', 'High Quality']}
  pricing="$0.01 / 1K tokens"
  actions={<SpokenlyButton size="sm">Configure</SpokenlyButton>}
/>
```

#### SpokenlyUploadArea
File upload with drag & drop functionality.

```tsx
<SpokenlyUploadArea
  title="Upload Audio Files"
  description="Drag and drop files or click to select"
  accept="audio/*"
  multiple
  maxSize={50 * 1024 * 1024} // 50MB
  onFilesDrop={(files) => handleFileDrop(files)}
  onFilesSelect={(files) => handleFileSelect(files)}
/>
```

### Interaction Components

#### SpokenlyHistoryItem
History record with expandable content and actions.

```tsx
<SpokenlyHistoryItem
  id="recording-123"
  title="Meeting Recording - Q4 Planning"
  content="Full transcription content here..."
  timestamp={new Date()}
  duration={1845} // seconds
  fileSize={12500000} // bytes
  format="MP3"
  isSelected={selectedItem === 'recording-123'}
  onSelect={setSelectedItem}
  onPlay={handlePlay}
  onExport={handleExport}
  onDelete={handleDelete}
/>
```

#### SpokenlySearchBox
Search input with debouncing and clear functionality.

```tsx
<SpokenlySearchBox
  placeholder="Search transcriptions..."
  value={searchQuery}
  onSearch={(query) => performSearch(query)}
  onClear={() => clearSearch()}
  debounceMs={300}
  showClearButton
  fullWidth
/>
```

## Design Tokens

All components use CSS custom properties from the design system:

```css
/* Colors */
--spokenly-primary: #007AFF;
--spokenly-bg-content: #FFFFFF;
--spokenly-text-primary: #1D1D1F;

/* Typography */
--spokenly-font-family: -apple-system, BlinkMacSystemFont, ...;
--spokenly-text-base: 15px;

/* Spacing */
--spokenly-space-4: 16px;
--spokenly-content-padding: 24px;

/* Animations */
--spokenly-duration-fast: 150ms;
--spokenly-ease-out: cubic-bezier(0.0, 0.0, 0.2, 1);
```

## Accessibility Features

- **Keyboard Navigation**: All interactive elements support keyboard navigation
- **Screen Readers**: Proper ARIA labels and roles
- **Focus Management**: Visible focus indicators and logical tab order
- **Color Contrast**: Meets WCAG AA standards
- **Reduced Motion**: Respects `prefers-reduced-motion`
- **High Contrast**: Supports high contrast mode

## Animation Guidelines

Components use consistent animation patterns:

- **Micro-interactions**: 150ms with ease-out timing
- **Page transitions**: 250ms with ease-in-out
- **Loading states**: Smooth spinner animations
- **Hover effects**: Subtle scale and color changes
- **Layout animations**: Framer Motion's layout animations

## Responsive Design

Components automatically adapt to different screen sizes:

```css
/* Mobile devices */
@media (max-width: 768px) {
  --spokenly-sidebar-width: 200px;
  --spokenly-content-padding: 16px;
}

/* Small screens */
@media (max-width: 640px) {
  --spokenly-sidebar-width: 100vw;
  --spokenly-content-padding: 12px;
}
```

## Best Practices

### Component Usage

1. **Import only what you need** to optimize bundle size
2. **Use semantic HTML** elements when possible
3. **Provide meaningful labels** for accessibility
4. **Handle loading and error states** appropriately
5. **Follow the established color patterns** from the design system

### Performance Optimization

1. **Lazy load components** for large applications
2. **Memoize expensive calculations** in custom hooks
3. **Use proper React keys** for list items
4. **Optimize animation performance** with `will-change`
5. **Implement virtualization** for large lists

### Customization

While components are designed to match Spokenly specifications exactly, you can customize them using:

1. **CSS custom properties** to override design tokens
2. **className prop** for additional styling
3. **style prop** for one-off customizations
4. **Component composition** for complex layouts

## Migration Guide

If upgrading from existing components:

1. **Update imports** to use the new component names
2. **Review prop changes** - some props may have been renamed
3. **Update CSS classes** to use the new design system
4. **Test accessibility** with screen readers and keyboard navigation
5. **Verify animations** work correctly across browsers

## Browser Support

- **Chrome**: 91+
- **Firefox**: 90+
- **Safari**: 14+
- **Edge**: 91+

## Contributing

When contributing new components:

1. **Follow the existing patterns** established by other components
2. **Include TypeScript interfaces** with comprehensive prop documentation
3. **Add accessibility features** from the start
4. **Write comprehensive examples** in the examples file
5. **Test across different screen sizes** and interaction methods

---

For more detailed examples, see the `examples.tsx` file in the same directory.