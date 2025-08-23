import React, { useEffect, useState } from 'react';
import { getVersion } from '@tauri-apps/api/app';

const AboutApp: React.FC = () => {
  const [version, setVersion] = useState<string>('');

  useEffect(() => {
    let mounted = true;
    getVersion().then(v => {
      if (mounted) setVersion(v);
    }).catch(() => {});
    return () => { mounted = false; };
  }, []);

  return (
    <div className="about-app" style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
      <div style={{ fontSize: 20, fontWeight: 600 }}>Recording King（录音王）</div>
      <div style={{ color: 'var(--spokenly-text-secondary, #666)' }}>
        版本：{version ? `v${version}` : '获取中...'}
      </div>
      <div>
        开发公司：<strong>miaoda</strong>（AI 科技公司）
      </div>
      <div>
        官网：<a href="https://miaoda.xin" target="_blank" rel="noreferrer">miaoda.xin</a>
      </div>
      <div style={{ marginTop: 8, color: 'var(--spokenly-text-secondary, #666)' }}>
        Recording King 由 miaoda 提供技术支持，聚焦语音输入与 AI 智能文本工作流。
      </div>
    </div>
  );
};

export default AboutApp;