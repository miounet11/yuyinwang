import React, { useState } from 'react';
import './HistorySettings.css';

interface HistorySettingsProps {
  isVisible: boolean;
  onClose: () => void;
  settings: {
    autoDelete: boolean;
    deleteAfterDays: number;
    maxStorageSize: number;
    groupByDate: boolean;
    showSummaries: boolean;
    exportFormat: 'txt' | 'json' | 'csv';
  };
  onUpdateSettings: (settings: any) => void;
}

const HistorySettings: React.FC<HistorySettingsProps> = ({
  isVisible,
  onClose,
  settings,
  onUpdateSettings
}) => {
  const [localSettings, setLocalSettings] = useState(settings);

  const handleSave = () => {
    onUpdateSettings(localSettings);
    onClose();
  };

  const handleReset = () => {
    const defaultSettings = {
      autoDelete: false,
      deleteAfterDays: 30,
      maxStorageSize: 1000,
      groupByDate: true,
      showSummaries: true,
      exportFormat: 'txt' as const
    };
    setLocalSettings(defaultSettings);
  };

  if (!isVisible) return null;

  return (
    <div className="history-settings-overlay" onClick={onClose}>
      <div 
        className="history-settings-dialog" 
        onClick={(e) => e.stopPropagation()}
      >
        <div className="history-settings-header">
          <h2>历史记录设置</h2>
          <button className="close-btn" onClick={onClose}>✕</button>
        </div>

        <div className="settings-section">
          <h3>自动管理</h3>
          
          <div className="setting-item">
            <label className="setting-label">
              <input
                type="checkbox"
                checked={localSettings.autoDelete}
                onChange={(e) => setLocalSettings({
                  ...localSettings,
                  autoDelete: e.target.checked
                })}
              />
              <span>自动删除旧记录</span>
            </label>
            <p className="setting-description">
              自动删除超过指定天数的历史记录
            </p>
          </div>

          {localSettings.autoDelete && (
            <div className="setting-item indented">
              <label className="setting-label">
                <span>保留天数：</span>
                <input
                  type="number"
                  value={localSettings.deleteAfterDays}
                  onChange={(e) => setLocalSettings({
                    ...localSettings,
                    deleteAfterDays: parseInt(e.target.value)
                  })}
                  min="1"
                  max="365"
                  className="number-input"
                />
                <span className="unit">天</span>
              </label>
            </div>
          )}

          <div className="setting-item">
            <label className="setting-label">
              <span>最大存储空间：</span>
              <input
                type="number"
                value={localSettings.maxStorageSize}
                onChange={(e) => setLocalSettings({
                  ...localSettings,
                  maxStorageSize: parseInt(e.target.value)
                })}
                min="100"
                max="10000"
                step="100"
                className="number-input"
              />
              <span className="unit">MB</span>
            </label>
            <p className="setting-description">
              当存储空间达到限制时，自动删除最旧的记录
            </p>
          </div>
        </div>

        <div className="settings-section">
          <h3>显示选项</h3>
          
          <div className="setting-item">
            <label className="setting-label">
              <input
                type="checkbox"
                checked={localSettings.groupByDate}
                onChange={(e) => setLocalSettings({
                  ...localSettings,
                  groupByDate: e.target.checked
                })}
              />
              <span>按日期分组</span>
            </label>
            <p className="setting-description">
              将历史记录按日期分组显示
            </p>
          </div>

          <div className="setting-item">
            <label className="setting-label">
              <input
                type="checkbox"
                checked={localSettings.showSummaries}
                onChange={(e) => setLocalSettings({
                  ...localSettings,
                  showSummaries: e.target.checked
                })}
              />
              <span>显示摘要</span>
            </label>
            <p className="setting-description">
              在列表中显示转录内容的简短摘要
            </p>
          </div>
        </div>

        <div className="settings-section">
          <h3>导出设置</h3>
          
          <div className="setting-item">
            <label className="setting-label">
              <span>默认导出格式：</span>
              <select
                value={localSettings.exportFormat}
                onChange={(e) => setLocalSettings({
                  ...localSettings,
                  exportFormat: e.target.value as 'txt' | 'json' | 'csv'
                })}
                className="format-select"
              >
                <option value="txt">纯文本 (.txt)</option>
                <option value="json">JSON (.json)</option>
                <option value="csv">CSV (.csv)</option>
              </select>
            </label>
            <p className="setting-description">
              选择导出历史记录时的默认文件格式
            </p>
          </div>
        </div>

        <div className="settings-section">
          <h3>隐私</h3>
          
          <div className="privacy-actions">
            <button className="danger-btn" onClick={() => {
              if (window.confirm('确定要清除所有历史记录吗？此操作不可恢复。')) {
                // 清除历史记录的逻辑
                console.log('Clearing all history...');
              }
            }}>
              清除所有历史记录
            </button>
            
            <button className="export-btn" onClick={() => {
              // 导出历史记录的逻辑
              console.log('Exporting history...');
            }}>
              导出历史记录
            </button>
          </div>
        </div>

        <div className="history-settings-footer">
          <button className="reset-btn" onClick={handleReset}>
            恢复默认
          </button>
          <div className="action-buttons">
            <button className="cancel-btn" onClick={onClose}>
              取消
            </button>
            <button className="save-btn" onClick={handleSave}>
              保存
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};

export default HistorySettings;