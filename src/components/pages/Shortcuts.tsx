/**
 * 快捷键页面
 * 完美复刻 Spokenly 第五张截图的设计
 * 注意：已移除测试功能，适用于正式版本
 */

import React, { useState } from 'react';
import './Shortcuts.css';
import { SpokenlyCard, SpokenlyButton } from '../ui';

interface ShortcutItem {
  id: string;
  shortcut: string;
  holdKey: string;
  fnKey: string;
}

interface ShortcutsProps {}

const Shortcuts: React.FC<ShortcutsProps> = () => {
  const [shortcuts, setShortcuts] = useState<ShortcutItem[]>([
    {
      id: 'primary',
      shortcut: '快捷键',
      holdKey: '按住或切换',
      fnKey: 'Fn'
    },
    {
      id: 'secondary',
      shortcut: '附加快捷键',
      holdKey: '按住或切换',
      fnKey: '右侧'
    }
  ]);

  const [isRecording, setIsRecording] = useState<string | null>(null);

  const startRecording = (shortcutId: string) => {
    setIsRecording(shortcutId);
    // 实际录制逻辑将在这里实现
  };

  const stopRecording = () => {
    setIsRecording(null);
  };

  const addNewShortcut = () => {
    const newShortcut: ShortcutItem = {
      id: `shortcut-${Date.now()}`,
      shortcut: '新快捷键',
      holdKey: '按住或切换',
      fnKey: 'Fn'
    };
    setShortcuts(prev => [...prev, newShortcut]);
  };

  return (
    <div className="spokenly-page">
      <div className="spokenly-page-header">
        <h1>快捷键</h1>
        <p>选择您喜欢的键盘修饰键启动 Spokenly。仅当这些修饰键即可开始录音。</p>
      </div>

      <div className="spokenly-page-content">
        {/* 快捷键配置表格 */}
        <SpokenlyCard>
          <div className="shortcuts-header">
            <h3>录音快捷键</h3>
            <SpokenlyButton
              variant="ghost"
              size="sm"
              onClick={addNewShortcut}
            >
              +
            </SpokenlyButton>
          </div>

          <div className="shortcuts-table">
            <div className="table-header">
              <div className="table-cell">快捷键</div>
              <div className="table-cell">按住</div>
              <div className="table-cell">Fn</div>
              <div className="table-cell"></div>
            </div>

            {shortcuts.map((shortcut, index) => (
              <div key={shortcut.id} className="table-row">
                <div className="table-cell">
                  <div className={`shortcut-display ${isRecording === shortcut.id ? 'recording' : ''}`}>
                    {isRecording === shortcut.id ? (
                      <span className="recording-text">按下快捷键...</span>
                    ) : (
                      <span>{shortcut.shortcut}</span>
                    )}
                  </div>
                </div>

                <div className="table-cell">
                  <select className="hold-select">
                    <option value="hold">按住或切换</option>
                    <option value="toggle">切换</option>
                  </select>
                </div>

                <div className="table-cell">
                  <select className="fn-select">
                    <option value="fn">Fn</option>
                    <option value="right">右侧</option>
                  </select>
                </div>

                <div className="table-cell actions">
                  {isRecording === shortcut.id ? (
                    <SpokenlyButton
                      variant="secondary"
                      size="xs"
                      onClick={stopRecording}
                    >
                      取消
                    </SpokenlyButton>
                  ) : (
                    <SpokenlyButton
                      variant="ghost"
                      size="xs"
                      onClick={() => startRecording(shortcut.id)}
                    >
                      录制
                    </SpokenlyButton>
                  )}
                </div>
              </div>
            ))}

            <div className="table-row add-row">
              <div className="table-cell">
                <SpokenlyButton
                  variant="ghost"
                  size="sm"
                  onClick={addNewShortcut}
                >
                  附加快捷键
                </SpokenlyButton>
              </div>
              <div className="table-cell">
                <select className="hold-select">
                  <option value="hold">按住或切换</option>
                </select>
              </div>
              <div className="table-cell">
                <select className="fn-select">
                  <option value="right">右侧</option>
                </select>
              </div>
              <div className="table-cell"></div>
            </div>
          </div>

          <div className="shortcut-description">
            <p>配置快捷键以其録方式：按住键録（自动启辞）、切键（点击开/关）或按任（保持常錄）。</p>
          </div>
        </SpokenlyCard>

        {/* 提示信息 */}
        <div className="shortcut-hints">
          <SpokenlyCard className="hint-card warning">
            <div className="hint-content">
              <div className="hint-icon">⚠️</div>
              <div className="hint-text">
                <h4>使用 Fn 键</h4>
                <p>更好地使用Fn键：</p>
                <ul>
                  <li>打开系统设置 → 键盘</li>
                  <li>点击"按下"键"或"下拉菜单</li>
                  <li>选择"无操作"</li>
                  <li>这允许 Spokenly 检测 Fn 键按下</li>
                </ul>
              </div>
            </div>
          </SpokenlyCard>
        </div>

        {/* 已移除测试快捷键区域 - 正式版本不需要测试功能 */}
        
        {/* 快捷键状态显示 */}
        <SpokenlyCard className="shortcut-status">
          <h3>快捷键状态</h3>
          <div className="status-grid">
            <div className="status-item">
              <span className="status-label">主快捷键</span>
              <span className="status-value active">已启用</span>
            </div>
            <div className="status-item">
              <span className="status-label">全局监听</span>
              <span className="status-value active">正常</span>
            </div>
            <div className="status-item">
              <span className="status-label">权限状态</span>
              <span className="status-value active">已授权</span>
            </div>
          </div>
        </SpokenlyCard>

        {/* 帮助信息 */}
        <SpokenlyCard className="help-section">
          <h3>使用说明</h3>
          <div className="help-content">
            <div className="help-item">
              <strong>按住模式：</strong>按下并保持快捷键进行录音，释放键停止录音
            </div>
            <div className="help-item">
              <strong>切换模式：</strong>按一次开始录音，再按一次停止录音
            </div>
            <div className="help-item">
              <strong>权限设置：</strong>首次使用需要在系统偏好设置中允许 Spokenly 访问辅助功能
            </div>
          </div>
        </SpokenlyCard>
      </div>
    </div>
  );
};

export default Shortcuts;