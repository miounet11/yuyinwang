import React, { useState, useEffect, useRef } from 'react';
import { advancedShortcutManager, AdvancedShortcut, TriggerMode } from '../utils/advancedShortcutManager';
import './AdvancedShortcutEditor.css';

interface AdvancedShortcutEditorProps {
  isVisible: boolean;
  onClose: () => void;
}

const AdvancedShortcutEditor: React.FC<AdvancedShortcutEditorProps> = ({
  isVisible,
  onClose
}) => {
  const [shortcuts, setShortcuts] = useState<AdvancedShortcut[]>([]);
  const [activeTab, setActiveTab] = useState<'preset' | 'custom'>('preset');
  const [selectedCategory, setSelectedCategory] = useState<'all' | 'recording' | 'navigation' | 'editing' | 'system' | 'custom'>('all');
  const [editingShortcut, setEditingShortcut] = useState<string | null>(null);
  const [showAddDialog, setShowAddDialog] = useState(false);
  const [testMode, setTestMode] = useState(false);
  const [voiceEnabled, setVoiceEnabled] = useState(false);
  
  // 新建快捷键表单
  const [newShortcut, setNewShortcut] = useState({
    name: '',
    description: '',
    triggerMode: 'single' as TriggerMode,
    key: '',
    doubleClickTimeout: 300,
    holdDuration: 500,
    sequenceKeys: '',
    sequenceTimeout: 500,
    voicePhrases: '',
    voiceLanguage: 'zh-CN'
  });

  useEffect(() => {
    if (isVisible) {
      loadShortcuts();
    }
  }, [isVisible]);

  const loadShortcuts = () => {
    const allShortcuts = advancedShortcutManager.getShortcuts();
    setShortcuts(allShortcuts);
  };

  const handleTriggerModeChange = (shortcutId: string, mode: TriggerMode) => {
    advancedShortcutManager.updateShortcut(shortcutId, { triggerMode: mode });
    loadShortcuts();
  };

  const handleToggleShortcut = (shortcutId: string) => {
    const shortcut = shortcuts.find(s => s.id === shortcutId);
    if (shortcut) {
      advancedShortcutManager.updateShortcut(shortcutId, { enabled: !shortcut.enabled });
      loadShortcuts();
    }
  };

  const handleDeleteShortcut = (shortcutId: string) => {
    if (window.confirm('确定要删除这个快捷键吗？')) {
      advancedShortcutManager.removeShortcut(shortcutId);
      loadShortcuts();
    }
  };

  const handleAddCustomShortcut = () => {
    const triggerConfig: any = {};
    
    switch (newShortcut.triggerMode) {
      case 'single':
        triggerConfig.key = newShortcut.key;
        break;
      case 'double':
        triggerConfig.key = newShortcut.key;
        triggerConfig.timeout = newShortcut.doubleClickTimeout;
        break;
      case 'hold':
        triggerConfig.key = newShortcut.key;
        triggerConfig.duration = newShortcut.holdDuration;
        break;
      case 'sequence':
        triggerConfig.keys = newShortcut.sequenceKeys.split(',').map(k => k.trim());
        triggerConfig.timeout = newShortcut.sequenceTimeout;
        break;
      case 'voice':
        triggerConfig.phrases = newShortcut.voicePhrases.split(',').map(p => p.trim());
        triggerConfig.language = newShortcut.voiceLanguage;
        break;
    }

    advancedShortcutManager.createCustomShortcut({
      name: newShortcut.name,
      description: newShortcut.description,
      triggerMode: newShortcut.triggerMode,
      triggerConfig,
      action: async () => {
        console.log(`自定义快捷键触发: ${newShortcut.name}`);
      }
    });

    setShowAddDialog(false);
    resetNewShortcut();
    loadShortcuts();
  };

  const resetNewShortcut = () => {
    setNewShortcut({
      name: '',
      description: '',
      triggerMode: 'single',
      key: '',
      doubleClickTimeout: 300,
      holdDuration: 500,
      sequenceKeys: '',
      sequenceTimeout: 500,
      voicePhrases: '',
      voiceLanguage: 'zh-CN'
    });
  };

  const handleToggleVoice = () => {
    if (voiceEnabled) {
      advancedShortcutManager.stopVoiceRecognition();
    } else {
      advancedShortcutManager.startVoiceRecognition();
    }
    setVoiceEnabled(!voiceEnabled);
  };

  const getFilteredShortcuts = () => {
    if (selectedCategory === 'all') {
      return shortcuts;
    }
    return shortcuts.filter(s => s.category === selectedCategory);
  };

  const getTriggerModeIcon = (mode: TriggerMode) => {
    switch (mode) {
      case 'single': return '⌨️';
      case 'double': return '👆👆';
      case 'hold': return '⏱️';
      case 'sequence': return '🔢';
      case 'voice': return '🗣️';
      default: return '❓';
    }
  };

  const getTriggerModeLabel = (mode: TriggerMode) => {
    switch (mode) {
      case 'single': return '单键';
      case 'double': return '双击';
      case 'hold': return '长按';
      case 'sequence': return '序列';
      case 'voice': return '语音';
      default: return '未知';
    }
  };

  const renderShortcutConfig = (shortcut: AdvancedShortcut) => {
    switch (shortcut.triggerMode) {
      case 'single':
        return <span className="config-text">{shortcut.primaryKey || '未设置'}</span>;
      case 'double':
        return (
          <span className="config-text">
            双击 {shortcut.doubleClick?.key} ({shortcut.doubleClick?.timeout}ms)
          </span>
        );
      case 'hold':
        return (
          <span className="config-text">
            长按 {shortcut.hold?.key} ({shortcut.hold?.duration}ms)
          </span>
        );
      case 'sequence':
        return (
          <span className="config-text">
            序列 {shortcut.sequence?.keys.join(' → ')} ({shortcut.sequence?.timeout}ms)
          </span>
        );
      case 'voice':
        return (
          <span className="config-text">
            语音: "{shortcut.voiceCommand?.phrases.join('" 或 "')}"
          </span>
        );
      default:
        return <span className="config-text">未配置</span>;
    }
  };

  if (!isVisible) return null;

  return (
    <div className="advanced-shortcut-editor-overlay" onClick={onClose}>
      <div className="advanced-shortcut-editor-dialog" onClick={(e) => e.stopPropagation()}>
        <div className="editor-header">
          <div className="header-content">
            <h2>⚡ 高级快捷键管理</h2>
            <p>支持单键、双击、长按、序列键、语音等多种触发方式</p>
          </div>
          <button className="close-btn" onClick={onClose}>✕</button>
        </div>

        {/* 主标签页 */}
        <div className="main-tabs">
          <button 
            className={`main-tab ${activeTab === 'preset' ? 'active' : ''}`}
            onClick={() => setActiveTab('preset')}
          >
            预设快捷键
          </button>
          <button 
            className={`main-tab ${activeTab === 'custom' ? 'active' : ''}`}
            onClick={() => setActiveTab('custom')}
          >
            自定义快捷键
          </button>
        </div>

        {/* 分类过滤 */}
        <div className="category-filter">
          <button 
            className={`filter-btn ${selectedCategory === 'all' ? 'active' : ''}`}
            onClick={() => setSelectedCategory('all')}
          >
            全部
          </button>
          <button 
            className={`filter-btn ${selectedCategory === 'recording' ? 'active' : ''}`}
            onClick={() => setSelectedCategory('recording')}
          >
            🎤 录音
          </button>
          <button 
            className={`filter-btn ${selectedCategory === 'navigation' ? 'active' : ''}`}
            onClick={() => setSelectedCategory('navigation')}
          >
            🧭 导航
          </button>
          <button 
            className={`filter-btn ${selectedCategory === 'editing' ? 'active' : ''}`}
            onClick={() => setSelectedCategory('editing')}
          >
            ✏️ 编辑
          </button>
          <button 
            className={`filter-btn ${selectedCategory === 'system' ? 'active' : ''}`}
            onClick={() => setSelectedCategory('system')}
          >
            ⚙️ 系统
          </button>
          {activeTab === 'custom' && (
            <button 
              className={`filter-btn ${selectedCategory === 'custom' ? 'active' : ''}`}
              onClick={() => setSelectedCategory('custom')}
            >
              ⭐ 自定义
            </button>
          )}
        </div>

        {/* 快捷键列表 */}
        <div className="shortcuts-list">
          {getFilteredShortcuts().map(shortcut => (
            <div key={shortcut.id} className={`shortcut-card ${!shortcut.enabled ? 'disabled' : ''}`}>
              <div className="shortcut-main">
                <div className="shortcut-info">
                  <h3>{shortcut.name}</h3>
                  <p>{shortcut.description}</p>
                </div>
                
                <div className="trigger-mode">
                  <span className="mode-icon">{getTriggerModeIcon(shortcut.triggerMode)}</span>
                  <select 
                    value={shortcut.triggerMode}
                    onChange={(e) => handleTriggerModeChange(shortcut.id, e.target.value as TriggerMode)}
                    disabled={shortcut.category !== 'custom'}
                  >
                    <option value="single">单键</option>
                    <option value="double">双击</option>
                    <option value="hold">长按</option>
                    <option value="sequence">序列</option>
                    <option value="voice">语音</option>
                  </select>
                </div>

                <div className="shortcut-config">
                  {renderShortcutConfig(shortcut)}
                </div>

                <div className="shortcut-actions">
                  <button 
                    className={`toggle-btn ${shortcut.enabled ? 'enabled' : ''}`}
                    onClick={() => handleToggleShortcut(shortcut.id)}
                  >
                    {shortcut.enabled ? '✅' : '❌'}
                  </button>
                  
                  {shortcut.category === 'custom' && (
                    <>
                      <button 
                        className="edit-btn"
                        onClick={() => setEditingShortcut(shortcut.id)}
                      >
                        ✏️
                      </button>
                      <button 
                        className="delete-btn"
                        onClick={() => handleDeleteShortcut(shortcut.id)}
                      >
                        🗑️
                      </button>
                    </>
                  )}
                </div>
              </div>
            </div>
          ))}
        </div>

        {/* 添加自定义快捷键按钮 */}
        {activeTab === 'custom' && (
          <button className="add-custom-btn" onClick={() => setShowAddDialog(true)}>
            ➕ 添加自定义快捷键
          </button>
        )}

        {/* 添加快捷键对话框 */}
        {showAddDialog && (
          <div className="add-dialog-overlay" onClick={() => setShowAddDialog(false)}>
            <div className="add-dialog" onClick={(e) => e.stopPropagation()}>
              <h3>添加自定义快捷键</h3>
              
              <div className="form-group">
                <label>名称</label>
                <input 
                  type="text"
                  value={newShortcut.name}
                  onChange={(e) => setNewShortcut({...newShortcut, name: e.target.value})}
                  placeholder="例如：快速笔记"
                />
              </div>

              <div className="form-group">
                <label>描述</label>
                <input 
                  type="text"
                  value={newShortcut.description}
                  onChange={(e) => setNewShortcut({...newShortcut, description: e.target.value})}
                  placeholder="例如：快速创建一条笔记"
                />
              </div>

              <div className="form-group">
                <label>触发方式</label>
                <select 
                  value={newShortcut.triggerMode}
                  onChange={(e) => setNewShortcut({...newShortcut, triggerMode: e.target.value as TriggerMode})}
                >
                  <option value="single">单键</option>
                  <option value="double">双击</option>
                  <option value="hold">长按</option>
                  <option value="sequence">序列键</option>
                  <option value="voice">语音命令</option>
                </select>
              </div>

              {/* 根据触发方式显示不同的配置 */}
              {(newShortcut.triggerMode === 'single' || 
                newShortcut.triggerMode === 'double' || 
                newShortcut.triggerMode === 'hold') && (
                <div className="form-group">
                  <label>按键</label>
                  <input 
                    type="text"
                    value={newShortcut.key}
                    onChange={(e) => setNewShortcut({...newShortcut, key: e.target.value})}
                    placeholder="例如：CommandOrControl+Shift+N"
                  />
                </div>
              )}

              {newShortcut.triggerMode === 'double' && (
                <div className="form-group">
                  <label>双击超时 (ms)</label>
                  <input 
                    type="number"
                    value={newShortcut.doubleClickTimeout}
                    onChange={(e) => setNewShortcut({...newShortcut, doubleClickTimeout: parseInt(e.target.value)})}
                  />
                </div>
              )}

              {newShortcut.triggerMode === 'hold' && (
                <div className="form-group">
                  <label>长按时长 (ms)</label>
                  <input 
                    type="number"
                    value={newShortcut.holdDuration}
                    onChange={(e) => setNewShortcut({...newShortcut, holdDuration: parseInt(e.target.value)})}
                  />
                </div>
              )}

              {newShortcut.triggerMode === 'sequence' && (
                <>
                  <div className="form-group">
                    <label>序列键 (逗号分隔)</label>
                    <input 
                      type="text"
                      value={newShortcut.sequenceKeys}
                      onChange={(e) => setNewShortcut({...newShortcut, sequenceKeys: e.target.value})}
                      placeholder="例如：R,R,Enter"
                    />
                  </div>
                  <div className="form-group">
                    <label>序列超时 (ms)</label>
                    <input 
                      type="number"
                      value={newShortcut.sequenceTimeout}
                      onChange={(e) => setNewShortcut({...newShortcut, sequenceTimeout: parseInt(e.target.value)})}
                    />
                  </div>
                </>
              )}

              {newShortcut.triggerMode === 'voice' && (
                <>
                  <div className="form-group">
                    <label>语音命令 (逗号分隔)</label>
                    <input 
                      type="text"
                      value={newShortcut.voicePhrases}
                      onChange={(e) => setNewShortcut({...newShortcut, voicePhrases: e.target.value})}
                      placeholder="例如：开始录音,录音,记录"
                    />
                  </div>
                  <div className="form-group">
                    <label>语言</label>
                    <select 
                      value={newShortcut.voiceLanguage}
                      onChange={(e) => setNewShortcut({...newShortcut, voiceLanguage: e.target.value})}
                    >
                      <option value="zh-CN">中文</option>
                      <option value="en-US">English</option>
                      <option value="ja-JP">日本語</option>
                      <option value="ko-KR">한국어</option>
                    </select>
                  </div>
                </>
              )}

              <div className="dialog-actions">
                <button className="cancel-btn" onClick={() => {
                  setShowAddDialog(false);
                  resetNewShortcut();
                }}>
                  取消
                </button>
                <button className="save-btn" onClick={handleAddCustomShortcut}>
                  保存
                </button>
              </div>
            </div>
          </div>
        )}

        {/* 底部工具栏 */}
        <div className="editor-footer">
          <div className="footer-left">
            <button 
              className={`voice-toggle-btn ${voiceEnabled ? 'active' : ''}`}
              onClick={handleToggleVoice}
            >
              {voiceEnabled ? '🎙️ 语音控制已启用' : '🔇 启用语音控制'}
            </button>
            
            <button 
              className={`test-mode-btn ${testMode ? 'active' : ''}`}
              onClick={() => setTestMode(!testMode)}
            >
              {testMode ? '🧪 测试模式' : '▶️ 开始测试'}
            </button>
          </div>

          <div className="footer-right">
            <button className="import-btn">
              📥 导入配置
            </button>
            <button className="export-btn" onClick={() => {
              const config = advancedShortcutManager.exportConfig();
              const blob = new Blob([config], { type: 'application/json' });
              const url = URL.createObjectURL(blob);
              const a = document.createElement('a');
              a.href = url;
              a.download = 'shortcuts-config.json';
              a.click();
            }}>
              📤 导出配置
            </button>
            <button className="done-btn" onClick={onClose}>
              完成
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};

export default AdvancedShortcutEditor;