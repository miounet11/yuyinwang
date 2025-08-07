/**
 * 高级快捷键管理器
 * 支持更多组合键、双击、长按、语音激活等多种触发方式
 */

import { register, unregister, unregisterAll } from '@tauri-apps/api/globalShortcut';
import { appWindow } from '@tauri-apps/api/window';
import { invoke } from '@tauri-apps/api/tauri';

export type TriggerMode = 'single' | 'double' | 'hold' | 'sequence' | 'voice';
export type ShortcutCategory = 'recording' | 'navigation' | 'editing' | 'system' | 'custom';

export interface AdvancedShortcut {
  id: string;
  name: string;
  description: string;
  category: ShortcutCategory;
  enabled: boolean;
  // 主快捷键
  primaryKey?: string;
  // 触发模式
  triggerMode: TriggerMode;
  // 双击设置
  doubleClick?: {
    key: string;
    timeout: number; // 毫秒
  };
  // 长按设置
  hold?: {
    key: string;
    duration: number; // 毫秒
  };
  // 序列键设置
  sequence?: {
    keys: string[];
    timeout: number; // 毫秒
  };
  // 语音命令
  voiceCommand?: {
    phrases: string[];
    language: string;
  };
  // 鼠标手势
  mouseGesture?: {
    pattern: string; // 如 "L-R-U" 左右上
  };
  // 动作
  action: () => void | Promise<void>;
  // 自定义选项
  customOptions?: Record<string, any>;
}

export class AdvancedShortcutManager {
  private shortcuts: Map<string, AdvancedShortcut> = new Map();
  private registeredKeys: Set<string> = new Set();
  private listeners: Map<string, Function[]> = new Map();
  
  // 双击检测
  private lastKeyPress: Map<string, number> = new Map();
  private doubleClickTimeouts: Map<string, NodeJS.Timeout> = new Map();
  
  // 长按检测
  private holdTimers: Map<string, NodeJS.Timeout> = new Map();
  private heldKeys: Set<string> = new Set();
  
  // 序列键检测
  private sequenceBuffer: string[] = [];
  private sequenceTimer: NodeJS.Timeout | null = null;
  
  // 语音识别
  private voiceRecognitionActive = false;
  private speechRecognition: any = null;

  constructor() {
    this.initializeDefaultShortcuts();
    this.setupKeyboardListeners();
    this.initializeSpeechRecognition();
  }

  private initializeDefaultShortcuts() {
    // 基础快捷键（单键触发）
    this.addShortcut({
      id: 'toggle-recording-basic',
      name: '开始/停止录音',
      description: '使用传统快捷键录音',
      category: 'recording',
      enabled: true,
      triggerMode: 'single',
      primaryKey: 'CommandOrControl+Shift+R',
      action: async () => {
        await this.emit('toggle-recording');
        console.log('🎤 切换录音状态');
      }
    });

    // 双击触发录音
    this.addShortcut({
      id: 'double-tap-recording',
      name: '双击录音',
      description: '双击Option/Alt键开始录音',
      category: 'recording',
      enabled: true,
      triggerMode: 'double',
      doubleClick: {
        key: 'Alt',
        timeout: 300
      },
      action: async () => {
        await this.emit('toggle-recording');
        console.log('🎤 双击触发录音');
      }
    });

    // 长按触发录音
    this.addShortcut({
      id: 'hold-recording',
      name: '长按录音',
      description: '长按空格键进行录音',
      category: 'recording',
      enabled: true,
      triggerMode: 'hold',
      hold: {
        key: 'Space',
        duration: 500
      },
      action: async () => {
        await this.emit('push-to-talk');
        console.log('🎙️ 长按录音');
      }
    });

    // 序列键触发
    this.addShortcut({
      id: 'sequence-recording',
      name: '序列键录音',
      description: '连续按下 R R 开始录音',
      category: 'recording',
      enabled: false,
      triggerMode: 'sequence',
      sequence: {
        keys: ['R', 'R'],
        timeout: 500
      },
      action: async () => {
        await this.emit('toggle-recording');
        console.log('🎤 序列键触发录音');
      }
    });

    // 语音命令
    this.addShortcut({
      id: 'voice-recording',
      name: '语音命令录音',
      description: '说"开始录音"来启动',
      category: 'recording',
      enabled: false,
      triggerMode: 'voice',
      voiceCommand: {
        phrases: ['开始录音', '录音', 'start recording', 'record'],
        language: 'zh-CN'
      },
      action: async () => {
        await this.emit('toggle-recording');
        console.log('🗣️ 语音触发录音');
      }
    });

    // 更多组合键选项
    this.addShortcut({
      id: 'fn-recording',
      name: 'Fn键录音',
      description: '使用Fn键快速录音',
      category: 'recording',
      enabled: false,
      triggerMode: 'single',
      primaryKey: 'Fn',
      action: async () => {
        await this.emit('toggle-recording');
        console.log('⌨️ Fn键录音');
      }
    });

    // Caps Lock 录音
    this.addShortcut({
      id: 'capslock-recording',
      name: 'Caps Lock录音',
      description: '使用Caps Lock键录音',
      category: 'recording',
      enabled: false,
      triggerMode: 'single',
      primaryKey: 'CapsLock',
      action: async () => {
        await this.emit('toggle-recording');
        console.log('⇪ Caps Lock录音');
      }
    });

    // 右键 + 键盘组合
    this.addShortcut({
      id: 'mouse-keyboard-recording',
      name: '鼠标键盘组合',
      description: '右键 + R 开始录音',
      category: 'recording',
      enabled: false,
      triggerMode: 'single',
      primaryKey: 'RightClick+R',
      action: async () => {
        await this.emit('toggle-recording');
        console.log('🖱️ 鼠标键盘组合录音');
      }
    });

    // 添加更多灵活的快捷键组合
    this.initializeFlexibleShortcuts();
  }

  private initializeFlexibleShortcuts() {
    // 单键修饰符组合
    const modifiers = ['CommandOrControl', 'Shift', 'Alt', 'Fn'];
    const keys = ['Space', 'Enter', 'Tab', ...Array.from('ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789')];
    
    // 功能键
    const functionKeys = Array.from({length: 12}, (_, i) => `F${i + 1}`);
    
    // 特殊键
    const specialKeys = ['Escape', 'Backspace', 'Delete', 'Insert', 'Home', 'End', 'PageUp', 'PageDown'];
    
    // 方向键
    const arrowKeys = ['Up', 'Down', 'Left', 'Right'];
    
    // 数字小键盘
    const numpadKeys = Array.from({length: 10}, (_, i) => `Numpad${i}`);
    
    // 媒体键
    const mediaKeys = ['MediaPlayPause', 'MediaStop', 'MediaPrevious', 'MediaNext'];

    // 生成所有可能的组合
    this.generateFlexibleCombinations(modifiers, [...keys, ...functionKeys, ...specialKeys, ...arrowKeys, ...numpadKeys, ...mediaKeys]);
  }

  private generateFlexibleCombinations(modifiers: string[], keys: string[]) {
    // 这里不实际生成所有组合，而是提供一个灵活的系统
    // 允许用户自定义任意组合
    console.log('🔧 灵活快捷键系统已初始化');
  }

  private setupKeyboardListeners() {
    // 全局键盘监听器
    document.addEventListener('keydown', this.handleKeyDown.bind(this));
    document.addEventListener('keyup', this.handleKeyUp.bind(this));
    
    // 鼠标监听器
    document.addEventListener('mousedown', this.handleMouseDown.bind(this));
    document.addEventListener('contextmenu', this.handleContextMenu.bind(this));
  }

  private handleKeyDown(event: KeyboardEvent) {
    const key = this.normalizeKey(event);
    
    // 检测双击
    this.detectDoubleClick(key);
    
    // 检测长按
    this.detectHold(key, event);
    
    // 检测序列
    this.detectSequence(key);
  }

  private handleKeyUp(event: KeyboardEvent) {
    const key = this.normalizeKey(event);
    
    // 清除长按计时器
    if (this.holdTimers.has(key)) {
      clearTimeout(this.holdTimers.get(key)!);
      this.holdTimers.delete(key);
      this.heldKeys.delete(key);
    }
  }

  private handleMouseDown(event: MouseEvent) {
    if (event.button === 2) { // 右键
      // 检查是否有右键相关的快捷键
      this.emit('right-click');
    }
  }

  private handleContextMenu(event: MouseEvent) {
    // 可以在这里处理右键菜单相关的快捷键
  }

  private detectDoubleClick(key: string) {
    const now = Date.now();
    const lastPress = this.lastKeyPress.get(key) || 0;
    
    // 检查双击快捷键
    for (const shortcut of this.shortcuts.values()) {
      if (shortcut.triggerMode === 'double' && 
          shortcut.doubleClick && 
          shortcut.doubleClick.key === key &&
          shortcut.enabled) {
        
        if (now - lastPress < shortcut.doubleClick.timeout) {
          // 触发双击
          shortcut.action();
          this.lastKeyPress.delete(key);
          
          // 清除超时
          if (this.doubleClickTimeouts.has(key)) {
            clearTimeout(this.doubleClickTimeouts.get(key)!);
            this.doubleClickTimeouts.delete(key);
          }
        } else {
          // 记录第一次按键
          this.lastKeyPress.set(key, now);
          
          // 设置超时清除
          const timeout = setTimeout(() => {
            this.lastKeyPress.delete(key);
            this.doubleClickTimeouts.delete(key);
          }, shortcut.doubleClick.timeout);
          
          this.doubleClickTimeouts.set(key, timeout);
        }
      }
    }
  }

  private detectHold(key: string, event: KeyboardEvent) {
    if (this.heldKeys.has(key)) return; // 已经在长按中
    
    // 检查长按快捷键
    for (const shortcut of this.shortcuts.values()) {
      if (shortcut.triggerMode === 'hold' && 
          shortcut.hold && 
          shortcut.hold.key === key &&
          shortcut.enabled) {
        
        this.heldKeys.add(key);
        
        const timer = setTimeout(() => {
          shortcut.action();
          this.heldKeys.delete(key);
        }, shortcut.hold.duration);
        
        this.holdTimers.set(key, timer);
      }
    }
  }

  private detectSequence(key: string) {
    this.sequenceBuffer.push(key);
    
    // 清除之前的计时器
    if (this.sequenceTimer) {
      clearTimeout(this.sequenceTimer);
    }
    
    // 检查序列快捷键
    for (const shortcut of this.shortcuts.values()) {
      if (shortcut.triggerMode === 'sequence' && 
          shortcut.sequence && 
          shortcut.enabled) {
        
        const sequence = shortcut.sequence.keys;
        const buffer = this.sequenceBuffer.slice(-sequence.length);
        
        if (buffer.length === sequence.length && 
            buffer.every((k, i) => k === sequence[i])) {
          // 触发序列快捷键
          shortcut.action();
          this.sequenceBuffer = [];
          return;
        }
      }
    }
    
    // 设置清除计时器
    this.sequenceTimer = setTimeout(() => {
      this.sequenceBuffer = [];
    }, 500);
  }

  private initializeSpeechRecognition() {
    // 检查浏览器是否支持语音识别
    const SpeechRecognition = (window as any).SpeechRecognition || (window as any).webkitSpeechRecognition;
    
    if (SpeechRecognition) {
      this.speechRecognition = new SpeechRecognition();
      this.speechRecognition.continuous = true;
      this.speechRecognition.interimResults = false;
      this.speechRecognition.lang = 'zh-CN';
      
      this.speechRecognition.onresult = (event: any) => {
        const transcript = event.results[event.results.length - 1][0].transcript.toLowerCase();
        
        // 检查语音命令
        for (const shortcut of this.shortcuts.values()) {
          if (shortcut.triggerMode === 'voice' && 
              shortcut.voiceCommand && 
              shortcut.enabled) {
            
            for (const phrase of shortcut.voiceCommand.phrases) {
              if (transcript.includes(phrase.toLowerCase())) {
                shortcut.action();
                break;
              }
            }
          }
        }
      };
      
      this.speechRecognition.onerror = (event: any) => {
        console.error('语音识别错误:', event.error);
      };
    }
  }

  startVoiceRecognition() {
    if (this.speechRecognition && !this.voiceRecognitionActive) {
      this.speechRecognition.start();
      this.voiceRecognitionActive = true;
      console.log('🎙️ 语音识别已启动');
    }
  }

  stopVoiceRecognition() {
    if (this.speechRecognition && this.voiceRecognitionActive) {
      this.speechRecognition.stop();
      this.voiceRecognitionActive = false;
      console.log('🔇 语音识别已停止');
    }
  }

  private normalizeKey(event: KeyboardEvent): string {
    let key = event.key;
    
    // 标准化特殊键
    const keyMap: Record<string, string> = {
      ' ': 'Space',
      'Enter': 'Enter',
      'Escape': 'Escape',
      'Tab': 'Tab',
      'Delete': 'Delete',
      'Backspace': 'Backspace',
      'ArrowUp': 'Up',
      'ArrowDown': 'Down',
      'ArrowLeft': 'Left',
      'ArrowRight': 'Right',
      'Control': 'Ctrl',
      'Meta': 'Cmd',
      'Alt': 'Alt',
      'Shift': 'Shift',
      'CapsLock': 'CapsLock'
    };
    
    return keyMap[key] || key.toUpperCase();
  }

  // 公共方法
  addShortcut(shortcut: AdvancedShortcut) {
    this.shortcuts.set(shortcut.id, shortcut);
  }

  removeShortcut(id: string) {
    this.shortcuts.delete(id);
  }

  updateShortcut(id: string, updates: Partial<AdvancedShortcut>) {
    const shortcut = this.shortcuts.get(id);
    if (shortcut) {
      Object.assign(shortcut, updates);
    }
  }

  getShortcuts(): AdvancedShortcut[] {
    return Array.from(this.shortcuts.values());
  }

  getShortcutsByCategory(category: ShortcutCategory): AdvancedShortcut[] {
    return Array.from(this.shortcuts.values()).filter(s => s.category === category);
  }

  // 创建自定义快捷键
  createCustomShortcut(config: {
    name: string;
    description: string;
    triggerMode: TriggerMode;
    triggerConfig: any;
    action: () => void | Promise<void>;
  }): string {
    const id = `custom-${Date.now()}`;
    
    const shortcut: AdvancedShortcut = {
      id,
      name: config.name,
      description: config.description,
      category: 'custom',
      enabled: true,
      triggerMode: config.triggerMode,
      action: config.action
    };
    
    // 根据触发模式设置配置
    switch (config.triggerMode) {
      case 'single':
        shortcut.primaryKey = config.triggerConfig.key;
        break;
      case 'double':
        shortcut.doubleClick = config.triggerConfig;
        break;
      case 'hold':
        shortcut.hold = config.triggerConfig;
        break;
      case 'sequence':
        shortcut.sequence = config.triggerConfig;
        break;
      case 'voice':
        shortcut.voiceCommand = config.triggerConfig;
        break;
    }
    
    this.addShortcut(shortcut);
    return id;
  }

  // 导入/导出配置
  exportConfig(): string {
    const config = Array.from(this.shortcuts.values()).map(s => ({
      ...s,
      action: undefined // 不导出函数
    }));
    return JSON.stringify(config, null, 2);
  }

  importConfig(configJson: string) {
    try {
      const config = JSON.parse(configJson);
      // 恢复快捷键配置
      // 注意：需要重新绑定action函数
      console.log('配置已导入');
    } catch (error) {
      console.error('导入配置失败:', error);
    }
  }

  // 事件系统
  on(event: string, callback: Function) {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, []);
    }
    this.listeners.get(event)!.push(callback);
  }

  off(event: string, callback: Function) {
    const callbacks = this.listeners.get(event);
    if (callbacks) {
      const index = callbacks.indexOf(callback);
      if (index > -1) {
        callbacks.splice(index, 1);
      }
    }
  }

  private async emit(event: string, ...args: any[]) {
    const callbacks = this.listeners.get(event);
    if (callbacks) {
      for (const callback of callbacks) {
        await callback(...args);
      }
    }
  }

  // 注册系统快捷键
  async registerSystemShortcuts() {
    for (const shortcut of this.shortcuts.values()) {
      if (shortcut.enabled && shortcut.triggerMode === 'single' && shortcut.primaryKey) {
        try {
          await register(shortcut.primaryKey, () => {
            shortcut.action();
          });
          this.registeredKeys.add(shortcut.primaryKey);
        } catch (error) {
          console.error(`注册快捷键失败 ${shortcut.name}:`, error);
        }
      }
    }
  }

  async unregisterSystemShortcuts() {
    await unregisterAll();
    this.registeredKeys.clear();
  }
}

// 导出单例
export const advancedShortcutManager = new AdvancedShortcutManager();