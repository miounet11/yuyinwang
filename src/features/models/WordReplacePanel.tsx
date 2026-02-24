import React, { useState } from 'react';
import { useAppStore } from '../../shared/stores/useAppStore';
import type { WordReplacement } from '../../shared/types';
import { escapeHtml } from '../../shared/utils';
import './WordReplacePanel.css';

export const WordReplacePanel: React.FC = () => {
  const { wordReplacements, addWordReplacement, updateWordReplacement, deleteWordReplacement } =
    useAppStore();
  const [editingId, setEditingId] = useState<string | null>(null);
  const [fromInput, setFromInput] = useState('');
  const [toInput, setToInput] = useState('');

  const handleAdd = () => {
    if (!fromInput.trim() || !toInput.trim()) return;

    const newReplacement: WordReplacement = {
      id: Date.now().toString(),
      from: escapeHtml(fromInput.trim()),
      to: escapeHtml(toInput.trim()),
      enabled: true,
    };

    addWordReplacement(newReplacement);
    setFromInput('');
    setToInput('');
  };

  const handleEdit = (replacement: WordReplacement) => {
    setEditingId(replacement.id);
    setFromInput(replacement.from);
    setToInput(replacement.to);
  };

  const handleUpdate = () => {
    if (!editingId || !fromInput.trim() || !toInput.trim()) return;

    updateWordReplacement(editingId, {
      from: escapeHtml(fromInput.trim()),
      to: escapeHtml(toInput.trim()),
    });

    setEditingId(null);
    setFromInput('');
    setToInput('');
  };

  const handleCancel = () => {
    setEditingId(null);
    setFromInput('');
    setToInput('');
  };

  const handleToggle = (id: string, enabled: boolean) => {
    updateWordReplacement(id, { enabled });
  };

  const handleDelete = (id: string) => {
    if (editingId === id) {
      handleCancel();
    }
    deleteWordReplacement(id);
  };

  return (
    <div className="word-replace-panel">
      <div className="panel-header">
        <h3 className="panel-title">词替换规则</h3>
        <p className="panel-desc">自动替换转录文本中的特定词语</p>
      </div>

      {/* 添加/编辑表单 */}
      <div className="replace-form">
        <div className="form-row">
          <input
            type="text"
            className="form-input"
            placeholder="原始词"
            value={fromInput}
            onChange={(e) => setFromInput(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') {
                editingId ? handleUpdate() : handleAdd();
              }
            }}
          />
          <span className="arrow">→</span>
          <input
            type="text"
            className="form-input"
            placeholder="替换词"
            value={toInput}
            onChange={(e) => setToInput(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') {
                editingId ? handleUpdate() : handleAdd();
              }
            }}
          />
          {editingId ? (
            <>
              <button className="btn-primary" onClick={handleUpdate}>
                更新
              </button>
              <button className="btn-secondary" onClick={handleCancel}>
                取消
              </button>
            </>
          ) : (
            <button className="btn-primary" onClick={handleAdd}>
              添加
            </button>
          )}
        </div>
      </div>

      {/* 替换规则列表 */}
      {wordReplacements.length > 0 ? (
        <div className="replace-list">
          {wordReplacements.map((replacement) => (
            <div
              key={replacement.id}
              className={`replace-item ${!replacement.enabled ? 'disabled' : ''} ${
                editingId === replacement.id ? 'editing' : ''
              }`}
            >
              <button
                className={`toggle ${replacement.enabled ? 'on' : ''}`}
                onClick={() => handleToggle(replacement.id, !replacement.enabled)}
              />
              <div className="replace-content">
                <span className="replace-from">{replacement.from}</span>
                <span className="replace-arrow">→</span>
                <span className="replace-to">{replacement.to}</span>
              </div>
              <div className="replace-actions">
                <button
                  className="action-btn edit"
                  onClick={() => handleEdit(replacement)}
                  title="编辑"
                >
                  ✏️
                </button>
                <button
                  className="action-btn delete"
                  onClick={() => handleDelete(replacement.id)}
                  title="删除"
                >
                  🗑️
                </button>
              </div>
            </div>
          ))}
        </div>
      ) : (
        <div className="empty-state">
          <p className="empty-text">暂无词替换规则</p>
          <p className="empty-hint">添加规则后，转录文本将自动应用替换</p>
        </div>
      )}
    </div>
  );
};
