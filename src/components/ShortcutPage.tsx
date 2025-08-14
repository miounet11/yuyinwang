import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import { shortcutManager } from '../utils/shortcutManager';

const ShortcutPage: React.FC = () => {
  const [currentShortcuts, setCurrentShortcuts] = useState(() => {
    return shortcutManager.getShortcuts();
  });
  
  const [recording, setRecording] = useState<string | null>(null);
  const [testMessage, setTestMessage] = useState<string>('');
  const [isRecording, setIsRecording] = useState<boolean>(false);
  const [transcribedText, setTranscribedText] = useState<string>('');
  const [lastEntryId, setLastEntryId] = useState<string>('');

  // 监听转录事件，更新测试区文本
  useEffect(() => {
    let unlisten1: (() => void) | undefined;
    let unlisten2: (() => void) | undefined;

    const setup = async () => {
      try {
        unlisten1 = await listen<any>('transcription_result', (event) => {
          const entry = event.payload as { id?: string; text?: string };
          if (entry && entry.text && entry.id && entry.id !== lastEntryId) {
            setLastEntryId(entry.id);
            setTranscribedText(entry.text);
            setTestMessage('✅ 转写完成');
          }
        });
        unlisten2 = await listen<any>('file_transcription_result', (event) => {
          const entry = event.payload as { id?: string; text?: string };
          if (entry && entry.text && entry.id && entry.id !== lastEntryId) {
            setLastEntryId(entry.id);
            setTranscribedText(entry.text);
            setTestMessage('✅ 文件转写完成');
          }
        });
      } catch (e) {
        // 忽略
      }
    };

    setup();
    return () => {
      try { unlisten1 && unlisten1(); } catch {}
      try { unlisten2 && unlisten2(); } catch {}
    };
  }, [lastEntryId]);

  // 按住开始录音，松开停止并转写
  const startHoldToRecord = async () => {
    if (isRecording) return;
    try {
      setTestMessage('🎤 正在录音…按住按钮说话');
      setTranscribedText('');
      setLastEntryId('');
      await invoke('start_recording');
      setIsRecording(true);
    } catch (error) {
      setTestMessage('❌ 开始录音失败');
    }
  };

  const stopHoldToRecord = async () => {
    if (!isRecording) return;
    try {
      setTestMessage('⏳ 正在转写，请稍候…');
      await invoke('stop_recording');
    } catch (error) {
      setTestMessage('❌ 停止录音失败');
    } finally {
      setIsRecording(false);
    }
  };
  
  // 获取录音快捷键
  const recordingShortcuts = currentShortcuts.filter(s => s.category === 'recording');
  const mainShortcut = recordingShortcuts.find(s => s.id === 'toggle-recording') || recordingShortcuts[0];
  const secondaryShortcut = recordingShortcuts.find(s => s.id === 'quick-transcribe') || recordingShortcuts[1];

  const handleKeyChange = (shortcutId: string, newKey: string) => {
    if (shortcutManager.updateShortcut(shortcutId, newKey)) {
      setCurrentShortcuts(shortcutManager.getShortcuts());
      setTestMessage(`✅ 已更新快捷键: ${newKey}`);
      setTimeout(() => setTestMessage(''), 3000);
    } else {
      setTestMessage(`❌ 无法更新快捷键，可能已被占用`);
      setTimeout(() => setTestMessage(''), 3000);
    }
  };

  const handleKeyRecord = (shortcutId: string) => {
    setRecording(shortcutId);
    setTestMessage('🎤 按下你想要的键组合...');
    
    const handleKeyDown = (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();
      
      const keys = [];
      if (e.metaKey) keys.push('CommandOrControl');
      if (e.shiftKey) keys.push('Shift');
      if (e.altKey) keys.push('Alt');
      if (e.ctrlKey && !e.metaKey) keys.push('Control');
      
      // 主键
      if (e.key === 'Escape') {
        setRecording(null);
        setTestMessage('❌ 已取消');
        document.removeEventListener('keydown', handleKeyDown);
        return;
      }
      
      let mainKey = e.key;
      if (e.key === ' ') mainKey = 'Space';
      if (e.key === 'Enter') mainKey = 'Return';
      if (e.key === 'Delete') mainKey = 'Delete';
      if (e.key === 'Backspace') mainKey = 'Backspace';
      if (e.key.startsWith('F') && e.key.length <= 3) mainKey = e.key;
      if (e.key.length === 1 && e.key.match(/[a-zA-Z]/)) mainKey = e.key.toUpperCase();
      
      if (keys.length === 0 && !['Space', 'Return', 'Delete', 'Backspace'].includes(mainKey) && !mainKey.startsWith('F')) {
        setTestMessage('⚠️ 需要至少一个修饰键 (⌘, Shift, Alt, Ctrl)');
        return;
      }
      
      if (mainKey !== 'Meta' && mainKey !== 'Shift' && mainKey !== 'Alt' && mainKey !== 'Control') {
        keys.push(mainKey);
      }
      
      const keyCombo = keys.join('+');
      if (keyCombo && keyCombo !== 'CommandOrControl' && keyCombo !== 'Shift' && keyCombo !== 'Alt' && keyCombo !== 'Control') {
        handleKeyChange(shortcutId, keyCombo);
        setRecording(null);
        document.removeEventListener('keydown', handleKeyDown);
      }
    };
    
    document.addEventListener('keydown', handleKeyDown);
    
    // 5秒后自动取消
    setTimeout(() => {
      if (recording === shortcutId) {
        setRecording(null);
        setTestMessage('⏰ 录制超时，已取消');
        document.removeEventListener('keydown', handleKeyDown);
      }
    }, 5000);
  };

  const testShortcut = async () => {
    if (mainShortcut) {
      setTestMessage(`🎛️ 测试快捷键 ${mainShortcut.key} - 按下试试！`);
      
      // 临时注册测试快捷键
      try {
        await shortcutManager.registerShortcut(mainShortcut.id);
        setTimeout(() => {
          setTestMessage('✅ 快捷键可以正常工作！');
          setTimeout(() => setTestMessage(''), 2000);
        }, 1000);
      } catch (error) {
        setTestMessage('❌ 快捷键测试失败，请检查权限设置');
      }
    }
  };

  return (
    <div className="page-content">
      <div className="page-header">
        <h1>快捷键</h1>
        <p>选择您喜欢的键盘操作键来启动 Spokenly，仅按这些键即可开启录音。</p>
      </div>

      <div className="section">
        <div className="section-title">
          <h2>录音快捷键</h2>
          <button 
            className="add-shortcut-btn"
            onClick={() => {
              // TODO: 添加新快捷键功能
              setTestMessage('添加新快捷键功能待实现');
              setTimeout(() => setTestMessage(''), 2000);
            }}
          >
            +
          </button>
        </div>
        
        <div className="shortcut-list">
          {mainShortcut && (
            <div className="shortcut-row">
              <div className="shortcut-name">快捷键</div>
              <div className="shortcut-controls">
                <select 
                  className="key-dropdown"
                  defaultValue="按住或切功能"
                >
                  <option>按住或切功能</option>
                  <option>单击</option>
                  <option>双击</option>
                </select>
                <select 
                  className="key-dropdown"
                  value={mainShortcut.key}
                  onChange={(e) => handleKeyChange(mainShortcut.id, e.target.value)}
                >
                  <option value="Fn">Fn</option>
                  <option value="CommandOrControl+Space">⌘+Space</option>
                  <option value="CommandOrControl+Shift+R">⌘+Shift+R</option>
                  <option value="F13">F13</option>
                  <option value="F14">F14</option>
                  <option value="F15">F15</option>
                </select>
                <button 
                  className="key-record-btn"
                  onClick={() => handleKeyRecord(mainShortcut.id)}
                  disabled={recording !== null}
                >
                  {recording === mainShortcut.id ? '录制中...' : '🎤 录制'}
                </button>
              </div>
            </div>
          )}
          
          {secondaryShortcut && (
            <div className="shortcut-row">
              <div className="shortcut-name">附加快捷键</div>
              <div className="shortcut-controls">
                <select 
                  className="key-dropdown"
                  defaultValue="按住或功能"
                >
                  <option>按住或功能</option>
                  <option>单击</option>
                  <option>双击</option>
                </select>
                <select 
                  className="key-dropdown"
                  value={secondaryShortcut.key}
                  onChange={(e) => handleKeyChange(secondaryShortcut.id, e.target.value)}
                >
                  <option value="CommandOrControl+Shift+Space">⌘+Shift+Space</option>
                  <option value="CommandOrControl+Shift+S">⌘+Shift+S</option>
                  <option value="Alt+Space">⌥+Space</option>
                  <option value="F16">F16</option>
                </select>
                <button 
                  className="key-record-btn"
                  onClick={() => handleKeyRecord(secondaryShortcut.id)}
                  disabled={recording !== null}
                >
                  {recording === secondaryShortcut.id ? '录制中...' : '🎤 录制'}
                </button>
              </div>
            </div>
          )}
        </div>
        
        <p className="shortcut-description">
          配置快捷键或其他录音方式：按住切功能（自动录音），切功能（点击录音）或触（快速录音功能），保持（快速录音功能）。
        </p>
      </div>

      <div className="section">
        <div className="hint-section">
          <div className="hint-title">⚠️ 使用Fn键</div>
          <div className="hint-content">
            要使用Fn键:
            <br />• 打开系统设置 → 键盘
            <br />• 点击"按下键盘以下拉菜单"
            <br />• 选择"无操作"
            <br />• 这允许 Spokenly 检测Fn键按下
          </div>
        </div>
      </div>

      <div className="section">
        <div className="test-section">
          <h2>测试您的快捷键 / 按住说话</h2>
          <div className="test-area">
            <div style={{ display: 'flex', gap: 12, flexWrap: 'wrap' }}>
              <button
                className={`test-btn ${isRecording ? 'recording' : ''}`}
                onMouseDown={startHoldToRecord}
                onMouseUp={stopHoldToRecord}
                onMouseLeave={() => { if (isRecording) stopHoldToRecord(); }}
                onTouchStart={startHoldToRecord}
                onTouchEnd={stopHoldToRecord}
                title="按住开始录音，松开后自动转文字"
              >
                {isRecording ? '松开停止并转写' : '按住开始说话'}
              </button>
              <button className="test-btn" onClick={testShortcut} title="验证快捷键是否被注册">
                测试快捷键是否触发
              </button>
            </div>
            {testMessage && (
              <div className="test-message">
                {testMessage}
              </div>
            )}
            <div className="test-textarea-container">
              <textarea
                placeholder="在这里查看转写结果，或聚焦后按快捷键测试…"
                className="test-textarea"
                value={transcribedText}
                readOnly
                onFocus={() => setTestMessage('✨ 聚焦后按快捷键或按住按钮说话')}
              />
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default ShortcutPage;