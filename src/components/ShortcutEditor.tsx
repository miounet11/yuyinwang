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
  const [shortcuts, setShortcuts] = useState([
    { id: 'shortcut', name: '快捷键', key: 'Fn', description: '按住或切功能' },
    { id: 'attach-shortcut', name: '附加快捷键', key: '右键', description: '按住或功能' }
  ]);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [recordingKeys, setRecordingKeys] = useState(false);
  const [currentKeyCombo, setCurrentKeyCombo] = useState<string>('');
  const [testResult, setTestResult] = useState<string>('');

  const handleKeyEdit = (shortcutId: string, dropdown: string) => {
    setShortcuts(prev => prev.map(s => 
      s.id === shortcutId ? { ...s, key: dropdown } : s
    ));
  };

  const testShortcut = (shortcut: any) => {
    if (shortcut.key === 'Fn') {
      setTestResult('🎛️ 按住Fn键话，松开停止录音。');
    } else {
      setTestResult('正在测试快捷键...');
    }
  };

  if (!isVisible) return null;

  return (
    <div className="shortcut-editor-overlay" onClick={onClose}>
      <div className="shortcut-editor-dialog" onClick={(e) => e.stopPropagation()}>
        <div className="shortcut-header">
          <h2>快捷键</h2>
          <p>选择您喜欢的键盘操作键来启动 Spokenly，仅按这些键即可开启录音。</p>
        </div>

        <div className="shortcut-section">
          <div className="section-title">
            录音快捷键
            <button className="add-shortcut-btn">+</button>
          </div>
          
          {shortcuts.map((shortcut) => (
            <div key={shortcut.id} className="shortcut-row">
              <div className="shortcut-name">{shortcut.name}</div>
              <div className="shortcut-controls">
                <select 
                  value={shortcut.key} 
                  onChange={(e) => handleKeyEdit(shortcut.id, e.target.value)}
                  className="key-dropdown"
                >
                  <option value="Fn">Fn</option>
                  <option value="F13">F13</option>
                  <option value="F14">F14</option>
                  <option value="F15">F15</option>
                  <option value="CommandOrControl+Space">⌘+Space</option>
                  <option value="CommandOrControl+Shift+R">⌘+Shift+R</option>
                </select>
              </div>
            </div>
          ))}
        </div>

        <div className="hint-section">
          <div className="hint-title">
            ⚠️ 使用Fn键
          </div>
          <div className="hint-content">
            要使用Fn键:
            <br />• 打开系统设置 → 键盘
            <br />• 点击"按下键盘以下拉菜单"
            <br />• 选择"无操作"
            <br />• 这允许 Spokenly 检测Fn键按下
          </div>
        </div>

        <div className="test-section">
          <div className="test-title">测试您的快捷键</div>
          <div className="test-area">
            <button 
              className="test-btn"
              onClick={() => testShortcut(shortcuts[0])}
            >
              🎛️ 按住Fn键话，松开停止录音。
            </button>
            {testResult && (
              <div className="test-result">
                {testResult}
              </div>
            )}
          </div>
        </div>

        <div className="shortcut-footer">
          <p className="shortcut-note">
            配置快捷键是其他录音方式：按住切功能（自动录音），切功能（点击录音）或触（快速录音功能），保持（快速录音功能）。
          </p>
        </div>
      </div>
    </div>
  );
};

export default ShortcutEditor;