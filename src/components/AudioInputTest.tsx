import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import './AudioInputTest.css';

interface AudioDevice {
  id: string;
  name: string;
  is_default: boolean;
  is_available: boolean;
}

interface AudioInputTestProps {
  isVisible: boolean;
  onClose: () => void;
  audioDevices?: AudioDevice[];
}

const AudioInputTest: React.FC<AudioInputTestProps> = ({ isVisible, onClose, audioDevices = [] }) => {
  const [devices, setDevices] = useState<AudioDevice[]>(audioDevices);
  const [selectedDevice, setSelectedDevice] = useState<string>('');
  const [testDuration, setTestDuration] = useState<number>(3);
  const [isTestingAudio, setIsTestingAudio] = useState<boolean>(false);
  const [testResult, setTestResult] = useState<string>('');
  const [lastTestData, setLastTestData] = useState<{
    maxVolume: number;
    rmsVolume: number;
    sampleCount: number;
    duration: number;
  } | null>(null);

  // 更新设备列表
  useEffect(() => {
    if (audioDevices && audioDevices.length > 0) {
      setDevices(audioDevices);
      // 自动选择默认设备
      const defaultDevice = audioDevices.find(device => device.is_default);
      if (defaultDevice && !selectedDevice) {
        setSelectedDevice(defaultDevice.id);
      }
    }
  }, [audioDevices]);

  // 加载音频设备列表（仅在没有传入设备时使用）
  const loadAudioDevices = async () => {
    if (!audioDevices || audioDevices.length === 0) {
      try {
        const devices = await invoke<AudioDevice[]>('get_audio_devices');
        setDevices(devices);
        
        // 自动选择默认设备
        const defaultDevice = devices.find(device => device.is_default);
        if (defaultDevice) {
          setSelectedDevice(defaultDevice.id);
        }
      } catch (error) {
        console.error('加载音频设备失败:', error);
        setTestResult(`❌ 加载音频设备失败: ${error}`);
      }
    }
  };

  // 测试音频输入
  const testAudioInput = async () => {
    if (isTestingAudio) return;
    
    setIsTestingAudio(true);
    setTestResult('🧪 开始测试音频输入...');
    setLastTestData(null);
    
    try {
      const result = await invoke<string>('test_audio_input', {
        deviceId: selectedDevice || null,
        durationSeconds: testDuration
      });
      
      setTestResult(result);
      
      // 解析测试结果数据
      const lines = result.split('\n');
      const dataLine = lines.find(line => line.includes('样本数:'));
      if (dataLine) {
        const match = dataLine.match(/样本数: (\d+), 最大音量: ([\d.]+), RMS音量: ([\d.]+)/);
        if (match) {
          setLastTestData({
            sampleCount: parseInt(match[1]),
            maxVolume: parseFloat(match[2]),
            rmsVolume: parseFloat(match[3]),
            duration: parseInt(match[1]) / 48000
          });
        }
      }
    } catch (error) {
      console.error('音频测试失败:', error);
      setTestResult(`❌ 音频测试失败: ${error}`);
    } finally {
      setIsTestingAudio(false);
    }
  };

  const getVolumeBarWidth = (volume: number, maxExpected: number = 1.0) => {
    return Math.min((volume / maxExpected) * 100, 100);
  };

  const getVolumeColor = (volume: number) => {
    if (volume < 0.01) return '#ff4444'; // 红色 - 音量过低
    if (volume < 0.05) return '#ff9800'; // 橙色 - 音量较低
    if (volume < 0.3) return '#4caf50';  // 绿色 - 正常
    return '#2196f3'; // 蓝色 - 音量较高
  };

  const getDiagnosisText = () => {
    if (!lastTestData) return '';
    
    const { maxVolume, rmsVolume } = lastTestData;
    
    if (maxVolume < 0.01) {
      return '🔧 建议检查: 1) 麦克风权限 2) 系统音量设置 3) 麦克风连接';
    } else if (rmsVolume < 0.005) {
      return '🎤 建议: 1) 提高麦克风音量 2) 靠近麦克风说话 3) 检查麦克风指向性';
    } else if (maxVolume > 0.8) {
      return '⚠️ 音量较高，可能导致失真，建议适当降低麦克风音量';
    } else {
      return '✅ 音频输入正常，可以进行录音';
    }
  };

  useEffect(() => {
    if (isVisible) {
      loadAudioDevices();
    }
  }, [isVisible]);

  if (!isVisible) return null;

  return (
    <div className="audio-test-overlay" onClick={onClose}>
      <div className="audio-test-panel" onClick={(e) => e.stopPropagation()}>
        <div className="audio-test-header">
          <h2>🎤 音频输入诊断工具</h2>
          <button className="close-btn" onClick={onClose}>×</button>
        </div>

        <div className="audio-test-content">
          {/* 设备选择 */}
          <div className="device-selection">
            <h3>选择音频输入设备</h3>
            <div className="device-list">
              {devices.map((device) => (
                <label key={device.id} className="device-option">
                  <input
                    type="radio"
                    name="audioDevice"
                    value={device.id}
                    checked={selectedDevice === device.id}
                    onChange={(e) => setSelectedDevice(e.target.value)}
                    disabled={!device.is_available}
                  />
                  <span className={`device-info ${!device.is_available ? 'unavailable' : ''}`}>
                    <span className="device-name">{device.name}</span>
                    {device.is_default && <span className="default-badge">默认</span>}
                    {!device.is_available && <span className="unavailable-badge">不可用</span>}
                  </span>
                </label>
              ))}
            </div>
          </div>

          {/* 测试配置 */}
          <div className="test-config">
            <h3>测试配置</h3>
            <div className="duration-config">
              <label>测试持续时间:</label>
              <select 
                value={testDuration} 
                onChange={(e) => setTestDuration(Number(e.target.value))}
                disabled={isTestingAudio}
              >
                <option value={1}>1秒</option>
                <option value={3}>3秒</option>
                <option value={5}>5秒</option>
                <option value={10}>10秒</option>
              </select>
            </div>
          </div>

          {/* 测试按钮 */}
          <div className="test-actions">
            <button 
              className={`test-btn ${isTestingAudio ? 'testing' : ''}`}
              onClick={testAudioInput}
              disabled={isTestingAudio || !selectedDevice}
            >
              {isTestingAudio ? `🔴 测试中... (${testDuration}秒)` : '🎤 开始音频测试'}
            </button>
            <button 
              className="refresh-btn"
              onClick={loadAudioDevices}
              disabled={isTestingAudio}
            >
              🔄 刷新设备
            </button>
          </div>

          {/* 测试结果 */}
          {testResult && (
            <div className="test-results">
              <h3>测试结果</h3>
              <div className="result-text">
                {testResult.split('\n').map((line, index) => (
                  <div key={index} className="result-line">{line}</div>
                ))}
              </div>
              
              {lastTestData && (
                <div className="audio-analysis">
                  <h4>音频分析</h4>
                  
                  <div className="volume-meter">
                    <div className="meter-label">最大音量: {lastTestData.maxVolume.toFixed(4)}</div>
                    <div className="meter-bar">
                      <div 
                        className="meter-fill" 
                        style={{ 
                          width: `${getVolumeBarWidth(lastTestData.maxVolume)}%`,
                          backgroundColor: getVolumeColor(lastTestData.maxVolume)
                        }}
                      ></div>
                    </div>
                  </div>

                  <div className="volume-meter">
                    <div className="meter-label">RMS音量: {lastTestData.rmsVolume.toFixed(4)}</div>
                    <div className="meter-bar">
                      <div 
                        className="meter-fill" 
                        style={{ 
                          width: `${getVolumeBarWidth(lastTestData.rmsVolume)}%`,
                          backgroundColor: getVolumeColor(lastTestData.rmsVolume)
                        }}
                      ></div>
                    </div>
                  </div>

                  <div className="audio-stats">
                    <div className="stat-item">
                      <span className="stat-label">样本数:</span>
                      <span className="stat-value">{lastTestData.sampleCount.toLocaleString()}</span>
                    </div>
                    <div className="stat-item">
                      <span className="stat-label">实际时长:</span>
                      <span className="stat-value">{lastTestData.duration.toFixed(2)}秒</span>
                    </div>
                  </div>

                  <div className="diagnosis">
                    <h4>诊断建议</h4>
                    <p className="diagnosis-text">{getDiagnosisText()}</p>
                  </div>
                </div>
              )}
            </div>
          )}

          {/* 帮助说明 */}
          <div className="help-section">
            <h3>使用说明</h3>
            <ul>
              <li>🎤 选择要测试的音频输入设备</li>
              <li>⏱️ 设置测试持续时间（建议3-5秒）</li>
              <li>🔴 点击开始测试，测试期间请对着麦克风说话</li>
              <li>📊 查看音频质量分析和诊断建议</li>
              <li>🔧 根据建议调整麦克风设置或环境</li>
            </ul>
            
            <div className="volume-guide">
              <h4>音量参考标准</h4>
              <div className="volume-levels">
                <div className="level-item">
                  <span className="level-color" style={{backgroundColor: '#ff4444'}}></span>
                  <span>过低 (&lt;0.01) - 检查权限和连接</span>
                </div>
                <div className="level-item">
                  <span className="level-color" style={{backgroundColor: '#ff9800'}}></span>
                  <span>较低 (0.01-0.05) - 建议提高音量</span>
                </div>
                <div className="level-item">
                  <span className="level-color" style={{backgroundColor: '#4caf50'}}></span>
                  <span>正常 (0.05-0.3) - 最佳录音范围</span>
                </div>
                <div className="level-item">
                  <span className="level-color" style={{backgroundColor: '#2196f3'}}></span>
                  <span>较高 (&gt;0.3) - 可能失真</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default AudioInputTest;