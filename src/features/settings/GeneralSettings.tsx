import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { useAppStore } from '../../shared/stores/useAppStore';

export const GeneralSettings: React.FC = () => {
  const { settings, audioDevices, setSettings, setAudioDevices, addToast } = useAppStore();
  const [globalShortcut, setGlobalShortcut] = useState('CommandOrControl+Shift+Space');
  const [isShortcutRegistered, setIsShortcutRegistered] = useState(false);
  const [playSound, setPlaySound] = useState(true);
  const [muteOnRecord, setMuteOnRecord] = useState(false);
  const [isRecordingShortcut, setIsRecordingShortcut] = useState(false);

  useEffect(() => {
    loadSettings();
    loadDevices();
  }, []);

  const loadSettings = async () => {
    try {
      const s = await invoke('get_settings') as any;
      setSettings(s);
      if (s.shortcut_key) {
        setGlobalShortcut(s.shortcut_key);
        setIsShortcutRegistered(true);
      }
    } catch (e) { console.error(e); }
  };

  const loadDevices = async () => {
    try {
      const devices = await invoke('get_audio_devices');
      setAudioDevices(devices as any[]);
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

  const handleRegisterShortcut = async () => {
    try {
      await invoke('register_global_shortcut', { key: globalShortcut });
      setIsShortcutRegistered(true);
      const updated = { ...settings, shortcut_key: globalShortcut };
      setSettings(updated);
      addToast('success', '快捷键已启用');
    } catch (e) {
      addToast('error', '快捷键注册失败: ' + e);
    }
  };

  const handleUnregisterShortcut = async () => {
    try {
      await invoke('unregister_global_shortcut', { key: globalShortcut });
      setIsShortcutRegistered(false);
      addToast('success', '快捷键已停用');
    } catch (e) { console.error(e); }
  };

  const handleShortcutKeyDown = (e: React.KeyboardEvent) => {
    if (!isRecordingShortcut) return;
    e.preventDefault();
    const parts: string[] = [];
    if (e.metaKey || e.ctrlKey) parts.push('CommandOrControl');
    if (e.shiftKey) parts.push('Shift');
    if (e.altKey) parts.push('Alt');
    const key = e.key;
    if (!['Meta', 'Control', 'Shift', 'Alt'].includes(key)) {
      parts.push(key.length === 1 ? key.toUpperCase() : key);
      setGlobalShortcut(parts.join('+'));
      setIsRecordingShortcut(false);
    }
  };

  return (
    <div className="page">
      <h1 className="page-title">常规设置</h1>
      <p className="page-desc">配置应用行为、快捷键和音频设备</p>

      <div className="section">
        <h2 className="section-title">行为设置</h2>
        <div className="card">
          <div className="card-row">
            <div>
              <span>自动注入转录文本</span>
              <div className="card-row-sub">转录完成后自动将文本输入到当前应用</div>
            </div>
            <button
              className={`toggle ${settings.auto_inject ? 'on' : ''}`}
              onClick={() => handleToggle('auto_inject', !settings.auto_inject)}
            />
          </div>
          <div className="card-row">
            <div>
              <span>注入延迟</span>
              <div className="card-row-sub">注入文本前的等待时间 ({settings.inject_delay_ms}ms)</div>
            </div>
            <input
              type="range"
              min="0"
              max="500"
              step="50"
              value={settings.inject_delay_ms}
              onChange={(e) => {
                const val = parseInt(e.target.value);
                const updated = { ...settings, inject_delay_ms: val };
                setSettings(updated);
                invoke('update_settings', { settings: updated }).catch(console.error);
              }}
              style={{
                width: '120px', accentColor: 'var(--accent)',
              }}
            />
          </div>
        </div>
      </div>

      <div className="section">
        <h2 className="section-title">按住说话快捷键</h2>
        <p className="section-desc">按住快捷键开始录音，松开自动转录并注入文字</p>
        <div className="card">
          <div className="card-row">
            <div style={{ flex: 1 }}>
              <input
                type="text"
                value={isRecordingShortcut ? '请按下快捷键组合...' : globalShortcut}
                readOnly
                onFocus={() => setIsRecordingShortcut(true)}
                onBlur={() => setIsRecordingShortcut(false)}
                onKeyDown={handleShortcutKeyDown}
                style={{
                  background: isRecordingShortcut ? 'rgba(59,130,246,0.1)' : 'var(--bg-tertiary)',
                  border: `1px solid ${isRecordingShortcut ? 'var(--accent)' : 'var(--border)'}`,
                  borderRadius: '6px', padding: '8px 12px', color: 'var(--text-primary)',
                  fontSize: '13px', width: '100%', outline: 'none',
                  cursor: 'pointer', transition: 'all 0.15s',
                }}
                placeholder="点击此处录入快捷键"
              />
            </div>
            {isShortcutRegistered ? (
              <button onClick={handleUnregisterShortcut} style={{
                padding: '8px 16px', background: 'var(--danger)', color: '#fff',
                border: 'none', borderRadius: '6px', fontSize: '12px', fontWeight: 600,
                cursor: 'pointer', marginLeft: '12px', transition: 'opacity 0.15s',
              }}>停用</button>
            ) : (
              <button onClick={handleRegisterShortcut} style={{
                padding: '8px 16px', background: 'var(--accent)', color: '#fff',
                border: 'none', borderRadius: '6px', fontSize: '12px', fontWeight: 600,
                cursor: 'pointer', marginLeft: '12px', transition: 'opacity 0.15s',
              }}>启用</button>
            )}
          </div>
          {isShortcutRegistered && (
            <div className="card-row" style={{ color: 'var(--success)', fontSize: '12px' }}>
              ✓ 快捷键已激活 — 在任意应用中按住 {globalShortcut.replace('CommandOrControl', '⌘').replace('Shift', '⇧').replace('Alt', '⌥').replace('+', ' ')} 即可语音输入
            </div>
          )}
        </div>
      </div>

      <div className="section">
        <h2 className="section-title">麦克风优先级</h2>
        <p className="section-desc">麦克风将按优先级顺序使用</p>
        <div className="card">
          {audioDevices.length === 0 ? (
            <div className="card-row" style={{ color: 'var(--text-muted)', justifyContent: 'center' }}>
              <span style={{ marginRight: '8px' }}>🎙</span> 未检测到音频设备
            </div>
          ) : (
            audioDevices.map((device, i) => (
              <div key={device.id} className="card-row">
                <div className="card-row-label">
                  <span style={{
                    color: device.is_default ? 'var(--accent)' : 'var(--text-muted)',
                    fontSize: '12px', width: '20px', fontWeight: device.is_default ? 600 : 400,
                  }}>{i + 1}</span>
                  <span>{device.name}</span>
                </div>
                {device.is_default && (
                  <span style={{
                    padding: '2px 8px', borderRadius: '4px', fontSize: '10px', fontWeight: 600,
                    background: 'rgba(59,130,246,0.15)', color: 'var(--accent)',
                  }}>默认</span>
                )}
              </div>
            ))
          )}
        </div>
      </div>

      <div className="section">
        <h2 className="section-title">音频与反馈</h2>
        <div className="card">
          <div className="card-row">
            <div>
              <span>播放音效</span>
              <div className="card-row-sub">录音开始和结束时播放提示音</div>
            </div>
            <button
              className={`toggle ${playSound ? 'on' : ''}`}
              onClick={() => setPlaySound(!playSound)}
            />
          </div>
          <div className="card-row">
            <div>
              <span>录音时静音</span>
              <div className="card-row-sub">录音期间静音系统音频避免干扰</div>
            </div>
            <button
              className={`toggle ${muteOnRecord ? 'on' : ''}`}
              onClick={() => setMuteOnRecord(!muteOnRecord)}
            />
          </div>
        </div>
      </div>
    </div>
  );
};
