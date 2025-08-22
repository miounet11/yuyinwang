/**
 * GeneralSettings - 常规设置页面
 * 复刻第一张截图的设计：常规首选项配置界面
 */

import React, { useState } from 'react';
import { motion } from 'framer-motion';
import { 
  SpokenlyContent,
  SpokenlyCard,
  SpokenlyCardHeader,
  SpokenlyCardBody,
  SpokenlySwitch,
  SpokenlySelect,
  SpokenlyButton,
  SelectOption
} from '../components/ui';

// 音频设备选项
const audioDeviceOptions: SelectOption[] = [
  { value: 'default', label: 'System Default' },
  { value: 'airpods', label: 'AirPods Pro' },
  { value: 'builtin', label: 'Built-in Microphone' },
  { value: 'usb', label: 'USB Microphone' }
];

// 语言选项
const languageOptions: SelectOption[] = [
  { value: 'zh-CN', label: '简体中文' },
  { value: 'en-US', label: 'English (US)' },
  { value: 'ja-JP', label: '日本語' },
  { value: 'ko-KR', label: '한국어' }
];

// 文本输入方法选项
const textInputOptions: SelectOption[] = [
  { value: 'replace', label: '替换选中的文本' },
  { value: 'insert', label: '插入到光标位置' },
  { value: 'append', label: '追加到末尾' }
];

interface GeneralSettingsProps {
  className?: string;
}

export const GeneralSettings: React.FC<GeneralSettingsProps> = ({
  className = ''
}) => {
  // 设置状态
  const [settings, setSettings] = useState({
    // 行为设置
    launchOnStartup: true,
    showInDock: true,
    showInStatusBar: true,
    appLanguage: 'zh-CN',
    
    // 麦克风设置
    selectedMicrophone: 'default',
    
    // 音频和反馈设置
    playSoundEffects: true,
    muteOnRecording: false,
    enableHapticFeedback: true,
    
    // 文本处理设置
    autoCopyToClipboard: true,
    textInputMethod: 'replace',
    
    // 高级设置
    enableLocalWhisper: false,
    enableQuickCommands: true,
    localOnlyMode: false
  });

  const handleSettingChange = (key: string, value: any) => {
    setSettings(prev => ({
      ...prev,
      [key]: value
    }));
  };

  const pageVariants = {
    initial: { opacity: 0, y: 20 },
    animate: { 
      opacity: 1, 
      y: 0,
      transition: {
        duration: 0.6,
        ease: [0.0, 0.0, 0.2, 1]
      }
    }
  };

  return (
    <SpokenlyContent className={className}>
      <motion.div
        className="general-settings"
        variants={pageVariants}
        initial="initial"
        animate="animate"
        style={{
          width: '100%',
          maxWidth: '800px',
          margin: '0 auto'
        }}
      >
        {/* 页面标题 */}
        <div 
          className="page-header"
          style={{
            marginBottom: 'var(--spokenly-space-8)',
            paddingBottom: 'var(--spokenly-space-4)',
            borderBottom: '1px solid var(--spokenly-border-subtle)'
          }}
        >
          <h1 
            style={{
              fontSize: 'var(--spokenly-text-2xl)',
              fontWeight: 600,
              color: 'var(--spokenly-text-primary)',
              margin: 0,
              marginBottom: 'var(--spokenly-space-2)'
            }}
          >
            常规首选项
          </h1>
          <p 
            style={{
              fontSize: 'var(--spokenly-text-sm)',
              color: 'var(--spokenly-text-secondary)',
              margin: 0
            }}
          >
            根据您的工作流程和偏好配置 Spokenly
          </p>
        </div>

        {/* 设置部分 */}
        <div className="settings-sections" style={{ display: 'flex', flexDirection: 'column', gap: 'var(--spokenly-space-6)' }}>
          
          {/* 行为设置 */}
          <SpokenlyCard>
            <SpokenlyCardHeader 
              title="行为" 
              style={{ paddingBottom: 'var(--spokenly-space-4)' }}
            />
            <SpokenlyCardBody>
              <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--spokenly-space-4)' }}>
                <SpokenlySwitch
                  checked={settings.launchOnStartup}
                  onCheckedChange={(checked) => handleSettingChange('launchOnStartup', checked)}
                  label="登录时启动"
                  description="系统启动时自动运行 Spokenly"
                />
                
                <SpokenlySwitch
                  checked={settings.showInDock}
                  onCheckedChange={(checked) => handleSettingChange('showInDock', checked)}
                  label="在程序坞中显示"
                  description="在 macOS 程序坞中显示应用图标"
                />
                
                <SpokenlySwitch
                  checked={settings.showInStatusBar}
                  onCheckedChange={(checked) => handleSettingChange('showInStatusBar', checked)}
                  label="在状态栏中显示"
                  description="在菜单栏中显示状态图标"
                />
                
                <div style={{ marginTop: 'var(--spokenly-space-2)' }}>
                  <SpokenlySelect
                    label="应用界面语言"
                    value={settings.appLanguage}
                    onValueChange={(value) => handleSettingChange('appLanguage', value)}
                    options={languageOptions}
                    fullWidth
                  />
                </div>
              </div>
            </SpokenlyCardBody>
          </SpokenlyCard>

          {/* 麦克风优先级设置 */}
          <SpokenlyCard>
            <SpokenlyCardHeader 
              title="麦克风优先级设置" 
              style={{ paddingBottom: 'var(--spokenly-space-4)' }}
            />
            <SpokenlyCardBody>
              <SpokenlySelect
                label="首选音频输入设备"
                value={settings.selectedMicrophone}
                onValueChange={(value) => handleSettingChange('selectedMicrophone', value)}
                options={audioDeviceOptions}
                fullWidth
              />
            </SpokenlyCardBody>
          </SpokenlyCard>

          {/* 音频和反馈设置 */}
          <SpokenlyCard>
            <SpokenlyCardHeader 
              title="音频和反馈" 
              style={{ paddingBottom: 'var(--spokenly-space-4)' }}
            />
            <SpokenlyCardBody>
              <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--spokenly-space-4)' }}>
                <SpokenlySwitch
                  checked={settings.playSoundEffects}
                  onCheckedChange={(checked) => handleSettingChange('playSoundEffects', checked)}
                  label="播放声音效果"
                  description="启用录音开始/结束的声音提示"
                />
                
                <SpokenlySwitch
                  checked={settings.muteOnRecording}
                  onCheckedChange={(checked) => handleSettingChange('muteOnRecording', checked)}
                  label="录音时静音"
                  description="录音期间自动静音其他应用程序"
                />
                
                <SpokenlySwitch
                  checked={settings.enableHapticFeedback}
                  onCheckedChange={(checked) => handleSettingChange('enableHapticFeedback', checked)}
                  label="启用触控板反馈"
                  description="在支持的设备上提供触觉反馈"
                />
              </div>
            </SpokenlyCardBody>
          </SpokenlyCard>

          {/* 文本处理设置 */}
          <SpokenlyCard>
            <SpokenlyCardHeader 
              title="文本处理" 
              style={{ paddingBottom: 'var(--spokenly-space-4)' }}
            />
            <SpokenlyCardBody>
              <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--spokenly-space-4)' }}>
                <SpokenlySwitch
                  checked={settings.autoCopyToClipboard}
                  onCheckedChange={(checked) => handleSettingChange('autoCopyToClipboard', checked)}
                  label="自动复制文本到剪贴板"
                  description="转录完成后自动将结果复制到剪贴板"
                />
                
                <div style={{ marginTop: 'var(--spokenly-space-2)' }}>
                  <SpokenlySelect
                    label="文本输入方法"
                    value={settings.textInputMethod}
                    onValueChange={(value) => handleSettingChange('textInputMethod', value)}
                    options={textInputOptions}
                    fullWidth
                  />
                </div>
              </div>
            </SpokenlyCardBody>
          </SpokenlyCard>

          {/* 高级设置 */}
          <SpokenlyCard>
            <SpokenlyCardHeader 
              title="高级" 
              style={{ paddingBottom: 'var(--spokenly-space-4)' }}
            />
            <SpokenlyCardBody>
              <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--spokenly-space-4)' }}>
                <SpokenlySwitch
                  checked={settings.enableLocalWhisper}
                  onCheckedChange={(checked) => handleSettingChange('enableLocalWhisper', checked)}
                  label="本地 Whisper 配置"
                  description="启用本地 Whisper 模型支持"
                />
                
                <SpokenlySwitch
                  checked={settings.enableQuickCommands}
                  onCheckedChange={(checked) => handleSettingChange('enableQuickCommands', checked)}
                  label="快速命令"
                  description="启用语音命令和快捷操作"
                />
                
                <SpokenlySwitch
                  checked={settings.localOnlyMode}
                  onCheckedChange={(checked) => handleSettingChange('localOnlyMode', checked)}
                  label="仅本地模式"
                  description="仅使用本地模型，不连接云端服务"
                />
              </div>
            </SpokenlyCardBody>
          </SpokenlyCard>

        </div>
      </motion.div>
    </SpokenlyContent>
  );
};