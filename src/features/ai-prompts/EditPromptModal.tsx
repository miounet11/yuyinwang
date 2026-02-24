import React, { useState, useEffect } from 'react';
import type { AIPrompt, PromptAction, CustomShortcut, AdvancedPromptSettings } from '../../shared/types';
import { validateCustomShortcut, generateShortcutLabel } from '../../shared/utils';
import './EditPromptModal.css';

interface EditPromptModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSave: (prompt: AIPrompt) => void;
  initialPrompt?: AIPrompt | null;
}

export const EditPromptModal: React.FC<EditPromptModalProps> = ({
  isOpen,
  onClose,
  onSave,
  initialPrompt,
}) => {
  const [currentStep, setCurrentStep] = useState(1);
  const [name, setName] = useState('');
  const [shortcut, setShortcut] = useState<CustomShortcut | undefined>(undefined);
  const [instruction, setInstruction] = useState('');
  const [actions, setActions] = useState<PromptAction[]>([]);
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [advancedSettings, setAdvancedSettings] = useState<AdvancedPromptSettings>({
    model: 'gpt-4o-mini',
    temperature: 0.7,
    maxTokens: 1000,
  });

  // 录制快捷键状态
  const [isRecordingShortcut, setIsRecordingShortcut] = useState(false);
  const [pressedModifiers, setPressedModifiers] = useState<Set<string>>(new Set());
  const [pressedKey, setPressedKey] = useState<string | null>(null);

  useEffect(() => {
    if (initialPrompt) {
      setName(initialPrompt.name);
      setShortcut(initialPrompt.shortcut);
      setInstruction(initialPrompt.instruction);
      setActions(initialPrompt.actions);
      if (initialPrompt.advancedSettings) {
        setAdvancedSettings(initialPrompt.advancedSettings);
        setShowAdvanced(true);
      }
    }
  }, [initialPrompt]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (!isRecordingShortcut) return;
    e.preventDefault();

    const modifiers = new Set<string>();
    if (e.metaKey) modifiers.add('cmd');
    if (e.ctrlKey) modifiers.add('ctrl');
    if (e.shiftKey) modifiers.add('shift');
    if (e.altKey) modifiers.add('opt');

    setPressedModifiers(modifiers);

    const MODIFIER_KEYS = new Set(['Meta', 'Control', 'Shift', 'Alt']);
    if (!MODIFIER_KEYS.has(e.key)) {
      const key = e.key.length === 1 ? e.key.toUpperCase() : e.key;
      setPressedKey(key);

      // 验证快捷键
      const validation = validateCustomShortcut(modifiers as Set<any>, key);
      if (validation.isValid) {
        const customShortcut: CustomShortcut = {
          type: 'custom',
          modifiers: Array.from(modifiers) as any[],
          key,
          displayLabel: generateShortcutLabel({
            type: 'custom',
            modifiers: Array.from(modifiers) as any[],
            key,
            displayLabel: '',
          }),
        };
        setShortcut(customShortcut);
        setIsRecordingShortcut(false);
        setPressedModifiers(new Set());
        setPressedKey(null);
      }
    }
  };

  const handleAddAction = (type: PromptAction['type']) => {
    let newAction: PromptAction;

    switch (type) {
      case 'google-search':
        newAction = { type: 'google-search', query: '' };
        break;
      case 'launch-app':
        newAction = { type: 'launch-app', appName: '' };
        break;
      case 'close-app':
        newAction = { type: 'close-app', appName: '' };
        break;
      case 'ask-chatgpt':
        newAction = { type: 'ask-chatgpt', prompt: '' };
        break;
      case 'ask-claude':
        newAction = { type: 'ask-claude', prompt: '' };
        break;
      case 'youtube-search':
        newAction = { type: 'youtube-search', query: '' };
        break;
      case 'open-website':
        newAction = { type: 'open-website', url: '' };
        break;
      case 'apple-shortcut':
        newAction = { type: 'apple-shortcut', shortcutName: '' };
        break;
      case 'shell-command':
        newAction = { type: 'shell-command', command: '' };
        break;
      case 'keypress':
        newAction = { type: 'keypress', keys: '' };
        break;
      default:
        return;
    }

    setActions([...actions, newAction]);
  };

  const handleRemoveAction = (index: number) => {
    setActions(actions.filter((_, i) => i !== index));
  };

  const handleUpdateAction = (index: number, updates: Partial<PromptAction>) => {
    setActions(
      actions.map((action, i) => (i === index ? { ...action, ...updates } : action))
    );
  };

  const handleSave = () => {
    if (!name.trim()) {
      alert('请输入提示名称');
      return;
    }

    if (!instruction.trim()) {
      alert('请输入 AI 指令');
      return;
    }

    const prompt: AIPrompt = {
      id: initialPrompt?.id || Date.now().toString(),
      name: name.trim(),
      shortcut,
      instruction: instruction.trim(),
      actions,
      advancedSettings: showAdvanced ? advancedSettings : undefined,
      enabled: initialPrompt?.enabled ?? true,
    };

    onSave(prompt);
  };

  const renderActionInput = (action: PromptAction, index: number) => {
    switch (action.type) {
      case 'google-search':
      case 'youtube-search':
        return (
          <input
            type="text"
            placeholder="搜索关键词"
            value={action.query}
            onChange={(e) => handleUpdateAction(index, { query: e.target.value })}
            className="action-input"
          />
        );
      case 'launch-app':
      case 'close-app':
        return (
          <input
            type="text"
            placeholder="应用名称 (如: Safari)"
            value={action.appName}
            onChange={(e) => handleUpdateAction(index, { appName: e.target.value })}
            className="action-input"
          />
        );
      case 'ask-chatgpt':
      case 'ask-claude':
        return (
          <textarea
            placeholder="AI 提示词"
            value={action.prompt}
            onChange={(e) => handleUpdateAction(index, { prompt: e.target.value })}
            className="action-textarea"
            rows={2}
          />
        );
      case 'open-website':
        return (
          <input
            type="url"
            placeholder="网站 URL (如: https://example.com)"
            value={action.url}
            onChange={(e) => handleUpdateAction(index, { url: e.target.value })}
            className="action-input"
          />
        );
      case 'apple-shortcut':
        return (
          <input
            type="text"
            placeholder="快捷指令名称"
            value={action.shortcutName}
            onChange={(e) => handleUpdateAction(index, { shortcutName: e.target.value })}
            className="action-input"
          />
        );
      case 'shell-command':
        return (
          <textarea
            placeholder="Shell 命令 (如: ls -la)"
            value={action.command}
            onChange={(e) => handleUpdateAction(index, { command: e.target.value })}
            className="action-textarea"
            rows={2}
          />
        );
      case 'keypress':
        return (
          <input
            type="text"
            placeholder="按键组合 (如: Command+C)"
            value={action.keys}
            onChange={(e) => handleUpdateAction(index, { keys: e.target.value })}
            className="action-input"
          />
        );
      default:
        return null;
    }
  };

  const getActionLabel = (type: PromptAction['type']): string => {
    const labels: Record<PromptAction['type'], string> = {
      'google-search': 'Google 搜索',
      'launch-app': '启动应用',
      'close-app': '关闭应用',
      'ask-chatgpt': 'ChatGPT',
      'ask-claude': 'Claude',
      'youtube-search': 'YouTube 搜索',
      'open-website': '打开网站',
      'apple-shortcut': 'Apple 快捷指令',
      'shell-command': 'Shell 命令',
      'keypress': '按键',
    };
    return labels[type];
  };

  if (!isOpen) return null;

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content edit-prompt-modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h2>{initialPrompt ? '编辑 AI 提示' : '创建 AI 提示'}</h2>
          <button className="modal-close" onClick={onClose}>
            ×
          </button>
        </div>

        <div className="modal-steps">
          <div className={`step ${currentStep === 1 ? 'active' : currentStep > 1 ? 'completed' : ''}`}>
            <div className="step-number">1</div>
            <div className="step-label">配置激活方式</div>
          </div>
          <div className="step-divider" />
          <div className={`step ${currentStep === 2 ? 'active' : currentStep > 2 ? 'completed' : ''}`}>
            <div className="step-number">2</div>
            <div className="step-label">定义 AI 指令</div>
          </div>
        </div>

        <div className="modal-body">
          {currentStep === 1 && (
            <div className="step-content">
              <div className="form-group">
                <label>提示名称</label>
                <input
                  type="text"
                  placeholder="输入提示名称"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  className="form-input"
                />
              </div>

              <div className="form-group">
                <label>快捷键 (可选)</label>
                <div className="shortcut-input-wrapper">
                  <input
                    type="text"
                    placeholder={isRecordingShortcut ? '请按下快捷键组合...' : '点击录制快捷键'}
                    value={shortcut?.displayLabel || ''}
                    readOnly
                    onFocus={() => setIsRecordingShortcut(true)}
                    onBlur={() => setIsRecordingShortcut(false)}
                    onKeyDown={handleKeyDown}
                    className={`form-input shortcut-input ${isRecordingShortcut ? 'recording' : ''}`}
                  />
                  {shortcut && (
                    <button
                      className="clear-shortcut"
                      onClick={() => setShortcut(undefined)}
                      title="清除快捷键"
                    >
                      ×
                    </button>
                  )}
                </div>
                {isRecordingShortcut && (
                  <div className="shortcut-hint">
                    按下至少一个修饰键 (⌘/⌥/⇧/⌃) + 一个普通键
                  </div>
                )}
              </div>
            </div>
          )}

          {currentStep === 2 && (
            <div className="step-content">
              <div className="form-group">
                <label>AI 指令</label>
                <textarea
                  placeholder="输入 AI 指令，例如：总结以下内容..."
                  value={instruction}
                  onChange={(e) => setInstruction(e.target.value)}
                  className="form-textarea"
                  rows={4}
                />
              </div>

              <div className="form-group">
                <label>动作按钮</label>
                <div className="actions-grid-selector">
                  <button
                    className="action-selector-btn"
                    onClick={() => handleAddAction('google-search')}
                  >
                    🔍 Google 搜索
                  </button>
                  <button
                    className="action-selector-btn"
                    onClick={() => handleAddAction('launch-app')}
                  >
                    🚀 启动应用
                  </button>
                  <button
                    className="action-selector-btn"
                    onClick={() => handleAddAction('close-app')}
                  >
                    ❌ 关闭应用
                  </button>
                  <button
                    className="action-selector-btn"
                    onClick={() => handleAddAction('ask-chatgpt')}
                  >
                    💬 ChatGPT
                  </button>
                  <button
                    className="action-selector-btn"
                    onClick={() => handleAddAction('ask-claude')}
                  >
                    🤖 Claude
                  </button>
                  <button
                    className="action-selector-btn"
                    onClick={() => handleAddAction('youtube-search')}
                  >
                    📺 YouTube 搜索
                  </button>
                  <button
                    className="action-selector-btn"
                    onClick={() => handleAddAction('open-website')}
                  >
                    🌐 打开网站
                  </button>
                  <button
                    className="action-selector-btn"
                    onClick={() => handleAddAction('apple-shortcut')}
                  >
                    ⚡ Apple 快捷指令
                  </button>
                  <button
                    className="action-selector-btn"
                    onClick={() => handleAddAction('shell-command')}
                  >
                    ⌨️ Shell 命令
                  </button>
                  <button
                    className="action-selector-btn"
                    onClick={() => handleAddAction('keypress')}
                  >
                    ⌨️ 按键
                  </button>
                </div>
              </div>

              {actions.length > 0 && (
                <div className="form-group">
                  <label>已添加的动作</label>
                  <div className="actions-list">
                    {actions.map((action, index) => (
                      <div key={index} className="action-item">
                        <div className="action-item-header">
                          <span className="action-item-label">{getActionLabel(action.type)}</span>
                          <button
                            className="action-item-remove"
                            onClick={() => handleRemoveAction(index)}
                            title="删除动作"
                          >
                            ×
                          </button>
                        </div>
                        {renderActionInput(action, index)}
                      </div>
                    ))}
                  </div>
                </div>
              )}

              <div className="form-group">
                <button
                  className="advanced-toggle"
                  onClick={() => setShowAdvanced(!showAdvanced)}
                >
                  {showAdvanced ? '▼' : '▶'} 高级设置
                </button>
                {showAdvanced && (
                  <div className="advanced-settings">
                    <div className="form-group">
                      <label>模型</label>
                      <select
                        value={advancedSettings.model}
                        onChange={(e) =>
                          setAdvancedSettings({ ...advancedSettings, model: e.target.value })
                        }
                        className="form-select"
                      >
                        <option value="gpt-4o-mini">GPT-4o mini</option>
                        <option value="gpt-4o">GPT-4o</option>
                        <option value="claude-3-5-sonnet">Claude 3.5 Sonnet</option>
                        <option value="claude-3-opus">Claude 3 Opus</option>
                      </select>
                    </div>
                    <div className="form-group">
                      <label>温度 ({advancedSettings.temperature})</label>
                      <input
                        type="range"
                        min="0"
                        max="2"
                        step="0.1"
                        value={advancedSettings.temperature}
                        onChange={(e) =>
                          setAdvancedSettings({
                            ...advancedSettings,
                            temperature: parseFloat(e.target.value),
                          })
                        }
                        className="form-range"
                      />
                    </div>
                    <div className="form-group">
                      <label>最大 Token 数</label>
                      <input
                        type="number"
                        min="100"
                        max="4000"
                        step="100"
                        value={advancedSettings.maxTokens}
                        onChange={(e) =>
                          setAdvancedSettings({
                            ...advancedSettings,
                            maxTokens: parseInt(e.target.value),
                          })
                        }
                        className="form-input"
                      />
                    </div>
                  </div>
                )}
              </div>
            </div>
          )}
        </div>

        <div className="modal-footer">
          {currentStep === 1 ? (
            <>
              <button className="btn-secondary" onClick={onClose}>
                取消
              </button>
              <button
                className="btn-primary"
                onClick={() => setCurrentStep(2)}
                disabled={!name.trim()}
              >
                下一步
              </button>
            </>
          ) : (
            <>
              <button className="btn-secondary" onClick={() => setCurrentStep(1)}>
                上一步
              </button>
              <button className="btn-primary" onClick={handleSave} disabled={!instruction.trim()}>
                保存
              </button>
            </>
          )}
        </div>
      </div>
    </div>
  );
};
