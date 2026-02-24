import React, { useEffect, useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import { useAppStore } from '../../shared/stores/useAppStore';
import './SettingsPage.css';

interface ModelInfo {
  id: string;
  name: string;
  icon: string;
  provider: string;
  description: string;
  accuracy: number;
  speed: number;
  languages: string;
  type: 'online' | 'local';
  free?: boolean;
  size?: string;
  keyType?: 'openai' | 'luyin';
  badge?: string;
  available: boolean;
  unavailableReason?: string;
}

const MODELS: ModelInfo[] = [
  { id: 'luyin-free', name: 'LuYinWang Transcribe', icon: 'LW', provider: '录音王',
    description: '由录音王驱动 - 卓越准确性和快速处理。需配置 Token。',
    accuracy: 5, speed: 4, languages: '多语言', type: 'online', free: true,
    badge: '推荐', keyType: 'luyin', available: true },
  { id: 'gpt-4o-mini-transcribe', name: 'GPT-4o mini', icon: '🌀', provider: 'OpenAI',
    description: '卓越准确性和快速处理。', accuracy: 5, speed: 4, languages: '多语言',
    type: 'online', keyType: 'openai', badge: '最准确', available: true },
  { id: 'deepgram-nova3', name: 'Nova-3 (English)', icon: 'D', provider: 'Deepgram',
    description: '实时听写，纯英语优化。', accuracy: 4, speed: 5, languages: '仅英语',
    type: 'online', keyType: 'openai', badge: '最快',
    available: false, unavailableReason: '接口开发中' },
  { id: 'voxtral-mini', name: 'Voxtral Mini', icon: 'M', provider: 'Mistral AI',
    description: '快速准确，出色多语言支持。', accuracy: 4, speed: 4, languages: '多语言',
    type: 'online', keyType: 'openai',
    available: false, unavailableReason: '接口开发中' },
  { id: 'elevenlabs-scribe', name: 'ElevenLabs Scribe', icon: 'II', provider: 'ElevenLabs',
    description: '高质量转录，先进语音识别。', accuracy: 4, speed: 4, languages: '多语言',
    type: 'online', keyType: 'openai',
    available: false, unavailableReason: '接口开发中' },
  { id: 'whisper-tiny', name: 'Whisper Tiny', icon: '🎯', provider: 'OpenAI Whisper',
    description: '最小最快，适合快速草稿。', accuracy: 2, speed: 5, languages: '多语言',
    type: 'local', size: '75 MB', available: true },
  { id: 'whisper-base', name: 'Whisper Base', icon: '📝', provider: 'OpenAI Whisper',
    description: '基础模型，速度与准确度平衡。', accuracy: 3, speed: 4, languages: '多语言',
    type: 'local', size: '148 MB', available: true },
  { id: 'whisper-small', name: 'Whisper Small', icon: '🎤', provider: 'OpenAI Whisper',
    description: '准确性和速度平衡，推荐本地模型。', accuracy: 4, speed: 3, languages: '多语言',
    type: 'local', size: '488 MB', badge: '本地推荐', available: true },
  { id: 'whisper-medium', name: 'Whisper Medium', icon: '🔊', provider: 'OpenAI Whisper',
    description: '高准确度，适合专业场景。', accuracy: 4, speed: 2, languages: '多语言',
    type: 'local', size: '1.5 GB', available: true },
  { id: 'whisper-large-v3', name: 'Whisper Large v3', icon: '🏆', provider: 'OpenAI Whisper',
    description: '最高准确度，99种语言支持。', accuracy: 5, speed: 1, languages: '99种语言',
    type: 'local', size: '3.1 GB', available: true },
  { id: 'whisper-large-v3-turbo', name: 'Large v3 Turbo', icon: '🚀', provider: 'OpenAI Whisper',
    description: '优化版，速度提升2倍，准确度接近 Large。', accuracy: 5, speed: 3, languages: '99种语言',
    type: 'local', size: '1.6 GB', badge: '本地最佳', available: true },
];

const DotBar: React.FC<{ value: number; max?: number }> = ({ value, max = 5 }) => (
  <span className="dot-bar">
    {Array.from({ length: max }, (_, i) => (
      <span key={i} className={`dot ${i < value ? 'filled' : ''}`} />
    ))}
  </span>
);

interface DownloadState {
  [modelId: string]: { progress: number; downloading: boolean };
}

export const SettingsPage: React.FC = () => {
  const { settings, setSettings, addToast } = useAppStore();
  const [modelFilter, setModelFilter] = useState<string>('all');
  const [showApiConfig, setShowApiConfig] = useState<string | null>(null);
  const [apiKeyInput, setApiKeyInput] = useState('');
  const [configType, setConfigType] = useState<'openai' | 'luyin'>('openai');
  const [downloadedModels, setDownloadedModels] = useState<Set<string>>(new Set());
  const [downloads, setDownloads] = useState<DownloadState>({});

  // 加载设置和本地模型状态
  useEffect(() => {
    invoke('get_settings').then((s: any) => setSettings(s)).catch(console.error);
    refreshModelStatus();
  }, []);

  const refreshModelStatus = useCallback(() => {
    invoke<Array<{ model_id: string; downloaded: boolean }>>('get_local_model_status')
      .then((statuses) => {
        const downloaded = new Set<string>();
        statuses.forEach((s) => { if (s.downloaded) downloaded.add(s.model_id); });
        setDownloadedModels(downloaded);
      })
      .catch(console.error);
  }, []);

  // 监听下载进度事件
  useEffect(() => {
    const unlisten = listen<{ model_id: string; progress: number }>('model-download-progress', (event) => {
      const { model_id, progress } = event.payload;
      setDownloads((prev) => ({
        ...prev,
        [model_id]: { progress, downloading: progress < 1 },
      }));
      if (progress >= 1) {
        setDownloadedModels((prev) => new Set([...prev, model_id]));
        addToast('success', `${MODELS.find(m => m.id === model_id)?.name || model_id} 下载完成`);
      }
    });
    return () => { unlisten.then((fn) => fn()); };
  }, []);

  const handleDownloadModel = async (modelId: string, e: React.MouseEvent) => {
    e.stopPropagation();
    setDownloads((prev) => ({ ...prev, [modelId]: { progress: 0, downloading: true } }));
    try {
      await invoke('download_local_model', { modelId });
    } catch (err: any) {
      addToast('error', `下载失败: ${err}`);
      setDownloads((prev) => ({ ...prev, [modelId]: { progress: 0, downloading: false } }));
    }
  };

  const handleDeleteModel = async (modelId: string, e: React.MouseEvent) => {
    e.stopPropagation();
    try {
      await invoke('delete_local_model', { modelId });
      setDownloadedModels((prev) => { const next = new Set(prev); next.delete(modelId); return next; });
      if (settings.selected_model === modelId) {
        applyModel('luyin-free');
      }
      addToast('success', '模型已删除');
    } catch (err: any) {
      addToast('error', `删除失败: ${err}`);
    }
  };

  const handleSelectModel = (model: ModelInfo) => {
    if (!model.available) {
      addToast('warning', model.unavailableReason || '该模型暂不可用');
      return;
    }
    // 本地模型需要先下载
    if (model.type === 'local' && !downloadedModels.has(model.id)) {
      addToast('warning', '请先下载模型');
      return;
    }
    if (model.keyType === 'luyin' && !settings.luyin_token) {
      setConfigType('luyin');
      setShowApiConfig(model.id);
      setApiKeyInput(settings.luyin_token || '');
      return;
    }
    if (model.keyType === 'openai' && !settings.openai_api_key) {
      setConfigType('openai');
      setShowApiConfig(model.id);
      setApiKeyInput(settings.openai_api_key || '');
      return;
    }
    applyModel(model.id);
  };

  const applyModel = async (id: string) => {
    const updated = { ...settings, selected_model: id };
    setSettings(updated);
    try {
      await invoke('update_settings', { settings: updated });
      addToast('success', `已切换到 ${MODELS.find(m => m.id === id)?.name || id}`);
    } catch (e) {
      addToast('error', '保存失败');
    }
  };

  const handleApiSave = async (modelId: string) => {
    if (!apiKeyInput.trim()) return;
    const keyField = configType === 'luyin' ? 'luyin_token' : 'openai_api_key';
    const updated = { ...settings, [keyField]: apiKeyInput.trim(), selected_model: modelId };
    setSettings(updated);
    try {
      await invoke('update_settings', { settings: updated });
      addToast('success', `${configType === 'luyin' ? '录音王 Token' : 'API Key'} 已保存`);
    } catch (e) {
      addToast('error', '保存失败');
    }
    setShowApiConfig(null);
    setApiKeyInput('');
  };

  const filters = [
    { key: 'all', label: '全部' },
    { key: 'available', label: '可用' },
    { key: 'online', label: '在线' },
    { key: 'local', label: '本地' },
  ];

  const filtered = MODELS.filter((m) => {
    if (modelFilter === 'all') return true;
    if (modelFilter === 'available') return m.available;
    if (modelFilter === 'online') return m.type === 'online';
    if (modelFilter === 'local') return m.type === 'local';
    return true;
  });

  const getAction = (m: ModelInfo) => {
    if (!m.available) return { label: m.unavailableReason || '不可用', cls: 'unavailable' };
    if (m.type === 'local') {
      const dl = downloads[m.id];
      if (dl?.downloading) return { label: `下载中 ${Math.round(dl.progress * 100)}%`, cls: 'downloading' };
      if (!downloadedModels.has(m.id)) return { label: '⬇ 下载', cls: 'download' };
      if (settings.selected_model === m.id) return { label: '✓ 使用中', cls: 'active' };
      return { label: '使用', cls: 'use' };
    }
    if (settings.selected_model === m.id) return { label: '✓ 使用中', cls: 'active' };
    if (m.keyType === 'luyin' && !settings.luyin_token) return { label: '🔑 配置 Token', cls: 'config' };
    if (m.keyType === 'openai' && !settings.openai_api_key) return { label: '🔑 配置 Key', cls: 'config' };
    return { label: '使用', cls: 'use' };
  };

  const hasLuyinToken = !!settings.luyin_token;
  const hasOpenaiKey = !!settings.openai_api_key;

  return (
    <div className="settings-page">
      <div className="page-header">
        <h1>听写模型</h1>
        <p>选择最适合您需求的准确性、隐私性和速度的平衡点</p>
      </div>

      {/* Token 状态概览 */}
      <div className="settings-section">
        <div style={{ display: 'flex', gap: '10px', marginBottom: '20px', flexWrap: 'wrap' }}>
          <div className={`token-badge ${hasLuyinToken ? 'ok' : 'missing'}`}
            onClick={() => { setConfigType('luyin'); setShowApiConfig('luyin-free'); setApiKeyInput(settings.luyin_token || ''); }}>
            <span className="token-dot" />
            录音王 Token {hasLuyinToken ? '✓' : '未配置'}
          </div>
          <div className={`token-badge ${hasOpenaiKey ? 'ok' : 'missing'}`}
            onClick={() => { setConfigType('openai'); setShowApiConfig('gpt-4o-mini-transcribe'); setApiKeyInput(settings.openai_api_key || ''); }}>
            <span className="token-dot" />
            OpenAI Key {hasOpenaiKey ? '✓' : '未配置'}
          </div>
        </div>
      </div>

      <div className="settings-section">
        <div className="model-filters">
          {filters.map((f) => (
            <button key={f.key} className={`filter-btn ${modelFilter === f.key ? 'active' : ''}`}
              onClick={() => setModelFilter(f.key)}>
              {f.label}
              <span className="filter-count">
                {f.key === 'all' ? MODELS.length : MODELS.filter(m => {
                  if (f.key === 'available') return m.available;
                  if (f.key === 'online') return m.type === 'online';
                  if (f.key === 'local') return m.type === 'local';
                  return false;
                }).length}
              </span>
            </button>
          ))}
        </div>

        <div className="model-grid">
          {filtered.map((model) => {
            const action = getAction(model);
            const dl = downloads[model.id];
            const isDownloaded = downloadedModels.has(model.id);
            return (
              <div key={model.id}
                className={`model-card ${settings.selected_model === model.id ? 'selected' : ''} ${!model.available ? 'disabled' : ''}`}
                onClick={() => handleSelectModel(model)}>
                <div className="model-header">
                  <span className="model-icon">{model.icon}</span>
                  <div className="model-title">
                    <span className="model-name">{model.name}</span>
                    {model.badge && <span className="tag badge-tag">{model.badge}</span>}
                    {model.free && <span className="tag free">免费</span>}
                    {model.type === 'local' && isDownloaded && <span className="tag downloaded">已下载</span>}
                  </div>
                </div>
                <p className="model-provider">{model.provider}{model.size ? ` · ${model.size}` : ''}</p>
                <p className="model-desc">{model.description}</p>
                <div className="model-stats">
                  <div className="stat-row"><span className="stat-label">准确度</span><DotBar value={model.accuracy} /></div>
                  <div className="stat-row"><span className="stat-label">速度</span><DotBar value={model.speed} /></div>
                  <div className="stat-row"><span className="stat-label">语言</span><span className="stat-value">{model.languages}</span></div>
                </div>
                {/* 下载进度条 */}
                {dl?.downloading && (
                  <div className="download-progress">
                    <div className="download-bar" style={{ width: `${dl.progress * 100}%` }} />
                  </div>
                )}
                <div className="model-card-actions">
                  {model.type === 'local' && !dl?.downloading && !isDownloaded && (
                    <button className="model-action download" onClick={(e) => handleDownloadModel(model.id, e)}>⬇ 下载</button>
                  )}
                  {model.type === 'local' && isDownloaded && settings.selected_model !== model.id && (
                    <button className="model-action use" onClick={(e) => { e.stopPropagation(); applyModel(model.id); }}>使用</button>
                  )}
                  {model.type === 'local' && isDownloaded && (
                    <button className="model-action delete" onClick={(e) => handleDeleteModel(model.id, e)}>删除</button>
                  )}
                  {model.type !== 'local' && (
                    <div className={`model-action ${action.cls}`}>{action.label}</div>
                  )}
                  {model.type === 'local' && dl?.downloading && (
                    <div className="model-action downloading">{action.label}</div>
                  )}
                  {model.type === 'local' && !isDownloaded && !dl?.downloading && (
                    <span className="model-action-hint">需要下载后使用</span>
                  )}
                  {settings.selected_model === model.id && model.type === 'local' && isDownloaded && (
                    <div className="model-action active">✓ 使用中</div>
                  )}
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {showApiConfig && (
        <div className="modal-overlay" onClick={() => setShowApiConfig(null)}>
          <div className="modal-content" onClick={e => e.stopPropagation()}>
            <h3>{configType === 'luyin' ? '🔑 配置录音王 Token' : '🔑 配置 OpenAI API Key'}</h3>
            <p className="modal-desc">
              {configType === 'luyin'
                ? '从 record-to-text.com 获取您的 JWT Token'
                : `${MODELS.find(m => m.id === showApiConfig)?.provider || 'OpenAI'} 需要 API Key`}
            </p>
            <div className="form-group">
              <label>{configType === 'luyin' ? 'JWT Token' : 'API Key'}</label>
              <input
                type="password"
                value={apiKeyInput}
                onChange={e => setApiKeyInput(e.target.value)}
                placeholder={configType === 'luyin' ? 'eyJ0eXAi...' : 'sk-...'}
              />
            </div>
            {configType === 'luyin' && (
              <p style={{ fontSize: '11px', color: 'var(--text-muted)', marginTop: '-8px', marginBottom: '12px' }}>
                Token 来自您的录音王账户，过期后需要重新获取
              </p>
            )}
            <div className="modal-actions">
              <button className="btn-cancel" onClick={() => setShowApiConfig(null)}>取消</button>
              <button className="btn-confirm" onClick={() => handleApiSave(showApiConfig)}>
                保存{showApiConfig !== 'luyin-free' && showApiConfig !== 'gpt-4o-mini-transcribe' ? '' : '并使用'}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
