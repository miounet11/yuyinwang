import React, { useEffect, useState } from 'react';

export type ToastType = 'success' | 'error' | 'warning' | 'info';

export interface ToastData {
  id: string;
  type: ToastType;
  message: string;
  duration?: number;
}

const COLORS: Record<ToastType, { bg: string; border: string; icon: string }> = {
  success: { bg: 'rgba(34,197,94,0.12)', border: 'rgba(34,197,94,0.25)', icon: '#22c55e' },
  error: { bg: 'rgba(239,68,68,0.12)', border: 'rgba(239,68,68,0.25)', icon: '#ef4444' },
  warning: { bg: 'rgba(245,158,11,0.12)', border: 'rgba(245,158,11,0.25)', icon: '#f59e0b' },
  info: { bg: 'rgba(59,130,246,0.12)', border: 'rgba(59,130,246,0.25)', icon: '#3b82f6' },
};

const ICONS: Record<ToastType, string> = { success: '✓', error: '✕', warning: '⚠', info: 'ℹ' };

const Toast: React.FC<{ toast: ToastData; onClose: (id: string) => void }> = ({ toast, onClose }) => {
  const [visible, setVisible] = useState(false);
  useEffect(() => {
    requestAnimationFrame(() => setVisible(true));
    const t = setTimeout(() => { setVisible(false); setTimeout(() => onClose(toast.id), 200); }, (toast.duration || 3000) - 200);
    return () => clearTimeout(t);
  }, [toast.id, toast.duration, onClose]);

  const c = COLORS[toast.type];
  return (
    <div style={{
      display: 'flex', alignItems: 'center', gap: '10px',
      padding: '10px 16px', background: c.bg, border: `1px solid ${c.border}`,
      borderRadius: '10px', marginBottom: '8px', minWidth: '240px', maxWidth: '360px',
      backdropFilter: 'blur(12px)',
      transform: visible ? 'translateX(0)' : 'translateX(20px)',
      opacity: visible ? 1 : 0, transition: 'all 0.2s ease',
    }}>
      <span style={{
        width: '20px', height: '20px', borderRadius: '50%', background: c.icon,
        color: '#fff', fontSize: '11px', fontWeight: 700,
        display: 'flex', alignItems: 'center', justifyContent: 'center', flexShrink: 0,
      }}>{ICONS[toast.type]}</span>
      <span style={{ flex: 1, fontSize: '13px', color: '#e8e8ec', fontWeight: 500 }}>{toast.message}</span>
      <button onClick={() => onClose(toast.id)} style={{
        background: 'none', border: 'none', color: '#666', cursor: 'pointer', fontSize: '14px', padding: '2px',
      }}>✕</button>
    </div>
  );
};

export const ToastContainer: React.FC<{ toasts: ToastData[]; onClose: (id: string) => void }> = ({ toasts, onClose }) => (
  <div style={{ position: 'fixed', top: '16px', right: '16px', zIndex: 9999 }}>
    {toasts.map((t) => <Toast key={t.id} toast={t} onClose={onClose} />)}
  </div>
);

export default Toast;
