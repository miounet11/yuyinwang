import React, { useState, useEffect, useRef } from 'react';
import { listen } from '@tauri-apps/api/event';
import { useAppStore } from '../../shared/stores/useAppStore';
import type { ShortcutPreset, ActivationMode, CustomShortcut } from '../../shared/types';
import { isSystemShortcutConflict, generateShortcutLabel } from '../../shared/utils';
import './ShortcutSettings.css';

const SHORTCUT_PRESETS: Array<{ value: ShortcutPreset; label: string }> = [
  { value: 'none', label: '未指定' },
  { value: 'right-cmd', label: '右 ⌘' },
  { value: 'right-opt', label: '右 ⌥' },
  { value: 'right-shift', label: '右 ⇧' },
  { value: 'right-ctrl', label: '右 ⌃' },
  { value: 'opt-cmd', label: '⌥ + ⌘' },
  { value: 'ctrl-cmd', label: '⌃ + ⌘' },
  { value: 'ctrl-opt', label: '⌃ + ⌥' },
  { value: 'shift-cmd', label: '⇧ + ⌘' },
  { value: 'opt-shift', label: '⌥ + ⇧' },
  { value: 'ctrl-shift', label: '⌃ + ⇧' },
  { value: 'fn', label: 'Fn' },
];

const ACTIVATION_MODES: Array<{ value: ActivationMode; label: string }> = [
  { value: 'hold-or-toggle', label: '按住或切换' },
  { value: 'toggle', label: '切换' },
  { value: 'hold', label: '按住' },
  { value: 'double-click', label: '双击' },
];

export const ShortcutSettings: React.FC = () => {
  const { shortcutSettings, setShortcutPreset, setCustomShortcut, setActivationMode, addToast } = useAppStore();
  const [showRecordModal, setShowRecordModal] = useState(false);
  const [testText, setTestText] = useState('');
  const [isTestRecording, setIsTestRecording] = useState(false);
  const [isTestTranscribing, setIsTestTranscribing] = useState(false);
  const testAreaRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    const unlistens = [
      listen('quick-input-started', () => {
        setIsTestRecording(true);
        setIsTestTranscribing(false);
      }),
      listen('quick-input-transcribing', () => {
        setIsTestRecording(false);
        setIsTestTranscribing(true);
      }),
      listen('quick-input-result', (e: any) => {
        setIsTestRecording(false);
        setIsTestTranscribing(false);
        if (e.payload && testAreaRef.current === document.activeElement) {
          setTestText((prev) => prev + (prev ? '\n' : '') + e.payload);
        }
      }),
      listen('quick-input-error', () => {
        setIsTestRecording(false);
        setIsTestTranscribing(false);
      }),
    ];
    return () => { unlistens.forEach((u) => u.then((fn) => fn())); };
  }, []);

  const handlePresetChange = async (preset: ShortcutPreset) => {
    if (preset === 'custom') {
      setShowRecordModal(true);
    } else {
      await setShortcutPreset(preset);
    }
  };

  const getCurrentPresetValue = (): string => {
    const current = shortcutSettings.selectedShortcut;
    return typeof current === 'string' ? current : 'custom';
  };

  const getCurrentShortcutLabel = (): string => {
    const current = shortcutSettings.selectedShortcut;
    if (typeof current === 'string') {
      const preset = SHORTCUT_PRESETS.find((p) => p.value === current);
      return preset?.label || '未指定';
    }
    return current.displayLabel;
  };

  const getTestHintText = (): string => {
    const label = getCurrentShortcutLabel();
    if (getCurrentPresetValue() === 'none') return '请先选择一个快捷键。';
    const mode = shortcutSettings.activationMode;
    switch (mode) {
      case 'hold':
        return `按住 ${label} 开始录音，松开停止。`;
      case 'toggle':
        return `按 ${label} 开始录音，然后再次按 ${label} 停止。`;
      case 'double-click':
        return `快速双击 ${label} 开始录音，再按一次停止。`;
      case 'hold-or-toggle':
      default:
        return `按住 ${label} 录音（松开停止），或短按切换录音。`;
    }
  };

  const isFnSelected = getCurrentPresetValue() === 'fn';

  return (
    <div className="page">
      <h1 className="page-title">快捷键</h1>
      <p className="page-desc">选择您喜欢的键盘修饰键来启动 Recording King。仅按这些修饰键即可开始录音。</p>

      {/* 录音快捷键 */}
      <div className="section">
        <h2 className="section-title">录音快捷键</h2>
        <div className="card">
          <div className="card-row">
            <div className="card-row-label">
              <span className="row-icon">⌨️</span>
              <span>快捷键</span>
            </div>
            <div className="shortcut-controls">
              <select
                className="inline-select"
                value={shortcutSettings.activationMode}
                onChange={(e) => setActivationMode(e.target.value as ActivationMode)}
              >
                {ACTIVATION_MODES.map((mode) => (
                  <option key={mode.value} value={mode.value}>{mode.label}</option>
                ))}
              </select>
              <select
                className="inline-select"
                value={getCurrentPresetValue()}
                onChange={(e) => handlePresetChange(e.target.value as ShortcutPreset)}
              >
                {SHORTCUT_PRESETS.map((preset) => (
                  <option key={preset.value} value={preset.value}>{preset.label}</option>
                ))}
                <option value="custom">录制快捷键...</option>
              </select>
            </div>
          </div>
        </div>
        <p className="section-hint">
          配置快捷键及其激活方式：按住或切换（自动检测）、切换（点击开始/停止）、按住（按下时录音）或双击（快速按两次）。
        </p>
      </div>

      {/* Fn 键提示 */}
      {isFnSelected && (
        <div className="section">
          <div className="card fn-warning-card">
            <div className="card-row fn-warning-row">
              <div>
                <div className="fn-warning-header">
                  <span className="fn-warning-icon">⚠️</span>
                  <span className="fn-warning-title">使用 Fn 键</span>
                </div>
                <div className="fn-warning-body">
                  要单独使用 Fn 键：
                  <ul className="fn-steps">
                    <li>打开系统设置 → 键盘</li>
                    <li>点击"按下 🌐 键以"下拉菜单</li>
                    <li>选择"无操作"</li>
                    <li>这允许 Recording King 检测 Fn 键按下</li>
                  </ul>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Escape 取消录音 */}
      <div className="section">
        <div className="card">
          <div className="card-row">
            <div className="card-row-label">
              <span className="row-icon">⏱</span>
              <span>使用 Escape 键取消录音</span>
            </div>
            <button
              className={`toggle ${shortcutSettings.escToCancel ? 'on' : ''}`}
              onClick={() => {
                addToast('info', shortcutSettings.escToCancel ? 'Escape 取消已关闭' : 'Escape 取消已开启');
              }}
              aria-label="切换 Escape 取消录音"
            />
          </div>
        </div>
      </div>

      {/* 测试快捷键 */}
      <div className="section">
        <h2 className="section-title">测试您的快捷键</h2>
        <div className="test-area">
          <div className={`test-hint ${isTestRecording ? 'recording' : isTestTranscribing ? 'transcribing' : ''}`}>
            <span className="test-hint-icon">
              {isTestRecording ? '🔴' : isTestTranscribing ? '⏳' : '⌨️'}
            </span>
            <span className="test-hint-text">
              {isTestRecording
                ? '正在录音...松开或再次按下快捷键停止。'
                : isTestTranscribing
                  ? '正在转录...'
                  : getTestHintText()}
            </span>
          </div>
          <textarea
            ref={testAreaRef}
            className={`test-textarea ${isTestRecording ? 'test-recording' : ''}`}
            placeholder="首先点击下方的文本框。"
            value={testText}
            onChange={(e) => setTestText(e.target.value)}
          />
        </div>
      </div>

      {/* 录制快捷键弹窗 */}
      {showRecordModal && (
        <RecordShortcutModal
          isOpen={showRecordModal}
          onClose={() => setShowRecordModal(false)}
          onSave={async (shortcut) => {
            if (isSystemShortcutConflict(shortcut)) {
              addToast('error', '该快捷键与系统关键快捷键冲突，请选择其他组合');
              return;
            }
            await setCustomShortcut(shortcut);
            setShowRecordModal(false);
          }}
        />
      )}
    </div>
  );
};

// 录制快捷键弹窗组件
const RecordShortcutModal: React.FC<{
  isOpen: boolean;
  onClose: () => void;
  onSave: (shortcut: CustomShortcut) => void;
}> = ({ isOpen, onClose, onSave }) => {
  const [pressedModifiers, setPressedModifiers] = useState<Set<string>>(new Set());
  const [pressedKey, setPressedKey] = useState<string | null>(null);
  const [isValid, setIsValid] = useState(false);

  const MODIFIER_KEYS = [
    { key: 'cmd', symbol: '⌘', label: 'Command' },
    { key: 'opt', symbol: '⌥', label: 'Option' },
    { key: 'shift', symbol: '⇧', label: 'Shift' },
    { key: 'ctrl', symbol: '⌃', label: 'Control' },
  ];

  const handleKeyDown = (e: React.KeyboardEvent) => {
    e.preventDefault();
    const modifiers = new Set<string>();
    if (e.metaKey) modifiers.add('cmd');
    if (e.altKey) modifiers.add('opt');
    if (e.shiftKey) modifiers.add('shift');
    if (e.ctrlKey) modifiers.add('ctrl');
    setPressedModifiers(modifiers);

    const key = e.key;
    if (!['Meta', 'Control', 'Shift', 'Alt'].includes(key)) {
      const normalizedKey = key.length === 1 ? key.toUpperCase() : key;
      setPressedKey(normalizedKey);
      setIsValid(modifiers.size >= 1);
    } else {
      setPressedKey(null);
      setIsValid(false);
    }
  };

  const handleSave = () => {
    if (!isValid || !pressedKey) return;
    const modifiers = Array.from(pressedModifiers) as Array<'cmd' | 'opt' | 'shift' | 'ctrl'>;
    const displayLabel = generateShortcutLabel({
      type: 'custom', modifiers, key: pressedKey, displayLabel: '',
    });
    onSave({ type: 'custom', modifiers, key: pressedKey, displayLabel });
  };

  if (!isOpen) return null;

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h2 className="modal-title">录制快捷键</h2>
          <button className="modal-close" onClick={onClose}>✕</button>
        </div>
        <div className="modal-body">
          <p className="modal-desc">按下修饰键和普通键的组合（至少一个修饰键 + 一个普通键）</p>
          <div className="modifier-keys">
            {MODIFIER_KEYS.map((mod) => (
              <div key={mod.key} className={`modifier-key ${pressedModifiers.has(mod.key) ? 'active' : ''}`}>
                <div className="modifier-symbol">{mod.symbol}</div>
                <div className="modifier-label">{mod.label}</div>
              </div>
            ))}
          </div>
          <div className="key-capture-area">
            <input
              type="text"
              className="key-capture-input"
              placeholder="点击此处并按下快捷键组合..."
              onKeyDown={handleKeyDown}
              autoFocus
              readOnly
              value={
                pressedKey
                  ? `${Array.from(pressedModifiers).map((m) => MODIFIER_KEYS.find((mk) => mk.key === m)?.symbol).join('')}${pressedKey}`
                  : ''
              }
            />
          </div>
          {pressedModifiers.size === 0 && <div className="validation-message error">需要至少一个修饰键</div>}
          {pressedModifiers.size > 0 && !pressedKey && <div className="validation-message warning">需要一个普通键</div>}
          {isValid && <div className="validation-message success">快捷键组合有效</div>}
        </div>
        <div className="modal-footer">
          <button className="btn-secondary" onClick={onClose}>取消</button>
          <button className="btn-primary" onClick={handleSave} disabled={!isValid}>保存</button>
        </div>
      </div>
    </div>
  );
};
