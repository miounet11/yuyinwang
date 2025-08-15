import React, { useState } from 'react';
import DiagnosticSteps from './DiagnosticSteps';
import './DiagnosticButton.css';

interface DiagnosticButtonProps {
  category: 'audio' | 'model' | 'api' | 'permission' | 'storage' | 'network' | 'shortcut';
  size?: 'small' | 'medium' | 'large';
  style?: 'button' | 'link' | 'icon';
  autoStart?: boolean;
  className?: string;
  children?: React.ReactNode;
}

const categoryConfig = {
  audio: {
    icon: '🎤',
    label: '音频诊断',
    description: '测试麦克风和音频录制',
    color: '#ff5722'
  },
  model: {
    icon: '🧠',
    label: '模型诊断',
    description: '检查AI模型状态',
    color: '#673ab7'
  },
  api: {
    icon: '🌐',
    label: 'API诊断',
    description: '测试在线服务连接',
    color: '#2196f3'
  },
  permission: {
    icon: '🔒',
    label: '权限诊断',
    description: '检查系统权限',
    color: '#ff9800'
  },
  storage: {
    icon: '💾',
    label: '存储诊断',
    description: '测试数据库和文件',
    color: '#4caf50'
  },
  network: {
    icon: '📡',
    label: '网络诊断',
    description: '检查网络连接',
    color: '#00bcd4'
  },
  shortcut: {
    icon: '⌨️',
    label: '快捷键诊断',
    description: '测试全局快捷键',
    color: '#795548'
  }
};

const DiagnosticButton: React.FC<DiagnosticButtonProps> = ({
  category,
  size = 'medium',
  style = 'button',
  autoStart = false,
  className = '',
  children
}) => {
  const [showDiagnostic, setShowDiagnostic] = useState(false);
  
  const config = categoryConfig[category];
  
  const handleClick = () => {
    setShowDiagnostic(true);
  };

  const renderButton = () => {
    switch (style) {
      case 'link':
        return (
          <button 
            className={`diagnostic-link ${size} ${className}`}
            onClick={handleClick}
            title={config.description}
          >
            <span className="diagnostic-icon">{config.icon}</span>
            <span className="diagnostic-text">
              {children || config.label}
            </span>
          </button>
        );
      
      case 'icon':
        return (
          <button 
            className={`diagnostic-icon-btn ${size} ${className}`}
            onClick={handleClick}
            title={config.description}
            style={{ '--diagnostic-color': config.color } as React.CSSProperties}
          >
            <span className="diagnostic-icon">{config.icon}</span>
          </button>
        );
      
      default: // button
        return (
          <button 
            className={`diagnostic-button ${size} ${className}`}
            onClick={handleClick}
            style={{ '--diagnostic-color': config.color } as React.CSSProperties}
          >
            <span className="diagnostic-icon">{config.icon}</span>
            <span className="diagnostic-text">
              {children || config.label}
            </span>
          </button>
        );
    }
  };

  return (
    <>
      {renderButton()}
      
      <DiagnosticSteps
        isVisible={showDiagnostic}
        onClose={() => setShowDiagnostic(false)}
        category={category}
        autoStart={autoStart}
      />
    </>
  );
};

export default DiagnosticButton;