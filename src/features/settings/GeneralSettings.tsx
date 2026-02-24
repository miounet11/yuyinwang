import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { useAppStore } from '../../shared/stores/useAppStore';
import type { AudioDevice } from '../../shared/types';

export const GeneralSettings: React.FC = () => {
  const { settings, audioDevices, setSettings, setAudioDevices, addToast, reorderMicrophones } = useAppStore();
  const [draggedIndex, setDraggedIndex] = useState<number | null>(null);
  const [dragOverIndex, setDragOverIndex] = useState<number | null>(null);

  useEffect(() => {
    loadSettings();
    loadDevices();
  }, []);

  const loadSettings = async () => {
    try {
      const s = await invoke('get_settings') as any;
      setSettings(s);
    } catch (e) { console.error(e); }
  };

  const loadDevices = async () => {
    try {
      const devices = await invoke('get_audio_devices');
      setAudioDevices(devices as AudioDevice[]);
    } catch (e) { console.error(e); }
  };

  const handleToggle = async (key: string, value: boolean) => {
    const updated = { ...settings, [key]: value };
    setSettings(updated);
    try {
      await invoke('update_settings', { settings: updated });
      addToast('success', '设置已保存');
    } catch (e) {
      console.error(e);
      addToast('error', '保存失败');
    }
  };

  const handleDisplayStyleChange = async (style: 'panel' | 'notch') => {
    const updated = { ...settings, display_style: style };
    setSettings(updated);
    try {
      await invoke('update_settings', { settings: updated });
      addToast('success', '显示样式已更新');
    } catch (e) {
      console.error(e);
      addToast('error', '保存失败');
    }
  };

  const handleAppearanceChange = async (appearance: 'system' | 'dark' | 'light') => {
    const updated = { ...settings, appearance };
    setSettings(updated);
    try {
      await invoke('update_settings', { settings: updated });
      document.documentElement.setAttribute('data-theme', appearance);
      addToast('success', '外观主题已更新');
    } catch (e) {
      console.error(e);
      addToast('error', '保存失败');
    }
  };

  const handleLanguageChange = async (language: 'system' | 'zh-CN' | 'en') => {
    const updated = { ...settings, ui_language: language };
    setSettings(updated);
    try {
      await invoke('update_settings', { settings: updated });
      addToast('success', '界面语言已更新');
    } catch (e) {
      console.error(e);
      addToast('error', '保存失败');
    }
  };

  // HTML5 Drag and Drop handlers
  const handleDragStart = (e: React.DragEvent, index: number) => {
    setDraggedIndex(index);
    e.dataTransfer.effectAllowed = 'move';
  };

  const handleDragOver = (e: React.DragEvent, index: number) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'move';
    setDragOverIndex(index);
  };

  const handleDragLeave = () => {
    setDragOverIndex(null);
  };

  const handleDrop = (e: React.DragEvent, dropIndex: number) => {
    e.preventDefault();
    if (draggedIndex !== null && draggedIndex !== dropIndex) {
      reorderMicrophones(draggedIndex, dropIndex);
      addToast('success', '麦克风优先级已更新');
    }
    setDraggedIndex(null);
    setDragOverIndex(null);
  };

  const handleDragEnd = () => {
    setDraggedIndex(null);
    setDragOverIndex(null);
  };

  return (
    <div className="page">
      <h1 className="page-title">常规设置</h1>
      <p className="page-desc">配置应用界面、行为和音频设备</p>

      {/* 界面区域 */}
      <div className="section">
        <h2 className="section-title">界面</h2>
        <div className="card">
          <div className="card-row">
            <div>
              <span>录音显示样式</span>
              <div className="card-row-sub">选择录音时的界面显示方式</div>
            </div>
            <div style={{ display: 'flex', gap: '8px' }}>
              <button
                onClick={() => handleDisplayStyleChange('panel')}
                style={{
                  padding: '6px 12px',
                  background: settings.display_style === 'panel' ? 'var(--accent)' : 'var(--bg-tertiary)',
                  color: settings.display_style === 'panel' ? '#fff' : 'var(--text-secondary)',
                  border: 'none',
                  borderRadius: '6px',
                  fontSize: '12px',
                  fontWeight: 500,
                  cursor: 'pointer',
                  transition: 'all 0.15s',
                }}
              >
                面板
              </button>
              <button
                onClick={() => handleDisplayStyleChange('notch')}
                style={{
                  padding: '6px 12px',
                  background: settings.display_style === 'notch' ? 'var(--accent)' : 'var(--bg-tertiary)',
                  color: settings.display_style === 'notch' ? '#fff' : 'var(--text-secondary)',
                  border: 'none',
                  borderRadius: '6px',
                  fontSize: '12px',
                  fontWeight: 500,
                  cursor: 'pointer',
                  transition: 'all 0.15s',
                }}
              >
                刘海
              </button>
            </div>
          </div>
          <div className="card-row">
            <div>
              <span>外观主题</span>
              <div className="card-row-sub">选择应用的颜色主题</div>
            </div>
            <select
              value={settings.appearance}
              onChange={(e) => handleAppearanceChange(e.target.value as 'system' | 'dark' | 'light')}
              style={{
                padding: '6px 12px',
                background: 'var(--bg-tertiary)',
                color: 'var(--text-primary)',
                border: '1px solid var(--border)',
                borderRadius: '6px',
                fontSize: '12px',
                cursor: 'pointer',
                outline: 'none',
              }}
            >
              <option value="system">跟随系统</option>
              <option value="dark">深色</option>
              <option value="light">浅色</option>
            </select>
          </div>
          <div className="card-row">
            <div>
              <span>界面语言</span>
              <div className="card-row-sub">选择应用的显示语言</div>
            </div>
            <select
              value={settings.ui_language}
              onChange={(e) => handleLanguageChange(e.target.value as 'system' | 'zh-CN' | 'en')}
              style={{
                padding: '6px 12px',
                background: 'var(--bg-tertiary)',
                color: 'var(--text-primary)',
                border: '1px solid var(--border)',
                borderRadius: '6px',
                fontSize: '12px',
                cursor: 'pointer',
                outline: 'none',
              }}
            >
              <option value="system">跟随系统</option>
              <option value="zh-CN">简体中文</option>
              <option value="en">English</option>
            </select>
          </div>
        </div>
      </div>

      {/* 行为区域 */}
      <div className="section">
        <h2 className="section-title">行为</h2>
        <div className="card">
          <div className="card-row">
            <div>
              <span>登录时启动</span>
              <div className="card-row-sub">系统启动时自动运行应用</div>
            </div>
            <button
              className={`toggle ${settings.launch_at_login ? 'on' : ''}`}
              onClick={() => handleToggle('launch_at_login', !settings.launch_at_login)}
            />
          </div>
          <div className="card-row">
            <div>
              <span>在程序坞中显示</span>
              <div className="card-row-sub">在 macOS 程序坞中显示应用图标</div>
            </div>
            <button
              className={`toggle ${settings.show_in_dock ? 'on' : ''}`}
              onClick={() => handleToggle('show_in_dock', !settings.show_in_dock)}
            />
          </div>
          <div className="card-row">
            <div>
              <span>在状态栏中显示</span>
              <div className="card-row-sub">在菜单栏显示应用图标</div>
            </div>
            <button
              className={`toggle ${settings.show_in_menu_bar ? 'on' : ''}`}
              onClick={() => handleToggle('show_in_menu_bar', !settings.show_in_menu_bar)}
            />
          </div>
          <div className="card-row">
            <div>
              <span>Escape 取消录音</span>
              <div className="card-row-sub">录音时按 Esc 键取消当前录音</div>
            </div>
            <button
              className={`toggle ${settings.esc_to_cancel ? 'on' : ''}`}
              onClick={() => handleToggle('esc_to_cancel', !settings.esc_to_cancel)}
            />
          </div>
        </div>
      </div>

      {/* 麦克风优先级区域 */}
      <div className="section">
        <h2 className="section-title">麦克风优先级</h2>
        <p className="section-desc">拖拽调整麦克风使用顺序，设备断开时自动切换到下一个可用设备</p>
        <div className="card">
          {audioDevices.length === 0 ? (
            <div className="card-row" style={{ color: 'var(--text-muted)', justifyContent: 'center' }}>
              <span style={{ marginRight: '8px' }}>🎙</span> 未检测到音频设备
            </div>
          ) : (
            audioDevices.map((device, i) => (
              <div
                key={device.id}
                draggable
                onDragStart={(e) => handleDragStart(e, i)}
                onDragOver={(e) => handleDragOver(e, i)}
                onDragLeave={handleDragLeave}
                onDrop={(e) => handleDrop(e, i)}
                onDragEnd={handleDragEnd}
                className="card-row"
                style={{
                  cursor: device.is_available ? 'grab' : 'not-allowed',
                  opacity: device.is_available ? 1 : 0.5,
                  background: dragOverIndex === i && draggedIndex !== i ? 'var(--bg-hover)' : 'transparent',
                  transition: 'all 0.15s',
                }}
              >
                <div className="card-row-label">
                  <span style={{
                    color: device.is_available ? 'var(--text-muted)' : 'var(--text-disabled)',
                    fontSize: '12px',
                    width: '20px',
                    fontWeight: 400,
                  }}>
                    {i + 1}
                  </span>
                  <span style={{
                    color: device.is_available ? 'var(--text-primary)' : 'var(--text-disabled)',
                  }}>
                    {device.name}
                  </span>
                  {!device.is_available && (
                    <span style={{
                      padding: '2px 6px',
                      borderRadius: '4px',
                      fontSize: '10px',
                      fontWeight: 600,
                      background: 'var(--bg-tertiary)',
                      color: 'var(--text-disabled)',
                      marginLeft: '8px',
                    }}>
                      不可用
                    </span>
                  )}
                </div>
                {device.is_default && device.is_available && (
                  <span style={{
                    padding: '2px 8px',
                    borderRadius: '4px',
                    fontSize: '10px',
                    fontWeight: 600,
                    background: 'rgba(59,130,246,0.15)',
                    color: 'var(--accent)',
                  }}>
                    默认
                  </span>
                )}
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
};
