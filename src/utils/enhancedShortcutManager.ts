import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';

export interface ShortcutConfig {
  key: string;
  action: ShortcutAction;
  description: string;
  enabled: boolean;
  global?: boolean;
}

export type ShortcutAction = 
  | 'toggle_recording'
  | 'start_recording'
  | 'stop_recording'
  | 'show_app'
  | 'hide_app'
  | 'toggle_visibility'
  | 'quick_transcribe'
  | 'show_history'
  | 'toggle_text_injection';

export interface ShortcutEvent {
  key: string;
  action: ShortcutAction;
  timestamp: number;
  source: 'global' | 'local' | 'system';
}

export class EnhancedShortcutManager {
  private shortcuts: Map<string, ShortcutConfig> = new Map();
  private listeners: Map<ShortcutAction, ((event: ShortcutEvent) => void)[]> = new Map();
  private registeredGlobal: Set<string> = new Set();

  constructor() {
    console.log('🏗️ 构造 enhancedShortcutManager...');
    this.initializeDefaults();
    // 注意：setupEventListeners 需要手动调用，因为它是异步的
    // 注意：不再自动注册快捷键，因为后端已经处理了
    console.log('✅ enhancedShortcutManager 构造完成');
  }

  private initializeDefaults() {
    const defaultShortcuts: ShortcutConfig[] = [
      {
        key: 'CommandOrControl+Shift+R',
        action: 'toggle_recording',
        description: '切换录音状态',
        enabled: true,
        global: true
      },
      {
        key: 'CommandOrControl+Shift+S',
        action: 'start_recording',
        description: '开始录音',
        enabled: true,
        global: true
      },
      {
        key: 'CommandOrControl+Shift+E',
        action: 'stop_recording',
        description: '停止录音',
        enabled: true,
        global: true
      },
      {
        key: 'CommandOrControl+Shift+H',
        action: 'show_history',
        description: '显示历史记录',
        enabled: true,
        global: true
      },
      {
        key: 'CommandOrControl+Shift+T',
        action: 'quick_transcribe',
        description: '快速转录',
        enabled: true,
        global: true
      },
      {
        key: 'CommandOrControl+Shift+I',
        action: 'toggle_text_injection',
        description: '切换文本注入',
        enabled: true,
        global: true
      },
      {
        key: 'CommandOrControl+Shift+W',
        action: 'toggle_visibility',
        description: '切换窗口显示',
        enabled: true,
        global: true
      }
    ];

    defaultShortcuts.forEach(shortcut => {
      this.shortcuts.set(shortcut.key, shortcut);
    });
  }

  public async setupEventListeners() {
    try {
      console.log('🔧 设置 enhancedShortcutManager 事件监听器...');
      
      // 监听来自后端的快捷键事件
      await listen('shortcut_pressed', async (event: any) => {
        console.log('🔥 收到 shortcut_pressed 事件:', event);
        const { shortcut, action } = event.payload;
        console.log('🎯 解析快捷键事件:', { shortcut, action });
        
        // 向后端确认事件接收
        try {
          await invoke('confirm_event_received', { 
            eventType: 'shortcut_pressed', 
            details: `${shortcut} -> ${action}` 
          });
        } catch (error) {
          console.error('❌ 确认事件接收失败:', error);
        }
        
        this.handleShortcutEvent({
          key: shortcut,
          action: action as ShortcutAction,
          timestamp: Date.now(),
          source: 'global'
        });
      });

      // 监听系统托盘快捷键
      await listen('tray_toggle_recording', () => {
        this.handleShortcutEvent({
          key: 'tray',
          action: 'toggle_recording',
          timestamp: Date.now(),
          source: 'system'
        });
      });

      console.log('✅ 增强快捷键管理器事件监听已设置');
    } catch (error) {
      console.error('❌ 设置快捷键事件监听失败:', error);
    }
  }

  // 初始化全局快捷键
  private async initializeGlobalShortcuts() {
    try {
      console.log('🚀 开始注册默认全局快捷键...');
      
      for (const shortcut of this.shortcuts.values()) {
        if (shortcut.enabled && shortcut.global) {
          await this.registerGlobalShortcut(shortcut.key, shortcut.action);
        }
      }
      
      console.log('✅ 默认全局快捷键注册完成');
    } catch (error) {
      console.error('❌ 注册默认全局快捷键失败:', error);
    }
  }

  private handleShortcutEvent(event: ShortcutEvent) {
    console.log('🔥 快捷键事件:', event);
    console.log('📋 当前监听器数量:', this.listeners.get(event.action)?.length || 0);
    
    const listeners = this.listeners.get(event.action) || [];
    listeners.forEach(listener => {
      try {
        listener(event);
      } catch (error) {
        console.error(`快捷键监听器执行失败 (${event.action}):`, error);
      }
    });
  }

  // 注册快捷键监听器
  public on(action: ShortcutAction, listener: (event: ShortcutEvent) => void): () => void {
    if (!this.listeners.has(action)) {
      this.listeners.set(action, []);
    }
    
    this.listeners.get(action)!.push(listener);
    
    // 返回取消注册函数
    return () => {
      const listeners = this.listeners.get(action);
      if (listeners) {
        const index = listeners.indexOf(listener);
        if (index > -1) {
          listeners.splice(index, 1);
        }
      }
    };
  }

  // 移除快捷键监听器
  public off(action: ShortcutAction, listener?: (event: ShortcutEvent) => void) {
    if (!listener) {
      this.listeners.delete(action);
      return;
    }

    const listeners = this.listeners.get(action);
    if (listeners) {
      const index = listeners.indexOf(listener);
      if (index > -1) {
        listeners.splice(index, 1);
      }
    }
  }

  // 更新快捷键配置
  public async updateShortcut(oldKey: string, newConfig: ShortcutConfig): Promise<boolean> {
    try {
      // 取消注册旧快捷键
      if (this.shortcuts.has(oldKey)) {
        await this.unregisterShortcut(oldKey);
      }

      // 注册新快捷键
      this.shortcuts.set(newConfig.key, newConfig);
      
      if (newConfig.enabled && newConfig.global) {
        await this.registerGlobalShortcut(newConfig.key, newConfig.action);
      }

      console.log(`✅ 快捷键已更新: ${oldKey} -> ${newConfig.key}`);
      return true;
    } catch (error) {
      console.error('❌ 更新快捷键失败:', error);
      return false;
    }
  }

  // 注册全局快捷键
  private async registerGlobalShortcut(key: string, action: ShortcutAction): Promise<boolean> {
    try {
      // 使用 Tauri 的后端快捷键注册
      await invoke('register_global_shortcut', { 
        shortcut: key, 
        action: action 
      });
      
      this.registeredGlobal.add(key);
      console.log(`✅ 全局快捷键已注册: ${key} -> ${action}`);
      return true;
    } catch (error) {
      console.error(`❌ 注册全局快捷键失败 (${key}):`, error);
      return false;
    }
  }

  // 取消注册快捷键
  private async unregisterShortcut(key: string): Promise<boolean> {
    try {
      if (this.registeredGlobal.has(key)) {
        await invoke('unregister_global_shortcut', { shortcut: key });
        this.registeredGlobal.delete(key);
      }
      
      this.shortcuts.delete(key);
      console.log(`✅ 快捷键已取消注册: ${key}`);
      return true;
    } catch (error) {
      console.error(`❌ 取消注册快捷键失败 (${key}):`, error);
      return false;
    }
  }

  // 启用/禁用快捷键
  public async toggleShortcut(key: string, enabled: boolean): Promise<boolean> {
    const shortcut = this.shortcuts.get(key);
    if (!shortcut) {
      console.error(`快捷键不存在: ${key}`);
      return false;
    }

    shortcut.enabled = enabled;

    if (enabled && shortcut.global) {
      return await this.registerGlobalShortcut(key, shortcut.action);
    } else {
      return await this.unregisterShortcut(key);
    }
  }

  // 获取所有快捷键配置
  public getShortcuts(): ShortcutConfig[] {
    return Array.from(this.shortcuts.values());
  }

  // 获取特定快捷键配置
  public getShortcut(key: string): ShortcutConfig | undefined {
    return this.shortcuts.get(key);
  }

  // 查找动作对应的快捷键
  public getShortcutForAction(action: ShortcutAction): ShortcutConfig | undefined {
    for (const shortcut of this.shortcuts.values()) {
      if (shortcut.action === action && shortcut.enabled) {
        return shortcut;
      }
    }
    return undefined;
  }

  // 检查快捷键是否已存在
  public isKeyRegistered(key: string): boolean {
    return this.shortcuts.has(key);
  }

  // 验证快捷键格式
  public validateShortcutKey(key: string): boolean {
    // 基本格式验证
    const validModifiers = ['CommandOrControl', 'Command', 'Control', 'Alt', 'Shift', 'Meta'];
    const parts = key.split('+');
    
    if (parts.length < 2) {
      return false;
    }

    const modifiers = parts.slice(0, -1);
    const mainKey = parts[parts.length - 1];

    // 检查修饰键
    for (const modifier of modifiers) {
      if (!validModifiers.includes(modifier)) {
        return false;
      }
    }

    // 检查主键
    if (!mainKey || mainKey.length === 0) {
      return false;
    }

    return true;
  }

  // 模拟触发快捷键事件（用于测试）
  public simulateShortcut(key: string) {
    const shortcut = this.shortcuts.get(key);
    if (shortcut && shortcut.enabled) {
      this.handleShortcutEvent({
        key,
        action: shortcut.action,
        timestamp: Date.now(),
        source: 'local'
      });
    }
  }

  // 获取快捷键统计信息
  public getStats() {
    const total = this.shortcuts.size;
    const enabled = Array.from(this.shortcuts.values()).filter(s => s.enabled).length;
    const global = Array.from(this.shortcuts.values()).filter(s => s.global && s.enabled).length;
    
    return {
      total,
      enabled,
      disabled: total - enabled,
      global,
      local: enabled - global
    };
  }

  // 重置为默认配置
  public async resetToDefaults(): Promise<boolean> {
    try {
      // 清除所有现有快捷键
      for (const key of this.shortcuts.keys()) {
        await this.unregisterShortcut(key);
      }

      // 重新初始化默认配置
      this.shortcuts.clear();
      this.initializeDefaults();

      // 注册启用的全局快捷键
      for (const shortcut of this.shortcuts.values()) {
        if (shortcut.enabled && shortcut.global) {
          await this.registerGlobalShortcut(shortcut.key, shortcut.action);
        }
      }

      console.log('✅ 快捷键配置已重置为默认值');
      return true;
    } catch (error) {
      console.error('❌ 重置快捷键配置失败:', error);
      return false;
    }
  }

  // 清理资源
  public async cleanup() {
    try {
      // 取消注册所有全局快捷键
      for (const key of this.registeredGlobal) {
        await invoke('unregister_global_shortcut', { shortcut: key });
      }
      
      this.registeredGlobal.clear();
      this.listeners.clear();
      console.log('✅ 快捷键管理器已清理');
    } catch (error) {
      console.error('❌ 清理快捷键管理器失败:', error);
    }
  }
}

// 创建全局实例
export const enhancedShortcutManager = new EnhancedShortcutManager();