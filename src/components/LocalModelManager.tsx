import React, { useState } from 'react';
import { useModelsStore } from '../stores/modelsStore';
import './LocalModelManager.css';

interface LocalModelManagerProps {
  isVisible: boolean;
  onClose: () => void;
}

const LocalModelManager: React.FC<LocalModelManagerProps> = ({
  isVisible,
  onClose
}) => {
  const {
    models,
    downloadTasks,
    startDownload,
    pauseDownload,
    resumeDownload,
    cancelDownload,
    uninstallModel
  } = useModelsStore();

  const [storagePath] = useState('/Users/lu/Library/Caches/Spokenly/Models');
  const [storageUsed] = useState(2.3); // GB
  const [storageTotal] = useState(10); // GB

  const localModels = models.filter(m => m.type === 'local');
  const installedModels = localModels.filter(m => m.installed);
  const downloadingModels = localModels.filter(m => m.downloading);

  const handleChangePath = async () => {
    // 在实际应用中，这里应该调用Tauri API选择文件夹
    console.log('选择新的存储路径');
  };

  const handleOpenFolder = () => {
    // 在实际应用中，这里应该调用Tauri API打开文件夹
    console.log('打开模型文件夹');
  };

  // const formatSize = (mb: number) => {
  //   if (mb < 1024) return `${mb} MB`;
  //   return `${(mb / 1024).toFixed(1)} GB`;
  // };

  if (!isVisible) return null;

  return (
    <div className="local-manager-overlay" onClick={onClose}>
      <div className="local-manager-dialog" onClick={(e) => e.stopPropagation()}>
        <div className="manager-header">
          <h2>本地模型管理</h2>
          <button className="close-btn" onClick={onClose}>✕</button>
        </div>

        <div className="manager-body">
          {/* 存储路径设置 */}
          <div className="storage-section">
            <h3>模型存储位置</h3>
            <div className="storage-path">
              <input 
                type="text" 
                value={storagePath} 
                readOnly 
                className="path-input"
              />
              <button className="path-btn" onClick={handleChangePath}>
                选择...
              </button>
              <button className="path-btn" onClick={handleOpenFolder}>
                📁 打开
              </button>
            </div>
            <p className="storage-hint">
              选择定义存储Whisper模型、更改位置后，首次使用模型时将重新下载。
            </p>
          </div>

          {/* 存储空间统计 */}
          <div className="storage-stats">
            <h3>存储使用情况</h3>
            <div className="storage-bar">
              <div 
                className="storage-used" 
                style={{ width: `${(storageUsed / storageTotal) * 100}%` }}
              ></div>
            </div>
            <div className="storage-info">
              <span>已使用: {storageUsed} GB / {storageTotal} GB</span>
              <span>{Math.round((storageUsed / storageTotal) * 100)}% 已使用</span>
            </div>
          </div>

          {/* 已安装模型列表 */}
          <div className="models-section">
            <h3>已安装的模型 ({installedModels.length})</h3>
            {installedModels.length === 0 ? (
              <div className="empty-state">
                <p>尚未安装任何本地模型</p>
              </div>
            ) : (
              <div className="installed-models">
                {installedModels.map(model => (
                  <div key={model.id} className="installed-model">
                    <div className="model-icon">{model.icon}</div>
                    <div className="model-details">
                      <h4>{model.name}</h4>
                      <p>{model.modelSize}</p>
                    </div>
                    <button 
                      className="uninstall-btn"
                      onClick={() => uninstallModel(model.id)}
                    >
                      删除
                    </button>
                  </div>
                ))}
              </div>
            )}
          </div>

          {/* 下载中的模型 */}
          {downloadingModels.length > 0 && (
            <div className="models-section">
              <h3>正在下载 ({downloadingModels.length})</h3>
              <div className="downloading-models">
                {downloadingModels.map(model => {
                  const task = downloadTasks.find(t => t.modelId === model.id);
                  return (
                    <div key={model.id} className="downloading-model">
                      <div className="model-info">
                        <div className="model-icon">{model.icon}</div>
                        <div className="model-details">
                          <h4>{model.name}</h4>
                          <div className="download-progress">
                            <div className="progress-bar">
                              <div 
                                className="progress-fill"
                                style={{ width: `${task?.progress || 0}%` }}
                              ></div>
                            </div>
                            <div className="progress-stats">
                              <span>{task?.speed}</span>
                              <span>{Math.round(task?.progress || 0)}%</span>
                              <span>{task?.remaining}</span>
                            </div>
                          </div>
                        </div>
                      </div>
                      <div className="download-controls">
                        {task?.status === 'downloading' ? (
                          <button onClick={() => pauseDownload(model.id)}>⏸</button>
                        ) : (
                          <button onClick={() => resumeDownload(model.id)}>▶</button>
                        )}
                        <button onClick={() => cancelDownload(model.id)}>✕</button>
                      </div>
                    </div>
                  );
                })}
              </div>
            </div>
          )}

          {/* 可下载模型 */}
          <div className="models-section">
            <h3>可用模型</h3>
            <div className="available-models">
              {localModels.filter(m => !m.installed && !m.downloading).map(model => (
                <div key={model.id} className="available-model">
                  <div className="model-icon">{model.icon}</div>
                  <div className="model-details">
                    <h4>{model.name}</h4>
                    <p className="model-desc">{model.description}</p>
                    <div className="model-meta">
                      <span>{model.modelSize}</span>
                      <span>准确度: {'•'.repeat(model.accuracy)}</span>
                      <span>速度: {'•'.repeat(model.speed)}</span>
                    </div>
                  </div>
                  <button 
                    className="download-btn"
                    onClick={() => startDownload(model.id)}
                  >
                    下载
                  </button>
                </div>
              ))}
            </div>
          </div>
        </div>

        <div className="manager-footer">
          <button className="close-btn" onClick={onClose}>关闭</button>
        </div>
      </div>
    </div>
  );
};

export default LocalModelManager;