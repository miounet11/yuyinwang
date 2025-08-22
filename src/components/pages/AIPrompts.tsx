/**
 * AI提示页面
 * 完美复刻 Spokenly 第六张截图的设计
 */

import React, { useState } from 'react';
import './AIPrompts.css';
import { SpokenlyCard, SpokenlyButton } from '../ui';

interface WorkflowStep {
  step: number;
  title: string;
  description: string;
  icon: string;
}

interface AIPromptsProps {}

const AIPrompts: React.FC<AIPromptsProps> = () => {
  const [isEditingPrompt, setIsEditingPrompt] = useState(false);
  const [currentPrompt, setCurrentPrompt] = useState('');

  const workflowSteps: WorkflowStep[] = [
    {
      step: 1,
      title: '激活并口述',
      description: '使用快捷键或在所需应用中说话',
      icon: '🎤'
    },
    {
      step: 2,
      title: 'AI增强',
      description: '您的语音用选定的提示进行处理',
      icon: '🤖'
    },
    {
      step: 3,
      title: '自动输入',
      description: 'AI增强的文本自动输入到应用中',
      icon: '⌨️'
    }
  ];

  const handleEditPrompt = () => {
    setIsEditingPrompt(true);
  };

  const handleSavePrompt = () => {
    setIsEditingPrompt(false);
    // 保存提示逻辑
  };

  const handleCancelEdit = () => {
    setIsEditingPrompt(false);
    setCurrentPrompt('');
  };

  return (
    <div className="spokenly-page">
      <div className="spokenly-page-header">
        <h1>AI提示</h1>
        <p>借助 AI 技术轻松优化您的文本转录质量。</p>
      </div>

      <div className="spokenly-page-content">
        {/* 工作原理流程 */}
        <SpokenlyCard className="workflow-section">
          <h3>工作原理：</h3>
          
          <div className="workflow-steps">
            {workflowSteps.map((step, index) => (
              <div key={step.step} className="workflow-step">
                <div className="step-indicator">
                  <div className="step-number">{step.step}</div>
                  <div className="step-icon">{step.icon}</div>
                </div>
                
                <div className="step-content">
                  <h4 className="step-title">{step.title}</h4>
                  <p className="step-description">{step.description}</p>
                </div>

                {/* 连接线 */}
                {index < workflowSteps.length - 1 && (
                  <div className="step-connector">
                    <div className="connector-line"></div>
                    <div className="connector-arrow">→</div>
                  </div>
                )}
              </div>
            ))}
          </div>
        </SpokenlyCard>

        {/* 编辑主提示按钮 */}
        <div className="prompt-actions">
          <SpokenlyButton
            variant="primary"
            size="lg"
            onClick={handleEditPrompt}
            className="edit-prompt-button"
          >
            <span className="button-icon">✏️</span>
            编辑主提示
          </SpokenlyButton>
          <p className="button-subtitle">Create New Prompt</p>
        </div>

        {/* 提示编辑模态框 */}
        {isEditingPrompt && (
          <SpokenlyCard className="prompt-editor">
            <h3>编辑 AI 提示</h3>
            <div className="editor-content">
              <div className="editor-description">
                <p>自定义 AI 如何处理您的语音转录。您可以添加指令来改善格式、语法、或添加特定的处理规则。</p>
              </div>
              
              <div className="editor-input">
                <label htmlFor="prompt-textarea">主提示内容：</label>
                <textarea
                  id="prompt-textarea"
                  value={currentPrompt}
                  onChange={(e) => setCurrentPrompt(e.target.value)}
                  placeholder="请输入您的 AI 提示指令...&#10;&#10;例如：&#10;- 请纠正语法和拼写错误&#10;- 添加适当的标点符号&#10;- 保持原意的同时使文本更流畅&#10;- 格式化为段落形式"
                  rows={8}
                  className="prompt-textarea"
                />
              </div>

              <div className="editor-actions">
                <SpokenlyButton
                  variant="secondary"
                  size="md"
                  onClick={handleCancelEdit}
                >
                  取消
                </SpokenlyButton>
                
                <SpokenlyButton
                  variant="primary"
                  size="md"
                  onClick={handleSavePrompt}
                >
                  保存提示
                </SpokenlyButton>
              </div>
            </div>
          </SpokenlyCard>
        )}

        {/* 预设提示模板 */}
        <SpokenlyCard className="preset-prompts">
          <h3>预设提示模板</h3>
          <div className="preset-grid">
            <div className="preset-item">
              <h4>📝 通用优化</h4>
              <p>纠正语法、添加标点、改善流畅度</p>
              <SpokenlyButton variant="ghost" size="sm">
                使用模板
              </SpokenlyButton>
            </div>
            
            <div className="preset-item">
              <h4>💼 商务邮件</h4>
              <p>格式化为正式商务邮件格式</p>
              <SpokenlyButton variant="ghost" size="sm">
                使用模板
              </SpokenlyButton>
            </div>
            
            <div className="preset-item">
              <h4>📚 学习笔记</h4>
              <p>整理为结构化的学习笔记格式</p>
              <SpokenlyButton variant="ghost" size="sm">
                使用模板
              </SpokenlyButton>
            </div>
            
            <div className="preset-item">
              <h4>📋 会议记录</h4>
              <p>格式化为清晰的会议纪要</p>
              <SpokenlyButton variant="ghost" size="sm">
                使用模板
              </SpokenlyButton>
            </div>
          </div>
        </SpokenlyCard>

        {/* AI 功能说明 */}
        <SpokenlyCard className="ai-features">
          <h3>AI 增强功能</h3>
          <div className="feature-list">
            <div className="feature-item">
              <div className="feature-icon">🎯</div>
              <div className="feature-content">
                <h4>智能纠错</h4>
                <p>自动识别和纠正语音识别中的常见错误</p>
              </div>
            </div>
            
            <div className="feature-item">
              <div className="feature-icon">📝</div>
              <div className="feature-content">
                <h4>格式优化</h4>
                <p>根据内容类型自动添加合适的格式和结构</p>
              </div>
            </div>
            
            <div className="feature-item">
              <div className="feature-icon">🌐</div>
              <div className="feature-content">
                <h4>多语言支持</h4>
                <p>支持多种语言的智能处理和优化</p>
              </div>
            </div>
            
            <div className="feature-item">
              <div className="feature-icon">⚡</div>
              <div className="feature-content">
                <h4>实时处理</h4>
                <p>边说边处理，无需等待即可获得优化结果</p>
              </div>
            </div>
          </div>
        </SpokenlyCard>
      </div>
    </div>
  );
};

export default AIPrompts;