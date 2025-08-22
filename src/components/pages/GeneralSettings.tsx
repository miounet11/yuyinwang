/**
 * 常规设置页面
 * 完美复刻 Spokenly 第一张截图的设计
 */

import React, { useState } from 'react';
import './GeneralSettings.css';
import { 
  SpokenlyCard,
  SpokenlySwitch,
  SpokenlySelect,
  SpokenlyButton
} from '../ui';

interface GeneralSettingsProps {}

const GeneralSettings: React.FC<GeneralSettingsProps> = () => {
  const [settings, setSettings] = useState({
    launchAtLogin: false,
    showInDock: false,
    showInStatusBar: true,
    interfaceLanguage: 'System Default',
    selectedMicrophone: 'sanag A30S Pro Max',
    playAudioEffects: true,
    recordingMute: true,
    touchpadFeedback: true,
    autoCopyText: false,
    textInputMethod: '粘贴 (CMD+V)',
    localMode: false
  });

  const microphoneOptions = [
    { value: 'sanag A30S Pro Max', label: 'sanag A30S Pro Max' },
    { value: 'built-in', label: 'Built-in Microphone' },
    { value: 'airpods', label: 'AirPods Pro' }
  ];

  const languageOptions = [
    { value: 'System Default', label: 'System Default' },
    { value: 'English', label: 'English' },
    { value: '中文', label: '中文' }
  ];

  const inputMethodOptions = [
    { value: '粘贴 (CMD+V)', label: '粘贴 (CMD+V)' },
    { value: '键入', label: '键入' },
    { value: '替换', label: '替换' }
  ];

  return (
    <div className="spokenly-page">
      {/* 页面头部 */}
      <div className="spokenly-page-header">
        <h1>常规首选项</h1>
        <p>根据您的工作流程和偏好配置 Spokenly。</p>
      </div>

      <div className="spokenly-page-content">
        {/* 行为设置 */}
        <SpokenlyCard>
          <h3>行为</h3>
          <div className="setting-group">
            <div className="setting-item">
              <div className="setting-label">
                <span>登录时启动</span>
              </div>
              <SpokenlySwitch
                checked={settings.launchAtLogin}
                onChange={(checked) => setSettings(prev => ({ ...prev, launchAtLogin: checked }))}
              />
            </div>

            <div className="setting-item">
              <div className="setting-label">
                <span>在程序坞中显示</span>
              </div>
              <SpokenlySwitch
                checked={settings.showInDock}
                onChange={(checked) => setSettings(prev => ({ ...prev, showInDock: checked }))}
              />
            </div>

            <div className="setting-item">
              <div className="setting-label">
                <span>在状态栏中显示</span>
              </div>
              <SpokenlySwitch
                checked={settings.showInStatusBar}
                onChange={(checked) => setSettings(prev => ({ ...prev, showInStatusBar: checked }))}
              />
            </div>

            <div className="setting-item">
              <div className="setting-label">
                <span>应用界面语言</span>
              </div>
              <SpokenlySelect
                value={settings.interfaceLanguage}
                options={languageOptions}
                onChange={(value) => setSettings(prev => ({ ...prev, interfaceLanguage: value }))}
              />
            </div>
          </div>
        </SpokenlyCard>

        {/* 麦克风先级设置 */}
        <SpokenlyCard>
          <h3>麦克风先级设置</h3>
          <div className="setting-group">
            <div className="microphone-list">
              <div className="microphone-item selected">
                <span className="microphone-number">1.</span>
                <span className="microphone-name">{settings.selectedMicrophone}</span>
                <button className="remove-btn">×</button>
              </div>
            </div>
            <p className="setting-description">
              麦克风按优先级排序使用，拔出可重新排序。
            </p>
          </div>
        </SpokenlyCard>

        {/* 音频和反馈 */}
        <SpokenlyCard>
          <h3>音频和反馈</h3>
          <div className="setting-group">
            <div className="setting-item">
              <div className="setting-label">
                <span>播放声音效果</span>
              </div>
              <SpokenlySwitch
                checked={settings.playAudioEffects}
                onChange={(checked) => setSettings(prev => ({ ...prev, playAudioEffects: checked }))}
              />
            </div>

            <div className="setting-item">
              <div className="setting-label">
                <span>录音时静音</span>
              </div>
              <SpokenlySwitch
                checked={settings.recordingMute}
                onChange={(checked) => setSettings(prev => ({ ...prev, recordingMute: checked }))}
              />
            </div>

            <div className="setting-item">
              <div className="setting-label">
                <span>启用触控板反馈</span>
              </div>
              <SpokenlySwitch
                checked={settings.touchpadFeedback}
                onChange={(checked) => setSettings(prev => ({ ...prev, touchpadFeedback: checked }))}
              />
            </div>
          </div>
        </SpokenlyCard>

        {/* 文本处理 */}
        <SpokenlyCard>
          <h3>文本处理</h3>
          <div className="setting-group">
            <div className="setting-item">
              <div className="setting-label">
                <span>自动复制文本到剪贴板</span>
              </div>
              <SpokenlySwitch
                checked={settings.autoCopyText}
                onChange={(checked) => setSettings(prev => ({ ...prev, autoCopyText: checked }))}
              />
            </div>

            <div className="setting-item">
              <div className="setting-label">
                <span>文本输入方法</span>
              </div>
              <SpokenlySelect
                value={settings.textInputMethod}
                options={inputMethodOptions}
                onChange={(value) => setSettings(prev => ({ ...prev, textInputMethod: value }))}
              />
            </div>
          </div>
        </SpokenlyCard>

        {/* 高级 */}
        <SpokenlyCard>
          <h3>高级</h3>
          <div className="setting-group">
            <div className="setting-item clickable">
              <div className="setting-label">
                <span>本地 Whisper 配置</span>
              </div>
              <span className="chevron">›</span>
            </div>

            <div className="setting-item clickable">
              <div className="setting-label">
                <span>快速命令（旧版 - 即将移除）</span>
              </div>
              <span className="chevron">›</span>
            </div>

            <div className="setting-item">
              <div className="setting-label">
                <span>仅本地模式</span>
              </div>
              <SpokenlySwitch
                checked={settings.localMode}
                onChange={(checked) => setSettings(prev => ({ ...prev, localMode: checked }))}
              />
            </div>
          </div>
        </SpokenlyCard>
      </div>
    </div>
  );
};

export default GeneralSettings;