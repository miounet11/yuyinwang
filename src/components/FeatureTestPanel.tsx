import React, { useState } from 'react';
import { validator } from '../utils/featureValidator';
import './FeatureTestPanel.css';

interface FeatureTestPanelProps {
  isVisible: boolean;
  onClose: () => void;
}

const FeatureTestPanel: React.FC<FeatureTestPanelProps> = ({ isVisible, onClose }) => {
  const [activeTab, setActiveTab] = useState<'features' | 'interactions' | 'report'>('features');
  const [testResults, setTestResults] = useState<any>(null);

  const runTests = () => {
    const features = validator.validateAll();
    const interactions = validator.testMicroInteractions();
    const report = validator.generateReport();
    
    setTestResults({
      features,
      interactions,
      report
    });
  };

  React.useEffect(() => {
    if (isVisible) {
      runTests();
    }
  }, [isVisible]);

  if (!isVisible || !testResults) return null;

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'complete': return '✅';
      case 'partial': return '⚠️';
      case 'mock': return '🔄';
      case 'missing': return '❌';
      default: return '❓';
    }
  };

  const getPriorityColor = (priority: string) => {
    switch (priority) {
      case 'critical': return '#ff4444';
      case 'high': return '#ff9800';
      case 'medium': return '#ffeb3b';
      case 'low': return '#4caf50';
      default: return '#808080';
    }
  };

  return (
    <div className="test-panel-overlay" onClick={onClose}>
      <div className="test-panel" onClick={(e) => e.stopPropagation()}>
        <div className="test-panel-header">
          <h2>🧪 Recording King 功能测试面板</h2>
          <button className="close-btn" onClick={onClose}>✕</button>
        </div>

        <div className="test-panel-tabs">
          <button 
            className={`test-tab ${activeTab === 'features' ? 'active' : ''}`}
            onClick={() => setActiveTab('features')}
          >
            功能完整性
          </button>
          <button 
            className={`test-tab ${activeTab === 'interactions' ? 'active' : ''}`}
            onClick={() => setActiveTab('interactions')}
          >
            微交互测试
          </button>
          <button 
            className={`test-tab ${activeTab === 'report' ? 'active' : ''}`}
            onClick={() => setActiveTab('report')}
          >
            测试报告
          </button>
        </div>

        <div className="test-panel-content">
          {activeTab === 'features' && (
            <div className="features-list">
              {testResults.features.map((feature: any, index: number) => (
                <div key={index} className="feature-item">
                  <div className="feature-header">
                    <span className="feature-icon">{getStatusIcon(feature.status)}</span>
                    <span className="feature-name">{feature.name}</span>
                    <span 
                      className="feature-priority"
                      style={{ backgroundColor: getPriorityColor(feature.priority) }}
                    >
                      {feature.priority}
                    </span>
                  </div>
                  <div className="feature-details">
                    <p className="feature-description">{feature.description}</p>
                    {feature.issues && feature.issues.length > 0 && (
                      <div className="feature-issues">
                        <strong>问题:</strong>
                        <ul>
                          {feature.issues.map((issue: string, i: number) => (
                            <li key={i}>{issue}</li>
                          ))}
                        </ul>
                      </div>
                    )}
                  </div>
                </div>
              ))}
            </div>
          )}

          {activeTab === 'interactions' && (
            <div className="interactions-list">
              {testResults.interactions.map((item: any, index: number) => (
                <div key={index} className="interaction-item">
                  <div className="interaction-header">
                    <span className="interaction-component">{item.component}</span>
                    <span className="interaction-status">{item.status}</span>
                  </div>
                  {item.improvements && (
                    <div className="interaction-improvements">
                      <strong>优化项:</strong>
                      <ul>
                        {item.improvements.map((improvement: string, i: number) => (
                          <li key={i}>{improvement}</li>
                        ))}
                      </ul>
                    </div>
                  )}
                </div>
              ))}
            </div>
          )}

          {activeTab === 'report' && (
            <div className="test-report">
              <pre>{testResults.report}</pre>
            </div>
          )}
        </div>

        <div className="test-panel-footer">
          <button className="refresh-btn" onClick={runTests}>
            🔄 重新测试
          </button>
          <button className="export-btn" onClick={() => {
            const blob = new Blob([testResults.report], { type: 'text/plain' });
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = 'spokenly-test-report.txt';
            a.click();
          }}>
            📥 导出报告
          </button>
        </div>
      </div>
    </div>
  );
};

export default FeatureTestPanel;