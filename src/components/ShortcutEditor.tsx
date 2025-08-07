import React, { useState, useEffect, useRef } from 'react';
import { shortcutManager, Shortcut } from '../utils/shortcutManager';
import './ShortcutEditor.css';

interface ShortcutEditorProps {
  isVisible: boolean;
  onClose: () => void;
  shortcuts?: any[]; // 保持兼容性
  onUpdateShortcut?: (shortcut: any) => void;
  onAddShortcut?: () => void;
}

const ShortcutEditor: React.FC<ShortcutEditorProps> = ({
  isVisible,
  onClose
}) => {
  const [allShortcuts, setAllShortcuts] = useState<Shortcut[]>([]);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [recordingKeys, setRecordingKeys] = useState(false);
  const [currentKeyCombo, setCurrentKeyCombo] = useState<string>('');
  const [activeCategory, setActiveCategory] = useState<'all' | 'recording' | 'navigation' | 'editing' | 'system'>('all');
  const [testMode, setTestMode] = useState(false);
  const [lastTriggered, setLastTriggered] = useState<string>('');
  const dialogRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (isVisible) {
      // 加载所有快捷键
      const shortcuts = shortcutManager.getShortcuts();
      setAllShortcuts(shortcuts);

      // 注册事件监听器
      const handleShortcutTrigger = (id: string) => {
        setLastTriggered(id);
        setTimeout(() => setLastTriggered(''), 2000);
      };

      // 为测试模式添加监听器
      shortcuts.forEach(shortcut => {
        shortcutManager.on(shortcut.id.replace(/-/g, '_'), () => {
          handleShortcutTrigger(shortcut.id);
        });
      });
    }
  }, [isVisible]);

  useEffect(() => {
    if (dialogRef.current && isVisible) {
      dialogRef.current.focus();
    }
  }, [isVisible]);

  const handleKeyDown = (e: KeyboardEvent) => {
    if (recordingKeys) {
      e.preventDefault();
      e.stopPropagation();
      
      const combo = shortcutManager.detectKeyCombo(e);
      if (combo) {
        setCurrentKeyCombo(combo);
      }
    } else if (e.key === 'Escape' && !editingId) {
      onClose();
    }
  };

  const handleKeyUp = (e: KeyboardEvent) => {
    if (recordingKeys && currentKeyCombo) {
      e.preventDefault();
      e.stopPropagation();
      
      // 验证快捷键
      if (shortcutManager.isValidShortcut(currentKeyCombo)) {
        // 检查是否已被使用
        if (shortcutManager.isShortcutInUse(currentKeyCombo, editingId || undefined)) {
          alert('此快捷键组合已被使用！');
        } else if (editingId) {
          // 更新快捷键
          shortcutManager.updateShortcut(editingId, currentKeyCombo);
          
          // 更新显示
          const updatedShortcuts = shortcutManager.getShortcuts();
          setAllShortcuts(updatedShortcuts);
        }
      } else {
        alert('请使用有效的快捷键组合（需要包含修饰键）');
      }
      
      // 重置状态
      setRecordingKeys(false);
      setCurrentKeyCombo('');
      setEditingId(null);
    }
  };

  useEffect(() => {
    if (recordingKeys) {
      document.addEventListener('keydown', handleKeyDown);
      document.addEventListener('keyup', handleKeyUp);
      
      return () => {
        document.removeEventListener('keydown', handleKeyDown);
        document.removeEventListener('keyup', handleKeyUp);
      };
    }
  }, [recordingKeys, currentKeyCombo, editingId]);

  const handleRecordShortcut = (shortcutId: string) => {
    setEditingId(shortcutId);
    setRecordingKeys(true);
    setCurrentKeyCombo('');
  };

  const handleToggleShortcut = async (shortcutId: string) => {
    const isEnabled = shortcutManager.toggleShortcut(shortcutId);
    
    // 更新显示
    const updatedShortcuts = shortcutManager.getShortcuts();
    setAllShortcuts(updatedShortcuts);
    
    console.log(`快捷键 ${shortcutId} 已${isEnabled ? '启用' : '禁用'}`);
  };

  const handleResetShortcut = (shortcutId: string) => {
    // 重置为默认值
    const defaultShortcuts: { [key: string]: string } = {
      'toggle-recording': 'CommandOrControl+Shift+R',
      'quick-transcribe': 'CommandOrControl+Shift+Space',
      'pause-recording': 'CommandOrControl+Shift+P',
      'open-ai-assistant': 'CommandOrControl+Shift+A',
      'switch-to-history': 'CommandOrControl+H',
      'switch-to-models': 'CommandOrControl+M',
      'switch-to-settings': 'CommandOrControl+Comma',
      'copy-transcription': 'CommandOrControl+Shift+C',
      'export-transcription': 'CommandOrControl+Shift+E',
      'delete-selected': 'CommandOrControl+Delete',
      'toggle-window': 'CommandOrControl+Shift+S',
      'minimize-window': 'CommandOrControl+Shift+M',
      'reload-app': 'CommandOrControl+R'
    };

    if (defaultShortcuts[shortcutId]) {
      shortcutManager.updateShortcut(shortcutId, defaultShortcuts[shortcutId]);
      const updatedShortcuts = shortcutManager.getShortcuts();
      setAllShortcuts(updatedShortcuts);
    }
  };

  const handleTestMode = async () => {
    setTestMode(!testMode);
    
    if (!testMode) {
      // 注册所有快捷键以进行测试
      await shortcutManager.registerAllShortcuts();
      alert('测试模式已启用！按下快捷键查看效果。');
    } else {
      // 注销快捷键
      await shortcutManager.unregisterAllShortcuts();
    }
  };

  const getFilteredShortcuts = () => {
    if (activeCategory === 'all') {
      return allShortcuts;
    }
    return shortcutManager.getShortcutsByCategory(activeCategory);
  };

  const getCategoryIcon = (category: string) => {
    switch (category) {
      case 'recording': return '🎤';
      case 'navigation': return '🧭';
      case 'editing': return '✏️';
      case 'system': return '⚙️';
      default: return '📌';
    }
  };

  const formatKey = (key: string) => {
    return key
      .replace('CommandOrControl', '⌘/Ctrl')
      .replace('Shift', '⇧')
      .replace('Alt', '⌥')
      .replace('Space', '␣')
      .replace('Delete', '⌫')
      .replace('Comma', ',')
      .replace('+', ' + ');
  };

  if (!isVisible) return null;

  return (
    <div className="shortcut-editor-overlay" onClick={onClose}>
      <div 
        ref={dialogRef}
        className="shortcut-editor-dialog enhanced" 
        onClick={(e) => e.stopPropagation()}
        tabIndex={0}
      >
        <div className="shortcut-editor-header">
          <div className="header-title">
            <h2>⌨️ 快捷键管理</h2>
            <p>配置和管理所有快捷键组合，提升您的工作效率</p>
          </div>
          <button className="close-btn" onClick={onClose}>✕</button>
        </div>

        {/* 分类标签 */}
        <div className="category-tabs">
          <button 
            className={`category-tab ${activeCategory === 'all' ? 'active' : ''}`}
            onClick={() => setActiveCategory('all')}
          >
            全部 ({allShortcuts.length})
          </button>
          <button 
            className={`category-tab ${activeCategory === 'recording' ? 'active' : ''}`}
            onClick={() => setActiveCategory('recording')}
          >
            🎤 录音 ({shortcutManager.getShortcutsByCategory('recording').length})
          </button>
          <button 
            className={`category-tab ${activeCategory === 'navigation' ? 'active' : ''}`}
            onClick={() => setActiveCategory('navigation')}
          >
            🧭 导航 ({shortcutManager.getShortcutsByCategory('navigation').length})
          </button>
          <button 
            className={`category-tab ${activeCategory === 'editing' ? 'active' : ''}`}
            onClick={() => setActiveCategory('editing')}
          >
            ✏️ 编辑 ({shortcutManager.getShortcutsByCategory('editing').length})
          </button>
          <button 
            className={`category-tab ${activeCategory === 'system' ? 'active' : ''}`}
            onClick={() => setActiveCategory('system')}
          >
            ⚙️ 系统 ({shortcutManager.getShortcutsByCategory('system').length})
          </button>
        </div>

        <div className="shortcuts-container">
          <div className="shortcuts-list enhanced">
            {getFilteredShortcuts().map((shortcut) => (
              <div 
                key={shortcut.id} 
                className={`shortcut-item enhanced ${!shortcut.enabled ? 'disabled' : ''} ${lastTriggered === shortcut.id ? 'triggered' : ''}`}
              >
                <div className="shortcut-left">
                  <div className="shortcut-icon">{getCategoryIcon(shortcut.category)}</div>
                  <div className="shortcut-info">
                    <div className="shortcut-name">{shortcut.name}</div>
                    <div className="shortcut-description">{shortcut.description}</div>
                  </div>
                </div>
                
                <div className="shortcut-controls">
                  <div className="key-display">
                    {editingId === shortcut.id && recordingKeys ? (
                      <span className="recording">
                        {currentKeyCombo || '录制中...'}
                      </span>
                    ) : (
                      <span className={`key-combo ${shortcut.enabled ? 'active' : 'inactive'}`}>
                        {formatKey(shortcut.key)}
                      </span>
                    )}
                  </div>

                  <div className="action-buttons">
                    <button 
                      className="toggle-btn"
                      onClick={() => handleToggleShortcut(shortcut.id)}
                      title={shortcut.enabled ? '禁用' : '启用'}
                    >
                      {shortcut.enabled ? '✅' : '❌'}
                    </button>
                    
                    <button 
                      className="edit-btn"
                      onClick={() => handleRecordShortcut(shortcut.id)}
                      disabled={!shortcut.enabled}
                      title="编辑快捷键"
                    >
                      ✏️
                    </button>
                    
                    <button 
                      className="reset-btn"
                      onClick={() => handleResetShortcut(shortcut.id)}
                      title="重置为默认"
                    >
                      🔄
                    </button>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* 测试区域 */}
        <div className="test-section enhanced">
          <div className="test-header">
            <h3>🧪 测试快捷键</h3>
            <button 
              className={`test-toggle ${testMode ? 'active' : ''}`}
              onClick={handleTestMode}
            >
              {testMode ? '停止测试' : '开始测试'}
            </button>
          </div>
          
          {testMode && (
            <div className="test-content">
              <p className="test-hint">测试模式已启用！按下任意已启用的快捷键查看效果。</p>
              {lastTriggered && (
                <div className="test-result">
                  <span className="triggered-label">触发:</span>
                  <span className="triggered-shortcut">
                    {allShortcuts.find(s => s.id === lastTriggered)?.name}
                  </span>
                </div>
              )}
            </div>
          )}
        </div>

        {/* 提示信息 */}
        <div className="shortcut-tips enhanced">
          <div className="tip-card">
            <div className="tip-icon">💡</div>
            <div className="tip-content">
              <h4>快捷键使用技巧</h4>
              <ul>
                <li>• 使用 <kbd>⌘/Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>R</kbd> 快速开始录音</li>
                <li>• 使用 <kbd>⌘/Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>Space</kbd> 进行快速转录</li>
                <li>• 使用 <kbd>⌘/Ctrl</kbd> + <kbd>H</kbd> 快速查看历史记录</li>
                <li>• 点击编辑按钮 ✏️ 可以自定义任何快捷键</li>
              </ul>
            </div>
          </div>

          <div className="tip-card">
            <div className="tip-icon">⚠️</div>
            <div className="tip-content">
              <h4>macOS Fn 键设置</h4>
              <ul>
                <li>• 系统设置 → 键盘</li>
                <li>• 点击"按下 🌐 键以"</li>
                <li>• 选择"无操作"</li>
                <li>• 启用 Fn 键快捷键功能</li>
              </ul>
            </div>
          </div>
        </div>

        <div className="shortcut-editor-footer">
          <div className="footer-stats">
            已启用: {allShortcuts.filter(s => s.enabled).length} / {allShortcuts.length}
          </div>
          <div className="footer-actions">
            <button className="apply-btn" onClick={async () => {
              await shortcutManager.registerAllShortcuts();
              alert('快捷键已应用！');
            }}>
              应用更改
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

export default ShortcutEditor;