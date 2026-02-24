import React, { useEffect, useState, useCallback, useMemo, memo } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import { useAppStore } from '../../shared/stores/useAppStore';
import type { ModelCardData, ModelFilter } from '../../shared/types';
import { filterAndSortModels } from '../../shared/utils';
import { WordReplacePanel } from './WordReplacePanel';
import './ModelSettings.css';

const VIRTUAL_SCROLL_THRESHOLD = 20; // Enable virtual scrolling when models > 20

const MODELS: ModelCardData[] = [
  {
    id: 'luyin-free',
    name: 'LuYinWang Transcribe',
    icon: 'LW',
    provider: '录音王',
    description: '由录音王驱动 - 卓越准确性和快速处理。需配置 Token。',
    accuracy: 5,
    speed: 4,
    languages: '多语言',
    type: 'online',
    tags: ['all', 'online', 'accurate', 'fast', 'punctuation'],
    isRealtime: false,
    isMultilingual: true,
    free: true,
    badge: '推荐',
    available: true,
  },
  {
    id: 'gpt-4o-mini-transcribe',
    name: 'GPT-4o mini',
    icon: '🌀',
    provider: 'OpenAI',
    description: '卓越准确性和快速处理。',
    accuracy: 5,
    speed: 4,
    languages: '多语言',
    type: 'api',
    tags: ['all', 'api', 'accurate', 'fast', 'punctuation'],
    isRealtime: false,
    isMultilingual: true,
    badge: '最准确',
    available: true,
  },
  {
    id: 'deepgram-nova3',
    name: 'Nova-3 (English)',
    icon: 'D',
    provider: 'Deepgram',
    description: '实时听写，纯英语优化。',
    accuracy: 4,
    speed: 5,
    languages: '仅英语',
    type: 'api',
    tags: ['all', 'api', 'fast', 'subtitle'],
    isRealtime: true,
    isMultilingual: false,
    badge: '最快',
    available: false,
    unavailableReason: '接口开发中',
  },
  {
    id: 'whisper-tiny',
    name: 'Whisper Tiny',
    icon: '🎯',
    provider: 'OpenAI Whisper',
    description: '最小最快，适合快速草稿。',
    accuracy: 2,
    speed: 5,
    languages: '多语言',
    type: 'local',
    tags: ['all', 'local', 'fast'],
    isRealtime: false,
    isMultilingual: true,
    size: '75 MB',
    available: true,
  },
  {
    id: 'whisper-base',
    name: 'Whisper Base',
    icon: '📝',
    provider: 'OpenAI Whisper',
    description: '基础模型，速度与准确度平衡。',
    accuracy: 3,
    speed: 4,
    languages: '多语言',
    type: 'local',
    tags: ['all', 'local', 'fast'],
    isRealtime: false,
    isMultilingual: true,
    size: '148 MB',
    available: true,
  },
  {
    id: 'whisper-small',
    name: 'Whisper Small',
    icon: '🎤',
    provider: 'OpenAI Whisper',
    description: '准确性和速度平衡，推荐本地模型。',
    accuracy: 4,
    speed: 3,
    languages: '多语言',
    type: 'local',
    tags: ['all', 'local', 'accurate', 'punctuation'],
    isRealtime: false,
    isMultilingual: true,
    size: '488 MB',
    badge: '本地推荐',
    available: true,
  },
  {
    id: 'whisper-medium',
    name: 'Whisper Medium',
    icon: '🔊',
    provider: 'OpenAI Whisper',
    description: '高准确度，适合专业场景。',
    accuracy: 4,
    speed: 2,
    languages: '多语言',
    type: 'local',
    tags: ['all', 'local', 'accurate', 'punctuation'],
    isRealtime: false,
    isMultilingual: true,
    size: '1.5 GB',
    available: true,
  },
  {
    id: 'whisper-large-v3',
    name: 'Whisper Large v3',
    icon: '🏆',
    provider: 'OpenAI Whisper',
    description: '最高准确度，99种语言支持。',
    accuracy: 5,
    speed: 1,
    languages: '99种语言',
    type: 'local',
    tags: ['all', 'local', 'accurate', 'punctuation', 'subtitle'],
    isRealtime: false,
    isMultilingual: true,
    size: '3.1 GB',
    available: true,
  },
  {
    id: 'whisper-large-v3-turbo',
    name: 'Large v3 Turbo',
    icon: '🚀',
    provider: 'OpenAI Whisper',
    description: '优化版，速度提升2倍，准确度接近 Large。',
    accuracy: 5,
    speed: 3,
    languages: '99种语言',
    type: 'local',
    tags: ['all', 'local', 'accurate', 'fast', 'punctuation', 'subtitle'],
    isRealtime: false,
    isMultilingual: true,
    size: '1.6 GB',
    badge: '本地最佳',
    available: true,
  },
];

const DotBar: React.FC<{ value: number; max?: number }> = memo(({ value, max = 5 }) => (
  <span className="dot-bar">
    {Array.from({ length: max }, (_, i) => (
      <span key={i} className={`dot ${i < value ? 'filled' : ''}`} />
    ))}
  </span>
));
DotBar.displayName = 'DotBar';

interface DownloadState {
  [modelId: string]: { progress: number; downloading: boolean };
}

interface ModelCardProps {
  model: ModelCardData;
  isSelected: boolean;
  isDownloaded: boolean;
  downloadState?: { progress: number; downloading: boolean };
  onSelect: (model: ModelCardData) => void;
  onDownload: (modelId: string, e: React.MouseEvent) => void;
  onDelete: (modelId: string, e: React.MouseEvent) => void;
  onUse: (modelId: string) => void;
  onOpenConfig: (modelId: string) => void;
}

const ModelCard = memo<ModelCardProps>(({
  model,
  isSelected,
  isDownloaded,
  downloadState,
  onSelect,
  onDownload,
  onDelete,
  onUse,
  onOpenConfig
}) => {
  const handleCardClick = useCallback(() => {
    onSelect(model);
  }, [model, onSelect]);

  const handleDownloadClick = useCallback((e: React.MouseEvent) => {
    onDownload(model.id, e);
  }, [model.id, onDownload]);

  const handleDeleteClick = useCallback((e: React.MouseEvent) => {
    onDelete(model.id, e);
  }, [model.id, onDelete]);

  const handleUseClick = useCallback((e: React.MouseEvent) => {
    e.stopPropagation();
    onUse(model.id);
  }, [model.id, onUse]);

  const handleConfigClick = useCallback((e: React.MouseEvent) => {
    e.stopPropagation();
    onOpenConfig(model.id);
  }, [model.id, onOpenConfig]);

  return (
    <div
      className={`model-card ${isSelected ? 'selected' : ''} ${
        !model.available ? 'disabled' : ''
      }`}
      onClick={handleCardClick}
    >
      <div className="model-header">
        <span className="model-icon">{model.icon}</span>
        <div className="model-title">
          <span className="model-name">{model.name}</span>
          {model.badge && <span className="tag badge-tag">{model.badge}</span>}
          {model.free && <span className="tag free">免费</span>}
          {model.type === 'local' && isDownloaded && (
            <span className="tag downloaded">已下载</span>
          )}
          {model.isRealtime && <span className="tag realtime">实时</span>}
          {model.isMultilingual && <span className="tag multilingual">多语言</span>}
        </div>
      </div>
      <p className="model-provider">
        {model.provider}
        {model.size ? ` · ${model.size}` : ''}
      </p>
      <p className="model-desc">{model.description}</p>
      <div className="model-stats">
        <div className="stat-row">
          <span className="stat-label">准确度</span>
          <DotBar value={model.accuracy} />
        </div>
        <div className="stat-row">
          <span className="stat-label">速度</span>
          <DotBar value={model.speed} />
        </div>
        <div className="stat-row">
          <span className="stat-label">语言</span>
          <span className="stat-value">{model.languages}</span>
        </div>
      </div>

      {downloadState?.downloading && (
        <div className="download-progress">
          <div className="download-bar" style={{ width: `${downloadState.progress * 100}%` }} />
        </div>
      )}

      <div className="model-card-actions">
        {model.type === 'local' && !downloadState?.downloading && !isDownloaded && (
          <button
            className="model-action download"
            onClick={handleDownloadClick}
          >
            ⬇ 下载
          </button>
        )}
        {model.type === 'local' && isDownloaded && !isSelected && (
          <button
            className="model-action use"
            onClick={handleUseClick}
          >
            使用
          </button>
        )}
        {model.type === 'local' && isDownloaded && (
          <button
            className="model-action delete"
            onClick={handleDeleteClick}
          >
            删除
          </button>
        )}
        {model.type !== 'local' && !isSelected && model.available && (
          <button className="model-action use">使用</button>
        )}
        {isSelected && <div className="model-action active">✓ 使用中</div>}
        {isSelected && (
          <button className="model-action settings-btn" onClick={handleConfigClick}>
            ⚙ Settings
          </button>
        )}
        {!model.available && (
          <div className="model-action unavailable">
            {model.unavailableReason || '不可用'}
          </div>
        )}
        {model.type === 'local' && downloadState?.downloading && (
          <div className="model-action downloading">
            下载中 {Math.round(downloadState.progress * 100)}%
          </div>
        )}
      </div>
    </div>
  );
});
ModelCard.displayName = 'ModelCard';

export const ModelSettings: React.FC = () => {
  const { settings, setSettings, addToast, wordReplacements } = useAppStore();
  const [activeFilters, setActiveFilters] = useState<ModelFilter[]>(['all']);
  const [showApiConfig, setShowApiConfig] = useState<string | null>(null);
  const [apiKeyInput, setApiKeyInput] = useState('');
  const [configType, setConfigType] = useState<'openai' | 'luyin'>('openai');
  const [downloadedModels, setDownloadedModels] = useState<Set<string>>(new Set());
  const [downloads, setDownloads] = useState<DownloadState>({});
  const [showWordReplace, setShowWordReplace] = useState(false);
  const [showModelConfig, setShowModelConfig] = useState<string | null>(null);
  const [modelLanguage, setModelLanguage] = useState(settings.transcription_language || 'auto');
  const [modelPrompt, setModelPrompt] = useState(settings.transcription_prompt || '');

  useEffect(() => {
    invoke('get_settings').then((s: any) => setSettings(s)).catch(console.error);
    refreshModelStatus();
  }, []);

  const refreshModelStatus = useCallback(() => {
    invoke<Array<{ model_id: string; downloaded: boolean }>>('get_local_model_status')
      .then((statuses) => {
        const downloaded = new Set<string>();
        statuses.forEach((s) => {
          if (s.downloaded) downloaded.add(s.model_id);
        });
        setDownloadedModels(downloaded);
      })
      .catch(console.error);
  }, []);

  useEffect(() => {
    const unlisten = listen<{ model_id: string; progress: number }>(
      'model-download-progress',
      (event) => {
        const { model_id, progress } = event.payload;
        setDownloads((prev) => ({
          ...prev,
          [model_id]: { progress, downloading: progress < 1 },
        }));
        if (progress >= 1) {
          setDownloadedModels((prev) => new Set([...prev, model_id]));
          addToast(
            'success',
            `${MODELS.find((m) => m.id === model_id)?.name || model_id} 下载完成`
          );
        }
      }
    );
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const handleFilterToggle = useCallback((filter: ModelFilter) => {
    if (filter === 'all') {
      setActiveFilters(['all']);
    } else {
      setActiveFilters((prev) => {
        const newFilters = prev.includes(filter)
          ? prev.filter((f) => f !== filter)
          : [...prev.filter((f) => f !== 'all'), filter];
        return newFilters.length === 0 ? ['all'] : newFilters;
      });
    }
  }, []);

  const handleDownloadModel = useCallback(async (modelId: string, e: React.MouseEvent) => {
    e.stopPropagation();
    setDownloads((prev) => ({ ...prev, [modelId]: { progress: 0, downloading: true } }));
    try {
      await invoke('download_local_model', { modelId });
    } catch (err: any) {
      addToast('error', `下载失败: ${err}`);
      setDownloads((prev) => ({ ...prev, [modelId]: { progress: 0, downloading: false } }));
    }
  }, [addToast]);

  const handleDeleteModel = useCallback(async (modelId: string, e: React.MouseEvent) => {
    e.stopPropagation();
    try {
      await invoke('delete_local_model', { modelId });
      setDownloadedModels((prev) => {
        const next = new Set(prev);
        next.delete(modelId);
        return next;
      });
      if (settings.selected_model === modelId) {
        applyModel('luyin-free');
      }
      addToast('success', '模型已删除');
    } catch (err: any) {
      addToast('error', `删除失败: ${err}`);
    }
  }, [settings.selected_model, addToast]);

  const handleSelectModel = useCallback((model: ModelCardData) => {
    if (!model.available) {
      addToast('warning', model.unavailableReason || '该模型暂不可用');
      return;
    }
    if (model.type === 'local' && !downloadedModels.has(model.id)) {
      addToast('warning', '请先下载模型');
      return;
    }
    // Check for API key requirements
    if (model.id === 'luyin-free' && !settings.luyin_token) {
      setConfigType('luyin');
      setShowApiConfig(model.id);
      setApiKeyInput(settings.luyin_token || '');
      return;
    }
    if (model.type === 'api' && !settings.openai_api_key) {
      setConfigType('openai');
      setShowApiConfig(model.id);
      setApiKeyInput(settings.openai_api_key || '');
      return;
    }
    applyModel(model.id);
  }, [downloadedModels, settings.luyin_token, settings.openai_api_key, addToast]);

  const applyModel = useCallback(async (id: string) => {
    const updated = { ...settings, selected_model: id };
    setSettings(updated);
    try {
      await invoke('update_settings', { settings: updated });
      addToast('success', `已切换到 ${MODELS.find((m) => m.id === id)?.name || id}`);
    } catch (e) {
      addToast('error', '保存失败');
    }
  }, [settings, setSettings, addToast]);

  const handleApiSave = useCallback(async (modelId: string) => {
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
  }, [apiKeyInput, configType, settings, setSettings, addToast]);

  const handleOpenModelConfig = useCallback((modelId: string) => {
    setModelLanguage(settings.transcription_language || 'auto');
    setModelPrompt(settings.transcription_prompt || '');
    setShowModelConfig(modelId);
  }, [settings.transcription_language, settings.transcription_prompt]);

  const handleSaveModelConfig = useCallback(async () => {
    const updated = {
      ...settings,
      transcription_language: modelLanguage,
      transcription_prompt: modelPrompt,
    };
    setSettings(updated);
    try {
      await invoke('update_settings', { settings: updated });
      addToast('success', '模型设置已保存');
    } catch (e) {
      addToast('error', '保存失败');
    }
    setShowModelConfig(null);
  }, [modelLanguage, modelPrompt, settings, setSettings, addToast]);

  const filters: { key: ModelFilter; label: string }[] = useMemo(() => [
    { key: 'all', label: '全部' },
    { key: 'online', label: '在线' },
    { key: 'local', label: '本地' },
    { key: 'api', label: 'API' },
    { key: 'fast', label: '快速' },
    { key: 'accurate', label: '准确' },
    { key: 'punctuation', label: '标点符号' },
    { key: 'subtitle', label: '字幕' },
  ], []);

  const filteredModels = useMemo(
    () => filterAndSortModels(MODELS, activeFilters, settings.selected_model),
    [activeFilters, settings.selected_model]
  );

  const hasLuyinToken = useMemo(() => !!settings.luyin_token, [settings.luyin_token]);
  const hasOpenaiKey = useMemo(() => !!settings.openai_api_key, [settings.openai_api_key]);

  // Enable virtual scrolling for large model lists
  const shouldUseVirtualScroll = useMemo(
    () => filteredModels.length > VIRTUAL_SCROLL_THRESHOLD,
    [filteredModels.length]
  );

  return (
    <div className="page">
      <h1 className="page-title">听写模型</h1>
      <p className="page-desc">选择最适合您需求的准确性、隐私性和速度的平衡点</p>

      {/* Token 状态概览 */}
      <div className="section">
        <div style={{ display: 'flex', gap: '10px', marginBottom: '20px', flexWrap: 'wrap' }}>
          <div
            className={`token-badge ${hasLuyinToken ? 'ok' : 'missing'}`}
            onClick={() => {
              setConfigType('luyin');
              setShowApiConfig('luyin-free');
              setApiKeyInput(settings.luyin_token || '');
            }}
          >
            <span className="token-dot" />
            录音王 Token {hasLuyinToken ? '✓' : '未配置'}
          </div>
          <div
            className={`token-badge ${hasOpenaiKey ? 'ok' : 'missing'}`}
            onClick={() => {
              setConfigType('openai');
              setShowApiConfig('gpt-4o-mini-transcribe');
              setApiKeyInput(settings.openai_api_key || '');
            }}
          >
            <span className="token-dot" />
            OpenAI Key {hasOpenaiKey ? '✓' : '未配置'}
          </div>
          <button
            className="token-badge word-replace-btn"
            onClick={() => setShowWordReplace(!showWordReplace)}
          >
            📝 词替换 ({wordReplacements.filter((r) => r.enabled).length})
          </button>
        </div>
      </div>

      {/* 词替换面板 */}
      {showWordReplace && (
        <div className="section">
          <WordReplacePanel />
        </div>
      )}

      {/* 筛选标签栏 */}
      <div className="section">
        <div className="filter-tabs">
          {filters.map((f) => (
            <button
              key={f.key}
              className={`filter-tab ${activeFilters.includes(f.key) ? 'active' : ''}`}
              onClick={() => handleFilterToggle(f.key)}
            >
              {f.label}
            </button>
          ))}
        </div>

        {/* 模型卡片网格 */}
        <div className="model-grid" style={shouldUseVirtualScroll ? { maxHeight: '70vh', overflowY: 'auto' } : undefined}>
          {filteredModels.map((model) => (
            <ModelCard
              key={model.id}
              model={model}
              isSelected={settings.selected_model === model.id}
              isDownloaded={downloadedModels.has(model.id)}
              downloadState={downloads[model.id]}
              onSelect={handleSelectModel}
              onDownload={handleDownloadModel}
              onDelete={handleDeleteModel}
              onUse={applyModel}
              onOpenConfig={handleOpenModelConfig}
            />
          ))}
        </div>
      </div>

      {/* API Key 配置弹窗 */}
      {showApiConfig && (
        <div className="modal-overlay" onClick={() => setShowApiConfig(null)}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            <h3>
              {configType === 'luyin' ? '🔑 配置录音王 Token' : '🔑 配置 OpenAI API Key'}
            </h3>
            <p className="modal-desc">
              {configType === 'luyin'
                ? '从 record-to-text.com 获取您的 JWT Token'
                : `${MODELS.find((m) => m.id === showApiConfig)?.provider || 'OpenAI'} 需要 API Key`}
            </p>
            <div className="form-group">
              <label>{configType === 'luyin' ? 'JWT Token' : 'API Key'}</label>
              <input
                type="password"
                value={apiKeyInput}
                onChange={(e) => setApiKeyInput(e.target.value)}
                placeholder={configType === 'luyin' ? 'eyJ0eXAi...' : 'sk-...'}
              />
            </div>
            {configType === 'luyin' && (
              <p
                style={{
                  fontSize: '11px',
                  color: 'var(--text-muted)',
                  marginTop: '-8px',
                  marginBottom: '12px',
                }}
              >
                Token 来自您的录音王账户，过期后需要重新获取
              </p>
            )}
            <div className="modal-actions">
              <button className="btn-cancel" onClick={() => setShowApiConfig(null)}>
                取消
              </button>
              <button className="btn-confirm" onClick={() => handleApiSave(showApiConfig)}>
                保存并使用
              </button>
            </div>
          </div>
        </div>
      )}

      {/* 模型设置弹窗 - 语言 & 提示词典 */}
      {showModelConfig && (
        <div className="modal-overlay" onClick={() => setShowModelConfig(null)}>
          <div className="modal-content model-config-modal" onClick={(e) => e.stopPropagation()}>
            <div className="model-config-header">
              <h3>{MODELS.find((m) => m.id === showModelConfig)?.name || ''} 设置</h3>
              <button className="model-config-close" onClick={() => setShowModelConfig(null)}>✕</button>
            </div>

            <div className="model-config-body">
              {/* 语言 */}
              <div className="config-section">
                <div className="config-label-row">
                  <label className="config-label">语言</label>
                  <span className="config-info" title="将模型固定为主要语言，或保留自动检测以处理混合输入。">ⓘ</span>
                </div>
                <p className="config-desc">将 {MODELS.find((m) => m.id === showModelConfig)?.name} 固定为主要语言，或保留自动检测以处理混合输入。</p>
                <select
                  className="config-select"
                  value={modelLanguage}
                  onChange={(e) => setModelLanguage(e.target.value)}
                >
                  <option value="auto">All Languages</option>
                  <option value="zh">中文</option>
                  <option value="en">English</option>
                  <option value="ja">日本語</option>
                  <option value="ko">한국어</option>
                  <option value="fr">Français</option>
                  <option value="de">Deutsch</option>
                  <option value="es">Español</option>
                  <option value="pt">Português</option>
                  <option value="ru">Русский</option>
                  <option value="ar">العربية</option>
                </select>
              </div>

              {/* 提示（词典） */}
              <div className="config-section">
                <div className="config-label-row">
                  <label className="config-label">提示（词典）</label>
                  <span className="config-info" title="为模型提供额外上下文，以提升识别和格式化效果。">ⓘ</span>
                </div>
                <p className="config-desc">为 {MODELS.find((m) => m.id === showModelConfig)?.name} 提供额外上下文，以提升识别和格式化效果。</p>
                <textarea
                  className="config-textarea"
                  placeholder="示例：会议讨论 GPT-4.5 更新，因此请清晰地转写任何提及。"
                  value={modelPrompt}
                  onChange={(e) => setModelPrompt(e.target.value)}
                />
              </div>
            </div>

            <div className="modal-actions">
              <button className="btn-cancel" onClick={() => setShowModelConfig(null)}>取消</button>
              <button className="btn-confirm" onClick={handleSaveModelConfig}>保存</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
