import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { useAppStore } from '../../shared/stores/useAppStore';

interface Settings {
  openai_api_key: string | null;
  selected_model: string;
  auto_inject: boolean;
  inject_delay_ms: number;
  shortcut_key: string | null;
}

export const SettingsPage: React.FC = () => {
  const { addToast } = useAppStore();
  const [settings, setSettings] = useState<Settings>({
    openai_api_key: null,
    selected_model: 'whisper-1',
    auto_inject: false,
    inject_delay_ms: 100,
    shortcut_key: null,
  });

  useEffect(() => {
    loadSettings();
  }, []);

  const loadSettings = async () => {
    try {
      const data = await invoke<Settings>('get_settings');
      setSettings(data);
    } catch (error) {
      addToast('error', '加载设置失败');
    }
  };

  const handleSave = async () => {
    try {
      await invoke('update_settings', { settings });
      addToast('success', '设置已保存');
    } catch (error) {
      addToast('error', '保存设置失败');
    }
  };

  return (
    <div style={{ padding: '20px' }}>
      <h1>设置</h1>
      <div style={{ marginBottom: '16px' }}>
        <label>OpenAI API Key:</label>
        <input
          type="password"
          value={settings.openai_api_key || ''}
          onChange={(e) => setSettings({ ...settings, openai_api_key: e.target.value })}
          style={{ width: '100%', padding: '8px', marginTop: '4px' }}
        />
      </div>
      <button onClick={handleSave} style={{ padding: '10px 20px' }}>
        保存
      </button>
    </div>
  );
};
