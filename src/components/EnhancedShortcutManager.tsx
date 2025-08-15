import React, { useState, useEffect } from 'react';
import { enhancedShortcutManager, ShortcutConfig, ShortcutAction } from '../utils/enhancedShortcutManager';
import './EnhancedShortcutManager.css';

interface EnhancedShortcutManagerProps {
  isVisible: boolean;
  onClose: () => void;
}

const actionLabels: Record<ShortcutAction, string> = {
  'toggle_recording': '切换录音状态',
  'start_recording': '开始录音',
  'stop_recording': '停止录音',
  'show_app': '显示应用',
  'hide_app': '隐藏应用',
  'toggle_visibility': '切换窗口显示',
  'quick_transcribe': '快速转录',
  'show_history': '显示历史记录',
  'toggle_text_injection': '切换文本注入'
};

export default function EnhancedShortcutManager({ isVisible, onClose }: EnhancedShortcutManagerProps) {
  const [shortcuts, setShortcuts] = useState<ShortcutConfig[]>([]);
  const [editingShortcut, setEditingShortcut] = useState<string | null>(null);
  const [newShortcutKey, setNewShortcutKey] = useState('');
  const [conflictMessage, setConflictMessage] = useState('');
  const [stats, setStats] = useState({ total: 0, enabled: 0, disabled: 0, global: 0, local: 0 });

  useEffect(() => {
    if (isVisible) {
      loadShortcuts();
    }
  }, [isVisible]);

  const loadShortcuts = () => {
    const allShortcuts = enhancedShortcutManager.getShortcuts();
    setShortcuts(allShortcuts);
    
    const currentStats = enhancedShortcutManager.getStats();
    setStats(currentStats);
  };

  const handleToggleEnabled = async (key: string) => {
    const shortcut = shortcuts.find(s => s.key === key);
    if (!shortcut) return;

    const success = await enhancedShortcutManager.toggleShortcut(key, !shortcut.enabled);
    if (success) {
      loadShortcuts();
    } else {
      setConflictMessage(`切换快捷键失败: ${key}`);
    }
  };

  const handleStartEdit = (key: string) => {
    setEditingShortcut(key);
    setNewShortcutKey(key);
    setConflictMessage('');
  };

  const handleSaveEdit = async () => {
    if (!editingShortcut || !newShortcutKey) return;

    // 验证快捷键格式
    if (!enhancedShortcutManager.validateShortcutKey(newShortcutKey)) {
      setConflictMessage('快捷键格式无效。请使用类似 "CommandOrControl+Shift+R" 的格式');
      return;
    }

    // 检查冲突
    if (newShortcutKey !== editingShortcut && enhancedShortcutManager.isKeyRegistered(newShortcutKey)) {
      setConflictMessage(`快捷键 "${newShortcutKey}" 已被使用`);
      return;
    }

    const oldShortcut = shortcuts.find(s => s.key === editingShortcut);
    if (!oldShortcut) return;

    const newConfig: ShortcutConfig = {
      ...oldShortcut,
      key: newShortcutKey
    };

    const success = await enhancedShortcutManager.updateShortcut(editingShortcut, newConfig);
    if (success) {
      setEditingShortcut(null);
      setNewShortcutKey('');
      setConflictMessage('');
      loadShortcuts();
    } else {
      setConflictMessage('更新快捷键失败');
    }
  };

  const handleCancelEdit = () => {
    setEditingShortcut(null);
    setNewShortcutKey('');
    setConflictMessage('');
  };

  const handleTestShortcut = (key: string) => {
    enhancedShortcutManager.simulateShortcut(key);
    setConflictMessage(`已测试快捷键: ${key}`);
    setTimeout(() => setConflictMessage(''), 2000);
  };

  const handleResetToDefaults = async () => {
    if (confirm('确定要重置所有快捷键为默认设置吗？')) {
      const success = await enhancedShortcutManager.resetToDefaults();
      if (success) {
        loadShortcuts();
        setConflictMessage('快捷键已重置为默认设置');
        setTimeout(() => setConflictMessage(''), 3000);
      } else {
        setConflictMessage('重置快捷键失败');
      }
    }
  };

  const renderShortcutKey = (key: string): React.ReactNode => {
    const parts = key.split('+');
    return (
      <div className="shortcut-key-display">
        {parts.map((part, index) => (
          <React.Fragment key={index}>
            <kbd className="key-part">{part}</kbd>
            {index < parts.length - 1 && <span className="key-separator">+</span>}
          </React.Fragment>
        ))}
      </div>
    );
  };

  if (!isVisible) return null;

  return (
    <div className="enhanced-shortcut-overlay">
      <div className="enhanced-shortcut-modal">
        <div className="enhanced-shortcut-header">
          <h2>⌨️ 快捷键管理</h2>
          <button className="close-btn" onClick={onClose}>×</button>
        </div>

        <div className="enhanced-shortcut-content">
          {/* 统计信息 */}
          <div className="shortcut-stats">
            <div className="stat-item">
              <span className="stat-label">总数:</span>
              <span className="stat-value">{stats.total}</span>
            </div>
            <div className="stat-item">
              <span className="stat-label">已启用:</span>
              <span className="stat-value enabled">{stats.enabled}</span>
            </div>
            <div className="stat-item">
              <span className="stat-label">已禁用:</span>
              <span className="stat-value disabled">{stats.disabled}</span>
            </div>
            <div className="stat-item">
              <span className="stat-label">全局:</span>
              <span className="stat-value global">{stats.global}</span>
            </div>
          </div>

          {/* 错误/状态消息 */}
          {conflictMessage && (
            <div className={`message ${conflictMessage.includes('失败') ? 'error' : 'info'}`}>
              {conflictMessage}
            </div>
          )}

          {/* 快捷键列表 */}
          <div className="shortcuts-list">
            <div className="list-header">
              <div className="col-action">功能</div>
              <div className="col-shortcut">快捷键</div>
              <div className="col-status">状态</div>
              <div className="col-controls">操作</div>
            </div>

            {shortcuts.map((shortcut) => (
              <div key={shortcut.key} className={`shortcut-row ${!shortcut.enabled ? 'disabled' : ''}`}>
                <div className="col-action">
                  <div className="action-name">{actionLabels[shortcut.action]}</div>
                  <div className="action-description">{shortcut.description}</div>
                </div>

                <div className="col-shortcut">
                  {editingShortcut === shortcut.key ? (
                    <div className="shortcut-edit">
                      <input
                        type="text"
                        value={newShortcutKey}
                        onChange={(e) => setNewShortcutKey(e.target.value)}
                        placeholder="Ctrl+Shift+Key"
                        className="shortcut-input"
                        onKeyDown={(e) => {
                          if (e.key === 'Enter') {
                            handleSaveEdit();
                          } else if (e.key === 'Escape') {
                            handleCancelEdit();
                          }
                        }}
                      />
                      <div className="edit-controls">
                        <button onClick={handleSaveEdit} className="save-btn">✓</button>
                        <button onClick={handleCancelEdit} className="cancel-btn">×</button>
                      </div>
                    </div>
                  ) : (
                    <div onClick={() => handleStartEdit(shortcut.key)} className="shortcut-display">
                      {renderShortcutKey(shortcut.key)}
                    </div>
                  )}
                </div>

                <div className="col-status">
                  <div className={`status-indicator ${shortcut.enabled ? 'enabled' : 'disabled'}`}>
                    {shortcut.enabled ? '✅ 启用' : '❌ 禁用'}
                  </div>
                  {shortcut.global && shortcut.enabled && (
                    <div className="global-badge">🌐 全局</div>
                  )}
                </div>

                <div className="col-controls">
                  <button
                    onClick={() => handleToggleEnabled(shortcut.key)}
                    className={`control-btn toggle-btn ${shortcut.enabled ? 'disable' : 'enable'}`}
                    title={shortcut.enabled ? '禁用' : '启用'}
                  >
                    {shortcut.enabled ? '⏸️' : '▶️'}
                  </button>

                  <button
                    onClick={() => handleTestShortcut(shortcut.key)}
                    className="control-btn test-btn"
                    title="测试快捷键"
                    disabled={!shortcut.enabled}
                  >
                    🧪
                  </button>

                  <button
                    onClick={() => handleStartEdit(shortcut.key)}
                    className="control-btn edit-btn"
                    title="编辑快捷键"
                  >
                    ✏️
                  </button>
                </div>
              </div>
            ))}
          </div>

          {/* 操作按钮 */}
          <div className="action-buttons">
            <button onClick={handleResetToDefaults} className="action-btn reset-btn">
              🔄 重置默认
            </button>
            <button onClick={loadShortcuts} className="action-btn refresh-btn">
              🔄 刷新列表
            </button>
            <button onClick={onClose} className="action-btn close-btn">
              ✅ 完成
            </button>
          </div>

          {/* 帮助信息 */}
          <div className="help-section">
            <h3>💡 使用说明</h3>
            <ul>
              <li>点击快捷键可以编辑，支持 CommandOrControl+Shift+Key 格式</li>
              <li>CommandOrControl 会根据系统自动选择 Cmd (Mac) 或 Ctrl (Windows/Linux)</li>
              <li>全局快捷键在应用不在前台时也能工作</li>
              <li>测试按钮会模拟触发对应的快捷键功能</li>
              <li>禁用的快捷键不会响应按键事件</li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  );
}