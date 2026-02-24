/**
 * RecordShortcutModal - 录制快捷键弹窗组件
 *
 * 功能：
 * 1. 显示四个修饰键图标（⌘ ⌥ ⇧ ⌃）
 * 2. 按下修饰键时实时高亮对应图标
 * 3. 检测普通键按下，验证组合有效性（至少 1 修饰键 + 1 普通键）
 * 4. 显示当前组合的文字描述
 * 5. 保存/取消按钮
 */

import React, { useEffect, useState } from 'react';
import type {
  RecordShortcutModalProps,
  RecordShortcutModalState,
  ModifierKey,
  CustomShortcut,
} from '../../shared/types';
import { validateCustomShortcut, generateShortcutLabel } from '../../shared/utils';
import './RecordShortcutModal.css';

export const RecordShortcutModal: React.FC<RecordShortcutModalProps> = ({
  isOpen,
  onClose,
  onSave,
  currentShortcut,
}) => {
  const [state, setState] = useState<RecordShortcutModalState>({
    pressedModifiers: new Set(),
    pressedKey: null,
    isValid: false,
  });

  // 修饰键配置
  const modifierKeys: Array<{ key: ModifierKey; symbol: string; label: string }> = [
    { key: 'cmd', symbol: '⌘', label: 'Command' },
    { key: 'opt', symbol: '⌥', label: 'Option' },
    { key: 'shift', symbol: '⇧', label: 'Shift' },
    { key: 'ctrl', symbol: '⌃', label: 'Control' },
  ];

  // 键盘事件处理
  useEffect(() => {
    if (!isOpen) return;

    const handleKeyDown = (event: KeyboardEvent) => {
      event.preventDefault();

      const MODIFIER_KEYS = new Set(['Meta', 'Control', 'Shift', 'Alt']);
      const newModifiers = new Set<ModifierKey>();

      // 收集修饰键状态
      if (event.metaKey) newModifiers.add('cmd');
      if (event.ctrlKey) newModifiers.add('ctrl');
      if (event.shiftKey) newModifiers.add('shift');
      if (event.altKey) newModifiers.add('opt');

      // 检测普通键
      let pressedKey: string | null = null;
      if (!MODIFIER_KEYS.has(event.key)) {
        pressedKey = event.key.length === 1 ? event.key.toUpperCase() : event.key;
      }

      // 验证组合
      const validation = validateCustomShortcut(newModifiers, pressedKey);

      setState({
        pressedModifiers: newModifiers,
        pressedKey,
        isValid: validation.isValid,
      });
    };

    const handleKeyUp = (event: KeyboardEvent) => {
      event.preventDefault();

      // 当所有键释放时，如果已有有效组合，保持状态
      // 否则重置修饰键状态
      const anyModifierPressed = event.metaKey || event.ctrlKey || event.shiftKey || event.altKey;

      if (!anyModifierPressed && !state.isValid) {
        setState({
          pressedModifiers: new Set(),
          pressedKey: null,
          isValid: false,
        });
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    window.addEventListener('keyup', handleKeyUp);

    return () => {
      window.removeEventListener('keydown', handleKeyDown);
      window.removeEventListener('keyup', handleKeyUp);
    };
  }, [isOpen, state.isValid]);

  // 重置状态
  const resetState = () => {
    setState({
      pressedModifiers: new Set(),
      pressedKey: null,
      isValid: false,
    });
  };

  // 处理关闭
  const handleClose = () => {
    resetState();
    onClose();
  };

  // 处理保存
  const handleSave = () => {
    if (!state.isValid || !state.pressedKey) return;

    const shortcut: CustomShortcut = {
      type: 'custom',
      modifiers: Array.from(state.pressedModifiers),
      key: state.pressedKey,
      displayLabel: generateShortcutLabel({
        type: 'custom',
        modifiers: Array.from(state.pressedModifiers),
        key: state.pressedKey,
        displayLabel: '',
      }),
    };

    onSave(shortcut);
    resetState();
  };

  // 生成当前组合的文字描述
  const getCurrentCombinationText = (): string => {
    if (state.pressedModifiers.size === 0 && !state.pressedKey) {
      return '请按下快捷键组合...';
    }

    const modifierSymbols = modifierKeys
      .filter((mk) => state.pressedModifiers.has(mk.key))
      .map((mk) => mk.symbol)
      .join('');

    if (state.pressedKey) {
      return `${modifierSymbols}${state.pressedKey}`;
    }

    if (state.pressedModifiers.size > 0) {
      return `${modifierSymbols} + ?`;
    }

    return '请按下快捷键组合...';
  };

  // 获取验证错误信息
  const getValidationMessage = (): string | null => {
    const validation = validateCustomShortcut(state.pressedModifiers, state.pressedKey);
    return validation.errorMessage || null;
  };

  if (!isOpen) return null;

  return (
    <div className="modal-overlay" onClick={handleClose}>
      <div className="modal-content" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h2 className="modal-title">录制快捷键</h2>
          <button className="modal-close" onClick={handleClose}>
            ✕
          </button>
        </div>

        <div className="modal-body">
          <p className="modal-instruction">
            按下修饰键和普通键的组合。至少需要一个修饰键和一个普通键。
          </p>

          {/* 修饰键图标区域 */}
          <div className="modifier-keys-grid">
            {modifierKeys.map((mk) => (
              <div
                key={mk.key}
                className={`modifier-key ${
                  state.pressedModifiers.has(mk.key) ? 'active' : ''
                }`}
              >
                <div className="modifier-symbol">{mk.symbol}</div>
                <div className="modifier-label">{mk.label}</div>
              </div>
            ))}
          </div>

          {/* 当前组合显示 */}
          <div className="current-combination">
            <div className="combination-label">当前组合</div>
            <div
              className={`combination-display ${
                state.isValid ? 'valid' : state.pressedKey ? 'invalid' : ''
              }`}
            >
              {getCurrentCombinationText()}
            </div>
          </div>

          {/* 验证消息 */}
          {!state.isValid && (state.pressedModifiers.size > 0 || state.pressedKey) && (
            <div className="validation-message">
              {getValidationMessage()}
            </div>
          )}

          {/* 当前快捷键提示 */}
          {currentShortcut && (
            <div className="current-shortcut-hint">
              当前快捷键: {currentShortcut.displayLabel}
            </div>
          )}
        </div>

        <div className="modal-footer">
          <button className="modal-btn modal-btn-secondary" onClick={handleClose}>
            取消
          </button>
          <button
            className="modal-btn modal-btn-primary"
            onClick={handleSave}
            disabled={!state.isValid}
          >
            保存
          </button>
        </div>
      </div>
    </div>
  );
};
