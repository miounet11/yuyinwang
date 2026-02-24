import React, { useEffect, useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { useAppStore } from '../../shared/stores/useAppStore';
import './PermissionsPage.css';

interface PermissionItem {
  id: string;
  icon: string;
  name: string;
  description: string;
  required: boolean;
  color: string;
  howTo: string;
}

const PERMISSIONS: PermissionItem[] = [
  {
    id: 'microphone', icon: '🎙', name: '麦克风权限',
    description: '用于录制音频和语音识别',
    required: true, color: '#22c55e',
    howTo: '系统设置 → 隐私与安全性 → 麦克风',
  },
  {
    id: 'accessibility', icon: '♿', name: '辅助功能权限',
    description: '用于系统集成和文本注入',
    required: true, color: '#22c55e',
    howTo: '系统设置 → 隐私与安全性 → 辅助功能',
  },
  {
    id: 'input_monitoring', icon: '⌨️', name: '输入监控权限',
    description: '用于全局快捷键功能（按住说话）',
    required: false, color: '#f59e0b',
    howTo: '系统设置 → 隐私与安全性 → 输入监控',
  },
];

export const PermissionsPage: React.FC = () => {
  const { addToast } = useAppStore();
  const [permStatus, setPermStatus] = useState<Record<string, boolean>>({});
  const [checking, setChecking] = useState(true);
  const [requesting, setRequesting] = useState<string | null>(null);

  const checkPermissions = useCallback(async () => {
    setChecking(true);
    try {
      const accessibility = await invoke<boolean>('check_injection_permission');
      setPermStatus({
        microphone: true,
        accessibility,
        input_monitoring: accessibility,
      });
    } catch (e) {
      console.error(e);
    } finally {
      setChecking(false);
    }
  }, []);

  useEffect(() => { checkPermissions(); }, [checkPermissions]);

  // Auto-refresh permissions every 3 seconds when some are missing
  useEffect(() => {
    const allGranted = PERMISSIONS.filter(p => p.required).every(p => permStatus[p.id]);
    if (allGranted || checking) return;
    const interval = setInterval(checkPermissions, 3000);
    return () => clearInterval(interval);
  }, [permStatus, checking, checkPermissions]);

  const handleRequestPermission = async (id: string) => {
    if (permStatus[id]) return;
    setRequesting(id);
    if (id === 'accessibility' || id === 'input_monitoring') {
      try {
        await invoke('request_injection_permission');
        addToast('info', '请在系统设置中授权 Recording King');
        setTimeout(checkPermissions, 2000);
      } catch (e) { console.error(e); }
    }
    setRequesting(null);
  };

  const allRequired = PERMISSIONS.filter(p => p.required).every(p => permStatus[p.id]);
  const grantedCount = PERMISSIONS.filter(p => permStatus[p.id]).length;

  return (
    <div className="page">
      <h1 className="page-title">权限管理</h1>
      <p className="page-desc">配置系统权限以启用所有功能</p>

      <div className="perm-card-wrap">
        <div className="perm-card-header">
          <div className="perm-shield">🛡</div>
          <h2>系统权限管理</h2>
          <p>Recording King 需要以下权限才能正常工作</p>
          <div className="perm-progress">
            <div className="perm-progress-bar">
              <div className="perm-progress-fill" style={{ width: `${(grantedCount / PERMISSIONS.length) * 100}%` }} />
            </div>
            <span className="perm-progress-text">{grantedCount}/{PERMISSIONS.length} 已授权</span>
          </div>
        </div>

        <div className="perm-list">
          {PERMISSIONS.map((perm) => {
            const granted = permStatus[perm.id];
            const isRequesting = requesting === perm.id;
            return (
              <div
                key={perm.id}
                className={`perm-item ${granted ? 'granted' : 'denied'}`}
                style={{ borderColor: granted ? 'rgba(34,197,94,0.2)' : 'rgba(239,68,68,0.2)' }}
                onClick={() => !granted && handleRequestPermission(perm.id)}
              >
                <div className="perm-icon-wrap" style={{ background: `${perm.color}20`, color: perm.color }}>
                  {perm.icon}
                </div>
                <div className="perm-info">
                  <div className="perm-name">
                    {perm.name}
                    {perm.required && <span className="perm-required">必需</span>}
                    {!perm.required && <span className="perm-optional">可选</span>}
                  </div>
                  <div className="perm-desc">{perm.description}</div>
                  {!granted && <div className="perm-howto">{perm.howTo}</div>}
                </div>
                <div className="perm-status">
                  {checking || isRequesting ? (
                    <div className="perm-loading" />
                  ) : granted ? (
                    <span className="perm-check">✓</span>
                  ) : (
                    <button className="perm-grant-btn">授权</button>
                  )}
                </div>
              </div>
            );
          })}
        </div>

        <div className={`perm-summary ${allRequired ? 'ok' : 'warn'}`}>
          {allRequired ? (
            <><span>✓</span> 所有关键权限已配置，功能正常</>
          ) : (
            <><span>⚠</span> 部分权限未配置，某些功能可能受限</>
          )}
        </div>
      </div>
    </div>
  );
};
