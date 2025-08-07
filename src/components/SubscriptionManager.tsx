import React, { useState, useEffect } from 'react';
import { ttsService } from '../services/ttsService';
import './SubscriptionManager.css';

interface SubscriptionManagerProps {
  isVisible: boolean;
  onClose: () => void;
  onUpgradeSuccess?: () => void;
  isFirstLaunch?: boolean;
}

const SubscriptionManager: React.FC<SubscriptionManagerProps> = ({
  isVisible,
  onClose,
  onUpgradeSuccess,
  isFirstLaunch = false
}) => {
  const [trialInfo, setTrialInfo] = useState<any>(null);
  const [selectedPlan, setSelectedPlan] = useState<'monthly' | 'yearly'>('monthly');
  const [isProcessing, setIsProcessing] = useState(false);

  useEffect(() => {
    if (isVisible) {
      updateTrialInfo();
      const interval = setInterval(updateTrialInfo, 60000); // 每分钟更新一次
      return () => clearInterval(interval);
    }
  }, [isVisible]);

  const updateTrialInfo = () => {
    const info = ttsService.getTrialInfo();
    setTrialInfo(info);
  };

  const handleUpgrade = async () => {
    setIsProcessing(true);
    
    try {
      // 这里应该集成实际的支付流程
      // 模拟支付成功后升级
      await simulatePayment(selectedPlan);
      
      const subscription = ttsService.upgradeToPro(selectedPlan);
      
      // 显示成功消息
      alert(`升级成功！您现在是 Pro 用户，订阅将持续到 ${new Date(subscription.expiresAt).toLocaleDateString('zh-CN')}`);
      
      if (onUpgradeSuccess) {
        onUpgradeSuccess();
      }
      
      onClose();
    } catch (error) {
      console.error('升级失败:', error);
      alert('升级失败，请稍后重试');
    } finally {
      setIsProcessing(false);
    }
  };

  const simulatePayment = (_plan: 'monthly' | 'yearly'): Promise<void> => {
    return new Promise((resolve) => {
      // 模拟支付处理时间
      setTimeout(() => {
        resolve();
      }, 2000);
    });
  };

  const getPricing = () => {
    return {
      monthly: {
        price: '¥29',
        period: '/月',
        save: '',
        total: '¥29/月'
      },
      yearly: {
        price: '¥299',
        period: '/年',
        save: '节省 ¥49',
        total: '¥24.9/月'
      }
    };
  };

  if (!isVisible) return null;

  const pricing = getPricing();

  return (
    <div className="subscription-overlay" onClick={onClose}>
      <div className="subscription-dialog" onClick={(e) => e.stopPropagation()}>
        {!isFirstLaunch && <button className="close-btn" onClick={onClose}>✕</button>}
        {isFirstLaunch && <button className="skip-btn" onClick={onClose}>稍后选择</button>}
        
        <div className="subscription-header">
          <h2>🚀 升级到 Spokenly Pro</h2>
          <p>解锁所有高级功能，享受无限制的语音服务</p>
        </div>

        {/* 试用状态 */}
        {trialInfo && !trialInfo.isPro && (
          <div className={`trial-banner ${trialInfo.daysLeft <= 1 ? 'expiring' : ''}`}>
            <div className="trial-icon">⏰</div>
            <div className="trial-text">
              <h4>{trialInfo.message}</h4>
              {trialInfo.daysLeft === 0 && trialInfo.hoursLeft && (
                <p>升级到 Pro 继续使用所有功能</p>
              )}
            </div>
          </div>
        )}

        {/* Pro 功能列表 */}
        <div className="features-section">
          <h3>Pro 版本包含：</h3>
          <ul className="features-list">
            <li>
              <span className="feature-icon">✅</span>
              <span>无限制语音转文字</span>
            </li>
            <li>
              <span className="feature-icon">✅</span>
              <span>高质量 TTS 语音合成</span>
            </li>
            <li>
              <span className="feature-icon">✅</span>
              <span>多语言支持与实时翻译</span>
            </li>
            <li>
              <span className="feature-icon">✅</span>
              <span>离线模式支持</span>
            </li>
            <li>
              <span className="feature-icon">✅</span>
              <span>自定义快捷键</span>
            </li>
            <li>
              <span className="feature-icon">✅</span>
              <span>优先技术支持</span>
            </li>
            <li>
              <span className="feature-icon">✅</span>
              <span>云端同步设置</span>
            </li>
            <li>
              <span className="feature-icon">✅</span>
              <span>批量处理功能</span>
            </li>
          </ul>
        </div>

        {/* 价格选择 */}
        <div className="pricing-section">
          <h3>选择您的计划：</h3>
          <div className="pricing-cards">
            <div 
              className={`pricing-card ${selectedPlan === 'monthly' ? 'selected' : ''}`}
              onClick={() => setSelectedPlan('monthly')}
            >
              <div className="plan-name">月付计划</div>
              <div className="plan-price">
                <span className="price">{pricing.monthly.price}</span>
                <span className="period">{pricing.monthly.period}</span>
              </div>
              <div className="plan-total">{pricing.monthly.total}</div>
            </div>

            <div 
              className={`pricing-card ${selectedPlan === 'yearly' ? 'selected' : ''} popular`}
              onClick={() => setSelectedPlan('yearly')}
            >
              <div className="popular-badge">最受欢迎</div>
              <div className="plan-name">年付计划</div>
              <div className="plan-price">
                <span className="price">{pricing.yearly.price}</span>
                <span className="period">{pricing.yearly.period}</span>
              </div>
              <div className="plan-save">{pricing.yearly.save}</div>
              <div className="plan-total">{pricing.yearly.total}</div>
            </div>
          </div>
        </div>

        {/* 升级按钮 */}
        <div className="subscription-footer">
          <button 
            className="upgrade-btn"
            onClick={handleUpgrade}
            disabled={isProcessing}
          >
            {isProcessing ? (
              <span>处理中...</span>
            ) : (
              <span>
                立即升级 - {selectedPlan === 'monthly' ? pricing.monthly.price : pricing.yearly.price}
                {selectedPlan === 'monthly' ? '/月' : '/年'}
              </span>
            )}
          </button>
          
          <p className="payment-note">
            支付安全由支付宝/微信保障 • 随时可取消订阅
          </p>
        </div>

        {/* 常见问题 */}
        <div className="faq-section">
          <h4>常见问题：</h4>
          <details>
            <summary>如何取消订阅？</summary>
            <p>您可以随时在设置中取消订阅，已支付的期限内仍可继续使用 Pro 功能。</p>
          </details>
          <details>
            <summary>支持哪些支付方式？</summary>
            <p>我们支持支付宝、微信支付、银联卡等多种支付方式。</p>
          </details>
          <details>
            <summary>可以更换订阅计划吗？</summary>
            <p>可以，您可以随时从月付升级到年付，差价将自动计算。</p>
          </details>
        </div>
      </div>
    </div>
  );
};

export default SubscriptionManager;