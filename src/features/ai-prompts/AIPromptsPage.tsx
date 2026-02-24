import React, { useState, useCallback, useMemo, memo } from 'react';
import { useAppStore } from '../../shared/stores/useAppStore';
import { EditPromptModal } from './EditPromptModal';
import type { AIPrompt, PromptAction } from '../../shared/types';
import './AIPromptsPage.css';

interface PromptCardProps {
  prompt: AIPrompt;
  onEdit: (prompt: AIPrompt) => void;
  onDelete: (id: string) => void;
  onToggle: (id: string, enabled: boolean) => void;
  onExecuteAction: (prompt: AIPrompt, action: PromptAction) => void;
  getActionLabel: (action: PromptAction) => string;
  getActionIcon: (action: PromptAction) => string;
}

const PromptCard = memo<PromptCardProps>(({
  prompt,
  onEdit,
  onDelete,
  onToggle,
  onExecuteAction,
  getActionLabel,
  getActionIcon
}) => {
  const handleEdit = useCallback(() => {
    onEdit(prompt);
  }, [prompt, onEdit]);

  const handleDelete = useCallback(() => {
    onDelete(prompt.id);
  }, [prompt.id, onDelete]);

  const handleToggle = useCallback(() => {
    onToggle(prompt.id, !prompt.enabled);
  }, [prompt.id, prompt.enabled, onToggle]);

  return (
    <div className={`prompt-card ${!prompt.enabled ? 'disabled' : ''}`}>
      <div className="prompt-header">
        <div className="prompt-title-row">
          <h3 className="prompt-name">{prompt.name}</h3>
          <button
            className={`toggle ${prompt.enabled ? 'on' : ''}`}
            onClick={handleToggle}
            aria-label={prompt.enabled ? '禁用' : '启用'}
          />
        </div>
        {prompt.shortcut && (
          <div className="prompt-shortcut">
            快捷键: {prompt.shortcut.displayLabel}
          </div>
        )}
      </div>

      <div className="prompt-instruction">
        {prompt.instruction}
      </div>

      {prompt.actions.length > 0 && (
        <div className="prompt-actions">
          <div className="actions-label">动作:</div>
          <div className="actions-grid">
            {prompt.actions.map((action, index) => (
              <button
                key={index}
                className="action-button"
                onClick={() => onExecuteAction(prompt, action)}
                disabled={!prompt.enabled}
                title={getActionLabel(action)}
              >
                <span className="action-icon">{getActionIcon(action)}</span>
                <span className="action-label">{getActionLabel(action)}</span>
              </button>
            ))}
          </div>
        </div>
      )}

      <div className="prompt-footer">
        <button
          className="btn-secondary"
          onClick={handleEdit}
        >
          编辑
        </button>
        <button
          className="btn-danger"
          onClick={handleDelete}
        >
          删除
        </button>
      </div>
    </div>
  );
});
PromptCard.displayName = 'PromptCard';

export const AIPromptsPage: React.FC = () => {
  const { aiPrompts, addAIPrompt, updateAIPrompt, deleteAIPrompt, addToast } = useAppStore();
  const [showEditModal, setShowEditModal] = useState(false);
  const [editingPrompt, setEditingPrompt] = useState<AIPrompt | null>(null);
  const [showConfirmDialog, setShowConfirmDialog] = useState(false);
  const [pendingAction, setPendingAction] = useState<{ prompt: AIPrompt; action: PromptAction } | null>(null);

  const handleAddPrompt = useCallback(() => {
    setEditingPrompt(null);
    setShowEditModal(true);
  }, []);

  const handleEditPrompt = useCallback((prompt: AIPrompt) => {
    setEditingPrompt(prompt);
    setShowEditModal(true);
  }, []);

  const handleDeletePrompt = useCallback(async (id: string) => {
    try {
      await deleteAIPrompt(id);
      addToast('success', 'AI 提示已删除');
    } catch (error) {
      addToast('error', `删除失败: ${error}`);
    }
  }, [deleteAIPrompt, addToast]);

  const handleTogglePrompt = useCallback(async (id: string, enabled: boolean) => {
    try {
      await updateAIPrompt(id, { enabled });
      addToast('success', enabled ? 'AI 提示已启用' : 'AI 提示已禁用');
    } catch (error) {
      addToast('error', `更新失败: ${error}`);
    }
  }, [updateAIPrompt, addToast]);

  const handleSavePrompt = useCallback(async (prompt: AIPrompt) => {
    try {
      if (editingPrompt) {
        await updateAIPrompt(prompt.id, prompt);
        addToast('success', 'AI 提示已更新');
      } else {
        await addAIPrompt(prompt);
        addToast('success', 'AI 提示已添加');
      }
      setShowEditModal(false);
      setEditingPrompt(null);
    } catch (error) {
      addToast('error', `保存失败: ${error}`);
    }
  }, [editingPrompt, updateAIPrompt, addAIPrompt, addToast]);

  const executeAction = useCallback(async (prompt: AIPrompt, action: PromptAction) => {
    try {
      // 这里调用后端 API 执行动作
      // await invoke('execute_prompt_action', { promptId: prompt.id, action });
      addToast('success', `执行动作: ${getActionLabel(action)}`);
    } catch (error) {
      // 显示错误弹窗，提供重试和跳过选项
      addToast('error', `执行失败: ${error}`);
    }
  }, [addToast]);

  const handleExecuteAction = useCallback(async (prompt: AIPrompt, action: PromptAction) => {
    // Shell 命令需要二次确认
    if (action.type === 'shell-command') {
      setPendingAction({ prompt, action });
      setShowConfirmDialog(true);
      return;
    }

    await executeAction(prompt, action);
  }, [executeAction]);

  const handleConfirmShellCommand = useCallback(async () => {
    if (pendingAction) {
      await executeAction(pendingAction.prompt, pendingAction.action);
      setShowConfirmDialog(false);
      setPendingAction(null);
    }
  }, [pendingAction, executeAction]);

  const getActionLabel = useCallback((action: PromptAction): string => {
    switch (action.type) {
      case 'google-search':
        return 'Google 搜索';
      case 'launch-app':
        return '启动应用';
      case 'close-app':
        return '关闭应用';
      case 'ask-chatgpt':
        return 'ChatGPT';
      case 'ask-claude':
        return 'Claude';
      case 'youtube-search':
        return 'YouTube 搜索';
      case 'open-website':
        return '打开网站';
      case 'apple-shortcut':
        return 'Apple 快捷指令';
      case 'shell-command':
        return 'Shell 命令';
      case 'keypress':
        return '按键';
      default:
        return '未知动作';
    }
  }, []);

  const getActionIcon = useCallback((action: PromptAction): string => {
    switch (action.type) {
      case 'google-search':
        return '🔍';
      case 'launch-app':
        return '🚀';
      case 'close-app':
        return '❌';
      case 'ask-chatgpt':
        return '💬';
      case 'ask-claude':
        return '🤖';
      case 'youtube-search':
        return '📺';
      case 'open-website':
        return '🌐';
      case 'apple-shortcut':
        return '⚡';
      case 'shell-command':
        return '⌨️';
      case 'keypress':
        return '⌨️';
      default:
        return '❓';
    }
  }, []);

  return (
    <div className="page">
      <div className="page-header">
        <div>
          <h1 className="page-title">AI 提示</h1>
          <p className="page-desc">配置 AI 提示和自动化动作</p>
        </div>
        <button className="btn-primary" onClick={handleAddPrompt}>
          + 添加提示
        </button>
      </div>

      <div className="section">
        {aiPrompts.length === 0 ? (
          <div className="empty-state">
            <div className="empty-icon">💡</div>
            <h3>还没有 AI 提示</h3>
            <p>创建 AI 提示来自动化常见任务</p>
            <button className="btn-primary" onClick={handleAddPrompt}>
              创建第一个提示
            </button>
          </div>
        ) : (
          <div className="prompts-list">
            {aiPrompts.map((prompt) => (
              <PromptCard
                key={prompt.id}
                prompt={prompt}
                onEdit={handleEditPrompt}
                onDelete={handleDeletePrompt}
                onToggle={handleTogglePrompt}
                onExecuteAction={handleExecuteAction}
                getActionLabel={getActionLabel}
                getActionIcon={getActionIcon}
              />
            ))}
          </div>
        )}
      </div>

      {showEditModal && (
        <EditPromptModal
          isOpen={showEditModal}
          onClose={() => {
            setShowEditModal(false);
            setEditingPrompt(null);
          }}
          onSave={handleSavePrompt}
          initialPrompt={editingPrompt}
        />
      )}

      {showConfirmDialog && pendingAction && (
        <div className="modal-overlay" onClick={() => setShowConfirmDialog(false)}>
          <div className="modal-content confirm-dialog" onClick={(e) => e.stopPropagation()}>
            <h3>确认执行 Shell 命令</h3>
            <p className="confirm-warning">
              ⚠️ 即将执行以下命令，请确认是否继续：
            </p>
            <div className="shell-command-preview">
              {pendingAction.action.type === 'shell-command' && pendingAction.action.command}
            </div>
            <div className="confirm-actions">
              <button
                className="btn-secondary"
                onClick={() => {
                  setShowConfirmDialog(false);
                  setPendingAction(null);
                }}
              >
                取消
              </button>
              <button className="btn-primary" onClick={handleConfirmShellCommand}>
                确认执行
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
