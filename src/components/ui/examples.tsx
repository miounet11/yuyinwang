/**
 * Spokenly Design System - Usage Examples
 * Comprehensive examples showing how to use each component
 */

import React, { useState } from 'react';
import {
  SpokenlyLayout,
  SpokemlySidebar,
  SpokenlyContent,
  SpokenlyNavItem,
  SpokenlyNavSection,
  SpokenlyButton,
  SpokenlyCard,
  SpokenlyCardHeader,
  SpokenlyCardBody,
  SpokenlyCardFooter,
  SpokenlyInput,
  SpokenlySwitch,
  SpokenlySelect,
  SpokenlyTag,
  SpokenlyModelCard,
  SpokenlyUploadArea,
  SpokenlyHistoryItem,
  SpokenlySearchBox,
  SelectOption,
  ModelStatus
} from './index';

export const SpokenlyComponentShowcase: React.FC = () => {
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false);
  const [selectedNav, setSelectedNav] = useState('dashboard');
  const [switchValue, setSwitchValue] = useState(false);
  const [selectValue, setSelectValue] = useState('');
  const [selectedModel, setSelectedModel] = useState('gpt-4');
  const [searchValue, setSearchValue] = useState('');

  // Sample data
  const navItems = [
    { id: 'dashboard', label: 'Dashboard', icon: '📊' },
    { id: 'transcription', label: 'Transcription', icon: '🎤', badge: '3' },
    { id: 'models', label: 'Models', icon: '🤖' },
    { id: 'history', label: 'History', icon: '📚' },
    { id: 'settings', label: 'Settings', icon: '⚙️' }
  ];

  const selectOptions: SelectOption[] = [
    { value: 'option1', label: 'Option 1', icon: '🔧' },
    { value: 'option2', label: 'Option 2', icon: '🎯' },
    { value: 'option3', label: 'Option 3', disabled: true, icon: '❌' }
  ];

  const modelStatus: ModelStatus = {
    type: 'online',
    message: 'Ready for transcription'
  };

  return (
    <SpokenlyLayout>
      {/* Sidebar */}
      <SpokemlySidebar
        isCollapsed={sidebarCollapsed}
        onToggle={() => setSidebarCollapsed(!sidebarCollapsed)}
      >
        <SpokenlyNavSection title="Main">
          {navItems.slice(0, 3).map(item => (
            <SpokenlyNavItem
              key={item.id}
              label={item.label}
              icon={item.icon}
              badge={item.badge}
              isActive={selectedNav === item.id}
              onClick={() => setSelectedNav(item.id)}
            />
          ))}
        </SpokenlyNavSection>

        <SpokenlyNavSection title="Tools">
          {navItems.slice(3).map(item => (
            <SpokenlyNavItem
              key={item.id}
              label={item.label}
              icon={item.icon}
              isActive={selectedNav === item.id}
              onClick={() => setSelectedNav(item.id)}
            />
          ))}
        </SpokenlyNavSection>
      </SpokemlySidebar>

      {/* Main Content */}
      <SpokenlyContent>
        <div style={{ maxWidth: '1200px', margin: '0 auto' }}>
          <h1 style={{ 
            fontSize: 'var(--spokenly-text-3xl)', 
            fontWeight: 'var(--spokenly-font-bold)',
            color: 'var(--spokenly-text-primary)',
            marginBottom: 'var(--spokenly-space-8)'
          }}>
            Spokenly Component Library
          </h1>

          {/* Search Section */}
          <SpokenlyCard className="mb-6">
            <SpokenlyCardHeader title="Search & Navigation" />
            <SpokenlyCardBody>
              <SpokenlySearchBox
                placeholder="Search transcriptions..."
                value={searchValue}
                onSearch={(query) => setSearchValue(query)}
                onClear={() => setSearchValue('')}
                fullWidth
              />
            </SpokenlyCardBody>
          </SpokenlyCard>

          {/* Buttons Section */}
          <SpokenlyCard className="mb-6">
            <SpokenlyCardHeader title="Button Examples" />
            <SpokenlyCardBody>
              <div style={{ 
                display: 'flex', 
                gap: 'var(--spokenly-space-4)', 
                flexWrap: 'wrap',
                marginBottom: 'var(--spokenly-space-4)'
              }}>
                <SpokenlyButton variant="primary">Primary Button</SpokenlyButton>
                <SpokenlyButton variant="secondary">Secondary Button</SpokenlyButton>
                <SpokenlyButton variant="ghost">Ghost Button</SpokenlyButton>
                <SpokenlyButton variant="danger">Danger Button</SpokenlyButton>
                <SpokenlyButton variant="success">Success Button</SpokenlyButton>
              </div>

              <div style={{ 
                display: 'flex', 
                gap: 'var(--spokenly-space-4)', 
                flexWrap: 'wrap' 
              }}>
                <SpokenlyButton size="xs">Extra Small</SpokenlyButton>
                <SpokenlyButton size="sm">Small</SpokenlyButton>
                <SpokenlyButton size="md">Medium</SpokenlyButton>
                <SpokenlyButton size="lg">Large</SpokenlyButton>
              </div>

              <div style={{ marginTop: 'var(--spokenly-space-4)' }}>
                <SpokenlyButton
                  variant="primary"
                  leftIcon="🚀"
                  rightIcon="→"
                >
                  Button with Icons
                </SpokenlyButton>
              </div>
            </SpokenlyCardBody>
          </SpokenlyCard>

          {/* Form Controls */}
          <SpokenlyCard className="mb-6">
            <SpokenlyCardHeader title="Form Controls" />
            <SpokenlyCardBody>
              <div style={{ 
                display: 'grid', 
                gap: 'var(--spokenly-space-6)',
                gridTemplateColumns: 'repeat(auto-fit, minmax(300px, 1fr))'
              }}>
                <div>
                  <SpokenlyInput
                    label="Email Address"
                    placeholder="Enter your email"
                    leftIcon="📧"
                    helperText="We'll never share your email"
                  />
                </div>

                <div>
                  <SpokenlySelect
                    label="Select Option"
                    placeholder="Choose an option..."
                    options={selectOptions}
                    value={selectValue}
                    onValueChange={setSelectValue}
                  />
                </div>

                <div>
                  <SpokenlySwitch
                    checked={switchValue}
                    onCheckedChange={setSwitchValue}
                    label="Enable notifications"
                    description="Get notified about transcription updates"
                  />
                </div>
              </div>
            </SpokenlyCardBody>
          </SpokenlyCard>

          {/* Tags */}
          <SpokenlyCard className="mb-6">
            <SpokenlyCardHeader title="Tags & Labels" />
            <SpokenlyCardBody>
              <div style={{ 
                display: 'flex', 
                gap: 'var(--spokenly-space-3)', 
                flexWrap: 'wrap' 
              }}>
                <SpokenlyTag variant="default">Default</SpokenlyTag>
                <SpokenlyTag variant="success" icon="✅">Success</SpokenlyTag>
                <SpokenlyTag variant="warning" icon="⚠️">Warning</SpokenlyTag>
                <SpokenlyTag variant="error" icon="❌">Error</SpokenlyTag>
                <SpokenlyTag variant="info" icon="ℹ️">Info</SpokenlyTag>
                <SpokenlyTag removable onRemove={() => alert('Tag removed!')}>
                  Removable
                </SpokenlyTag>
              </div>
            </SpokenlyCardBody>
          </SpokenlyCard>

          {/* Model Card */}
          <SpokenlyCard className="mb-6">
            <SpokenlyCardHeader title="AI Model Cards" />
            <SpokenlyCardBody>
              <div style={{ 
                display: 'grid', 
                gap: 'var(--spokenly-space-4)',
                gridTemplateColumns: 'repeat(auto-fit, minmax(350px, 1fr))'
              }}>
                <SpokenlyModelCard
                  title="GPT-4 Turbo"
                  description="Most capable GPT-4 model with improved performance and efficiency"
                  provider="OpenAI"
                  status={modelStatus}
                  isSelected={selectedModel === 'gpt-4'}
                  onSelect={() => setSelectedModel('gpt-4')}
                  tags={['Latest', 'High Quality', 'Fast']}
                  pricing="$0.01 / 1K tokens"
                />

                <SpokenlyModelCard
                  title="Whisper Large"
                  description="Advanced speech recognition model for high-accuracy transcription"
                  provider="OpenAI"
                  status={{ type: 'offline', message: 'Model loading...' }}
                  isSelected={selectedModel === 'whisper'}
                  onSelect={() => setSelectedModel('whisper')}
                  tags={['Offline', 'Privacy', 'Multilingual']}
                  pricing="Free"
                />
              </div>
            </SpokenlyCardBody>
          </SpokenlyCard>

          {/* Upload Area */}
          <SpokenlyCard className="mb-6">
            <SpokenlyCardHeader title="File Upload" />
            <SpokenlyCardBody>
              <SpokenlyUploadArea
                title="Upload Audio Files"
                description="Drag and drop audio files or click to select"
                accept="audio/*"
                multiple
                maxSize={50 * 1024 * 1024} // 50MB
                onFilesDrop={(files) => console.log('Files dropped:', files)}
                onFilesSelect={(files) => console.log('Files selected:', files)}
              />
            </SpokenlyCardBody>
          </SpokenlyCard>

          {/* History Items */}
          <SpokenlyCard>
            <SpokenlyCardHeader title="Transcription History" />
            <SpokenlyCardBody>
              <SpokenlyHistoryItem
                id="1"
                title="Meeting Recording - Q4 Planning"
                content="This is a sample transcription content that would appear here. It contains the full text of the transcribed audio with proper formatting and structure. The content can be quite long and will show an expand/collapse functionality when it exceeds a certain length."
                timestamp={new Date(Date.now() - 3600000)} // 1 hour ago
                duration={1845} // 30 minutes 45 seconds
                fileSize={12500000} // 12.5MB
                format="MP3"
                onPlay={(id) => console.log('Play:', id)}
                onExport={(id) => console.log('Export:', id)}
                onDelete={(id) => console.log('Delete:', id)}
              />

              <SpokenlyHistoryItem
                id="2"
                title="Voice Note - Project Ideas"
                content="Short voice note about project ideas."
                timestamp={new Date(Date.now() - 86400000)} // 1 day ago
                duration={125} // 2 minutes 5 seconds
                fileSize={2100000} // 2.1MB
                format="WAV"
                onPlay={(id) => console.log('Play:', id)}
                onExport={(id) => console.log('Export:', id)}
                onDelete={(id) => console.log('Delete:', id)}
              />
            </SpokenlyCardBody>
          </SpokenlyCard>
        </div>
      </SpokenlyContent>
    </SpokenlyLayout>
  );
};

// Individual component examples for documentation
export const ButtonExamples = () => (
  <div>
    <h2>Button Examples</h2>
    
    {/* Basic Usage */}
    <SpokenlyButton variant="primary" onClick={() => alert('Clicked!')}>
      Click Me
    </SpokenlyButton>

    {/* With Loading State */}
    <SpokenlyButton variant="primary" isLoading loadingText="Processing...">
      Submit
    </SpokenlyButton>

    {/* With Icons */}
    <SpokenlyButton 
      variant="secondary" 
      leftIcon="📁" 
      rightIcon="→"
    >
      Open File
    </SpokenlyButton>
  </div>
);

export const CardExamples = () => (
  <div>
    <h2>Card Examples</h2>
    
    {/* Simple Card */}
    <SpokenlyCard>
      <SpokenlyCardHeader 
        title="Simple Card" 
        subtitle="This is a basic card example"
      />
      <SpokenlyCardBody>
        <p>Card content goes here...</p>
      </SpokenlyCardBody>
      <SpokenlyCardFooter>
        <SpokenlyButton variant="primary">Action</SpokenlyButton>
      </SpokenlyCardFooter>
    </SpokenlyCard>

    {/* Interactive Card */}
    <SpokenlyCard hover selected onClick={() => alert('Card clicked!')}>
      <SpokenlyCardBody>
        <p>This card is interactive and selectable</p>
      </SpokenlyCardBody>
    </SpokenlyCard>
  </div>
);

export const FormExamples = () => {
  const [formData, setFormData] = useState({
    email: '',
    notifications: false,
    category: ''
  });

  return (
    <div>
      <h2>Form Examples</h2>
      
      <SpokenlyInput
        label="Email"
        value={formData.email}
        onChange={(e) => setFormData({...formData, email: e.target.value})}
        placeholder="Enter email address"
        leftIcon="📧"
      />

      <SpokenlySwitch
        checked={formData.notifications}
        onCheckedChange={(checked) => setFormData({...formData, notifications: checked})}
        label="Email Notifications"
        description="Receive updates via email"
      />

      <SpokenlySelect
        label="Category"
        value={formData.category}
        onValueChange={(value) => setFormData({...formData, category: value})}
        options={[
          { value: 'work', label: 'Work' },
          { value: 'personal', label: 'Personal' }
        ]}
      />
    </div>
  );
};

export default SpokenlyComponentShowcase;