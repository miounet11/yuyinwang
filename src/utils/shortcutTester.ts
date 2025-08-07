/**
 * Recording King 快捷键测试器
 * 提供完整的快捷键测试功能实现
 */

export interface ShortcutOption {
  id: string;
  label: string;
  key: string;
  description: string;
  category: 'basic' | 'advanced' | 'special';
}

export interface KeyCombination {
  ctrlKey: boolean;
  shiftKey: boolean;
  altKey: boolean;
  metaKey: boolean;
  key: string;
  code: string;
}

export interface TestResult {
  success: boolean;
  selectedShortcut: ShortcutOption | null;
  pressedKeys: KeyCombination;
  timestamp: number;
  matchType: 'exact' | 'partial' | 'none';
}

export class ShortcutTester {
  private isListening = false;
  private currentShortcut: ShortcutOption | null = null;
  private testStartTime = 0;
  private testResults: TestResult[] = [];
  private keyListeners: ((event: KeyboardEvent) => void)[] = [];
  private testCompleteCallbacks: ((result: TestResult) => void)[] = [];
  private keyPressCallbacks: ((keys: KeyCombination) => void)[] = [];

  // 预定义的快捷键选项
  private predefinedShortcuts: ShortcutOption[] = [
    {
      id: 'fn-key',
      label: 'Fn 键',
      key: 'Fn',
      description: '单独按下 Fn 键进行录音',
      category: 'special'
    },
    {
      id: 'cmd-space',
      label: 'Cmd + Space',
      key: 'Meta+Space',
      description: '命令键 + 空格键组合',
      category: 'basic'
    },
    {
      id: 'cmd-shift-space',
      label: 'Cmd + Shift + Space',
      key: 'Meta+Shift+Space',
      description: '命令键 + Shift + 空格键组合',
      category: 'basic'
    },
    {
      id: 'ctrl-shift-r',
      label: 'Ctrl + Shift + R',
      key: 'Control+Shift+KeyR',
      description: '控制键 + Shift + R 组合',
      category: 'basic'
    },
    {
      id: 'alt-space',
      label: 'Alt + Space',
      key: 'Alt+Space',
      description: 'Alt 键 + 空格键组合',
      category: 'basic'
    },
    {
      id: 'cmd-alt-space',
      label: 'Cmd + Alt + Space',
      key: 'Meta+Alt+Space',
      description: '命令键 + Alt + 空格键组合',
      category: 'advanced'
    },
    {
      id: 'ctrl-alt-shift-space',
      label: 'Ctrl + Alt + Shift + Space',
      key: 'Control+Alt+Shift+Space',
      description: '多键组合快捷键',
      category: 'advanced'
    },
    {
      id: 'double-shift',
      label: '双击 Shift',
      key: 'Shift+Shift',
      description: '快速双击 Shift 键',
      category: 'special'
    },
    {
      id: 'f1-key',
      label: 'F1 键',
      key: 'F1',
      description: '功能键 F1',
      category: 'basic'
    },
    {
      id: 'f13-key',
      label: 'F13 键',
      key: 'F13',
      description: '功能键 F13（TouchBar）',
      category: 'special'
    }
  ];

  constructor() {
    this.initializeKeyListener();
  }

  /**
   * 获取所有可用的快捷键选项
   */
  getShortcutOptions(): ShortcutOption[] {
    return [...this.predefinedShortcuts];
  }

  /**
   * 按分类获取快捷键选项
   */
  getShortcutsByCategory(category: ShortcutOption['category']): ShortcutOption[] {
    return this.predefinedShortcuts.filter(shortcut => shortcut.category === category);
  }

  /**
   * 开始测试指定的快捷键
   */
  startTest(shortcutOption: ShortcutOption): void {
    this.currentShortcut = shortcutOption;
    this.isListening = true;
    this.testStartTime = Date.now();
    
    console.log(`🧪 开始测试快捷键: ${shortcutOption.label}`);
    console.log(`📌 请按下: ${this.formatShortcutDisplay(shortcutOption.key)}`);
  }

  /**
   * 停止当前测试
   */
  stopTest(): void {
    this.isListening = false;
    this.currentShortcut = null;
    this.testStartTime = 0;
    
    console.log('🛑 快捷键测试已停止');
  }

  /**
   * 检测按键组合
   */
  detectShortcut(event: KeyboardEvent): KeyCombination {
    return {
      ctrlKey: event.ctrlKey,
      shiftKey: event.shiftKey,
      altKey: event.altKey,
      metaKey: event.metaKey,
      key: event.key,
      code: event.code
    };
  }

  /**
   * 验证快捷键是否匹配
   */
  validateShortcut(pressedKeys: KeyCombination, targetShortcut: ShortcutOption): 'exact' | 'partial' | 'none' {
    const targetKeys = this.parseShortcutString(targetShortcut.key);
    
    // 处理特殊键
    if (targetShortcut.id === 'fn-key') {
      // Fn 键检测（在大多数浏览器中Fn键无法直接检测）
      return pressedKeys.key === 'Fn' ? 'exact' : 'none';
    }

    if (targetShortcut.id === 'double-shift') {
      // 双击 Shift 检测需要特殊处理
      return this.detectDoubleShift(pressedKeys) ? 'exact' : 'none';
    }

    // 标准组合键验证
    const modifiersMatch = 
      pressedKeys.ctrlKey === targetKeys.ctrl &&
      pressedKeys.shiftKey === targetKeys.shift &&
      pressedKeys.altKey === targetKeys.alt &&
      pressedKeys.metaKey === targetKeys.meta;

    const mainKeyMatch = this.compareMainKey(pressedKeys, targetKeys);

    if (modifiersMatch && mainKeyMatch) {
      return 'exact';
    } else if (modifiersMatch || mainKeyMatch) {
      return 'partial';
    } else {
      return 'none';
    }
  }

  /**
   * 格式化快捷键显示
   */
  formatShortcut(keys: KeyCombination): string {
    const parts: string[] = [];
    
    if (keys.ctrlKey) parts.push('Ctrl');
    if (keys.metaKey) parts.push('Cmd');
    if (keys.altKey) parts.push('Alt');
    if (keys.shiftKey) parts.push('Shift');
    
    if (keys.key && !['Control', 'Meta', 'Alt', 'Shift'].includes(keys.key)) {
      parts.push(this.formatKeyName(keys.key));
    }
    
    return parts.join(' + ');
  }

  /**
   * 格式化快捷键显示（用于UI）
   */
  formatShortcutDisplay(shortcutString: string): string {
    return shortcutString
      .replace(/Meta/g, '⌘')
      .replace(/Control/g, 'Ctrl')
      .replace(/Alt/g, '⌥')
      .replace(/Shift/g, '⇧')
      .replace(/Space/g, '␣')
      .replace(/Key([A-Z])/g, '$1')
      .replace(/\+/g, ' + ');
  }

  /**
   * 保存用户选择
   */
  saveUserChoice(shortcut: ShortcutOption): void {
    const savedShortcuts = this.getSavedShortcuts();
    const existingIndex = savedShortcuts.findIndex(s => s.id === shortcut.id);
    
    if (existingIndex >= 0) {
      savedShortcuts[existingIndex] = shortcut;
    } else {
      savedShortcuts.push(shortcut);
    }
    
    localStorage.setItem('spokenly_user_shortcuts', JSON.stringify(savedShortcuts));
    console.log('💾 用户快捷键偏好已保存:', shortcut.label);
  }

  /**
   * 获取已保存的用户选择
   */
  getSavedShortcuts(): ShortcutOption[] {
    const saved = localStorage.getItem('spokenly_user_shortcuts');
    return saved ? JSON.parse(saved) : [];
  }

  /**
   * 获取测试历史
   */
  getTestResults(): TestResult[] {
    return [...this.testResults];
  }

  /**
   * 清除测试历史
   */
  clearTestResults(): void {
    this.testResults = [];
  }

  /**
   * 添加测试完成回调
   */
  onTestComplete(callback: (result: TestResult) => void): void {
    this.testCompleteCallbacks.push(callback);
  }

  /**
   * 添加按键回调
   */
  onKeyPress(callback: (keys: KeyCombination) => void): void {
    this.keyPressCallbacks.push(callback);
  }

  /**
   * 移除回调
   */
  removeCallback(callback: Function): void {
    this.testCompleteCallbacks = this.testCompleteCallbacks.filter(cb => cb !== callback);
    this.keyPressCallbacks = this.keyPressCallbacks.filter(cb => cb !== callback);
  }

  /**
   * 获取推荐的快捷键
   */
  getRecommendedShortcuts(): ShortcutOption[] {
    return [
      this.predefinedShortcuts.find(s => s.id === 'cmd-space')!,
      this.predefinedShortcuts.find(s => s.id === 'cmd-shift-space')!,
      this.predefinedShortcuts.find(s => s.id === 'fn-key')!,
      this.predefinedShortcuts.find(s => s.id === 'f13-key')!
    ].filter(Boolean);
  }

  // 私有方法

  private initializeKeyListener(): void {
    const keyDownHandler = (event: KeyboardEvent) => {
      if (!this.isListening) return;

      const keys = this.detectShortcut(event);
      
      // 通知按键监听器
      this.keyPressCallbacks.forEach(callback => callback(keys));

      if (this.currentShortcut) {
        const matchType = this.validateShortcut(keys, this.currentShortcut);
        
        if (matchType === 'exact') {
          const result: TestResult = {
            success: true,
            selectedShortcut: this.currentShortcut,
            pressedKeys: keys,
            timestamp: Date.now(),
            matchType
          };

          this.testResults.push(result);
          this.testCompleteCallbacks.forEach(callback => callback(result));
          
          // 自动保存成功的快捷键
          this.saveUserChoice(this.currentShortcut);
          
          console.log('✅ 快捷键测试成功!', this.currentShortcut.label);
          this.stopTest();
        }
      }
    };

    const keyUpHandler = (event: KeyboardEvent) => {
      // 处理需要 keyup 事件的特殊情况
      if (this.isListening && this.currentShortcut?.id === 'double-shift') {
        this.handleDoubleShiftDetection(event);
      }
    };

    document.addEventListener('keydown', keyDownHandler, { capture: true });
    document.addEventListener('keyup', keyUpHandler, { capture: true });

    this.keyListeners.push(keyDownHandler, keyUpHandler);
  }

  private parseShortcutString(shortcutString: string): {
    ctrl: boolean;
    meta: boolean;
    alt: boolean;
    shift: boolean;
    key: string;
  } {
    const parts = shortcutString.split('+');
    
    return {
      ctrl: parts.includes('Control'),
      meta: parts.includes('Meta'),
      alt: parts.includes('Alt'),
      shift: parts.includes('Shift'),
      key: parts.find(p => !['Control', 'Meta', 'Alt', 'Shift'].includes(p)) || ''
    };
  }

  private compareMainKey(pressed: KeyCombination, target: any): boolean {
    if (!target.key) return true;

    // 处理空格键
    if (target.key === 'Space') {
      return pressed.key === ' ' || pressed.code === 'Space';
    }

    // 处理字母键
    if (target.key.startsWith('Key')) {
      const letter = target.key.replace('Key', '');
      return pressed.key.toLowerCase() === letter.toLowerCase() || 
             pressed.code === target.key;
    }

    // 处理功能键
    if (target.key.startsWith('F') && /F\d+/.test(target.key)) {
      return pressed.key === target.key;
    }

    return pressed.key === target.key || pressed.code === target.key;
  }

  private formatKeyName(key: string): string {
    if (key === ' ') return 'Space';
    if (key.length === 1) return key.toUpperCase();
    return key;
  }

  private detectDoubleShift(keys: KeyCombination): boolean {
    // 这里需要实现双击检测逻辑
    // 由于浏览器限制，这可能需要特殊的实现
    return false;
  }

  private handleDoubleShiftDetection(event: KeyboardEvent): void {
    // 实现双击 Shift 检测
    // 这需要跟踪时间间隔和按键序列
  }

  /**
   * 销毁实例，清理事件监听器
   */
  destroy(): void {
    this.keyListeners.forEach(listener => {
      document.removeEventListener('keydown', listener, { capture: true });
      document.removeEventListener('keyup', listener, { capture: true });
    });
    
    this.keyListeners = [];
    this.testCompleteCallbacks = [];
    this.keyPressCallbacks = [];
    this.stopTest();
  }

  /**
   * 创建自定义快捷键选项
   */
  createCustomShortcut(
    id: string,
    label: string,
    key: string,
    description: string,
    category: ShortcutOption['category'] = 'basic'
  ): ShortcutOption {
    const customShortcut: ShortcutOption = {
      id,
      label,
      key,
      description,
      category
    };

    // 添加到预定义列表中（可选）
    const existingIndex = this.predefinedShortcuts.findIndex(s => s.id === id);
    if (existingIndex >= 0) {
      this.predefinedShortcuts[existingIndex] = customShortcut;
    } else {
      this.predefinedShortcuts.push(customShortcut);
    }

    return customShortcut;
  }

  /**
   * 检查快捷键是否可用（不与系统快捷键冲突）
   */
  isShortcutAvailable(keys: KeyCombination): boolean {
    // 检查常见的系统快捷键冲突
    const systemShortcuts = [
      { meta: true, key: 'Tab' }, // Cmd+Tab (应用切换)
      { meta: true, key: 'Space' }, // Cmd+Space (Spotlight)
      { alt: true, key: 'Tab' }, // Alt+Tab
      { ctrl: true, alt: true, key: 'Delete' } // 强制退出
    ];

    return !systemShortcuts.some(shortcut => 
      shortcut.meta === keys.metaKey &&
      shortcut.ctrl === keys.ctrlKey &&
      shortcut.alt === keys.altKey &&
      shortcut.shift === keys.shiftKey &&
      shortcut.key === keys.key
    );
  }
}

// 导出单例实例
export const shortcutTester = new ShortcutTester();

// 导出工具函数
export const ShortcutTestUtils = {
  /**
   * 检测当前平台
   */
  getPlatform(): 'mac' | 'windows' | 'linux' | 'unknown' {
    const platform = navigator.platform.toLowerCase();
    if (platform.includes('mac')) return 'mac';
    if (platform.includes('win')) return 'windows';
    if (platform.includes('linux')) return 'linux';
    return 'unknown';
  },

  /**
   * 获取平台特定的修饰键标记
   */
  getPlatformModifiers(): { cmd: string; ctrl: string; alt: string; shift: string } {
    const isMac = this.getPlatform() === 'mac';
    return {
      cmd: isMac ? '⌘' : 'Ctrl',
      ctrl: 'Ctrl',
      alt: isMac ? '⌥' : 'Alt',
      shift: isMac ? '⇧' : 'Shift'
    };
  },

  /**
   * 验证快捷键字符串格式
   */
  validateShortcutString(shortcutString: string): boolean {
    const validModifiers = ['Control', 'Meta', 'Alt', 'Shift'];
    const parts = shortcutString.split('+');
    
    // 至少需要一个修饰键和一个主键
    if (parts.length < 2) return false;
    
    const modifiers = parts.filter(p => validModifiers.includes(p));
    const mainKeys = parts.filter(p => !validModifiers.includes(p));
    
    return modifiers.length >= 1 && mainKeys.length === 1;
  }
};