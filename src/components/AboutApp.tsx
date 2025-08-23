import React, { useEffect, useState, useRef } from 'react';
import { getVersion } from '@tauri-apps/api/app';
import { invoke } from '@tauri-apps/api/tauri';

const AboutApp: React.FC = () => {
  const [version, setVersion] = useState<string>('');
  const [unlockCount, setUnlockCount] = useState<number>(0);
  const [unlocked, setUnlocked] = useState<boolean>(false);
  const [diagRunning, setDiagRunning] = useState<boolean>(false);
  const [repairRunning, setRepairRunning] = useState<boolean>(false);
  const [result, setResult] = useState<string>('');
  const timerRef = useRef<number | null>(null);

  useEffect(() => {
    let mounted = true;
    getVersion().then(v => {
      if (mounted) setVersion(v);
    }).catch(() => {});
    return () => { mounted = false; if (timerRef.current) window.clearTimeout(timerRef.current); };
  }, []);

  const handleVersionClick = (e: React.MouseEvent) => {
    // 支持按住 Option 并点击，或连续点击 5 次解锁
    const isOption = (e as any).altKey === true;
    if (isOption) {
      setUnlocked(true);
      return;
    }
    setUnlockCount((c) => {
      const next = c + 1;
      if (next >= 5) {
        setUnlocked(true);
        return 0;
      }
      if (timerRef.current) window.clearTimeout(timerRef.current);
      timerRef.current = window.setTimeout(() => setUnlockCount(0), 1000);
      return next;
    });
  };

  const runDiagnostics = async () => {
    try {
      setDiagRunning(true);
      const res = await invoke<string>('run_full_diagnostics');
      setResult(res || '');
    } catch (e: any) {
      setResult(`诊断失败: ${String(e)}`);
    } finally {
      setDiagRunning(false);
    }
  };

  const runSelfRepair = async () => {
    try {
      setRepairRunning(true);
      const res = await invoke<string>('run_self_repair');
      setResult(res || '');
    } catch (e: any) {
      setResult(`自修复失败: ${String(e)}`);
    } finally {
      setRepairRunning(false);
    }
  };

  return (
    <div className="about-app" style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
      <div style={{ fontSize: 20, fontWeight: 600 }}>Recording King（录音王）</div>
      <div 
        style={{ color: 'var(--spokenly-text-secondary, #666)', cursor: 'pointer', userSelect: 'none' }}
        title="版本信息"
        onClick={handleVersionClick}
      >
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

      {unlocked && (
        <div style={{ marginTop: 16, paddingTop: 12, borderTop: '1px solid var(--spokenly-border-subtle, #e5e7eb)' }}>
          <div style={{ fontWeight: 600, marginBottom: 8 }}>诊断与自修复（高级）</div>
          <div style={{ display: 'flex', gap: 8 }}>
            <button onClick={runDiagnostics} disabled={diagRunning}>
              {diagRunning ? '诊断中…' : '运行诊断'}
            </button>
            <button onClick={runSelfRepair} disabled={repairRunning}>
              {repairRunning ? '修复中…' : '一键修复'}
            </button>
          </div>
          {!!result && (
            <pre style={{ marginTop: 10, maxHeight: 180, overflow: 'auto', background: '#f7f7f7', padding: 8, borderRadius: 6 }}>
              {result}
            </pre>
          )}
        </div>
      )}
    </div>
  );
};

export default AboutApp;