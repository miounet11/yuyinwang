/**
 * Recording King 快捷键管理器 (重写版本)
 * 提供完整的快捷键功能实现，包括优化后的权限处理和事件系统
 */

import { invoke } from '@tauri-apps/api/tauri';
import { register, unregister, unregisterAll } from '@tauri-apps/api/globalShortcut';
import { appWindow } from '@tauri-apps/api/window';
import { message } from '@tauri-apps/api/dialog';
import { listen } from '@tauri-apps/api/event';

export interface Shortcut {
  id: string;
  name: string;
  description: string;
  key: string;
  action: () => void | Promise<void>;
  category: 'recording' | 'navigation' | 'editing' | 'system';
  enabled: boolean;
}

export class ShortcutManager {
  private shortcuts: Map<string, Shortcut> = new Map();
  private registeredShortcuts: Set<string> = new Set();
  private listeners: Map<string, Function[]> = new Map();
  private safeMode: boolean = true;

  constructor() {
    this.initializeDefaultShortcuts();
    this.loadCustomShortcuts();
    this.setupShortcutEventListener();
  }

  setSafeMode(enabled: boolean) {
    this.safeMode = enabled;
  }

  private async setupShortcutEventListener() {
    try {
      await listen('shortcut_pressed', (event: any) => {
        const data = event.payload;
        this.handleShortcutAction(data.action, data.shortcut);
      });
    } catch (error) {
      console.error('设置快捷键事件监听器失败:', error);
    }
  }

  private async handleShortcutAction(action: string, shortcut: string) {
    try {
      await this.emit(action);
    } catch (error) {
      console.error(`执行快捷键动作失败: ${action}`, error);
    }
  }

  searchShortcuts(query: string): Shortcut[] {
    const lowerQuery = query.toLowerCase();
    return Array.from(this.shortcuts.values()).filter(s => 
      s.name.toLowerCase().includes(lowerQuery) || 
      s.description.toLowerCase().includes(lowerQuery) ||
      s.key.toLowerCase().includes(lowerQuery)
    );
  }

  private initializeDefaultShortcuts() {
    // 默认快捷键列表（简化并优化）
    const defaultShortcuts: Shortcut[] = [
      {
        id: 'toggle-recording',
        name: '开始/停止录音',
        description: '切换录音状态',
        key: 'CommandOrControl+Shift+R',
        action: () => this.emit('toggle-recording'),
        category: 'recording',
        enabled: true
      },
      {
        id: 'quick-transcribe',
        name: '快速转录',
        description: '快速开始转录',
        key: 'CommandOrControl+Shift+Space',
        action: () => this.emit('quick-transcribe'),
        category: 'recording',
        enabled: true
      },
      // 添加更多默认快捷键...
    ];

    defaultShortcuts.forEach(s => this.addShortcut(s));
  }

  private addShortcut(shortcut: Shortcut) {
    this.shortcuts.set(shortcut.id, shortcut);
  }

  public async registerShortcut(shortcutId: string): Promise<boolean> {
    const shortcut = this.shortcuts.get(shortcutId);
    if (!shortcut || !shortcut.enabled) return false;

    if (this.safeMode && shortcut.category === 'system') return false;

    try {
      const success = await invoke('register_global_shortcut', {
        shortcut: shortcut.key,
        action: shortcut.id
      }) as boolean;

      if (success) {
        this.registeredShortcuts.add(shortcut.key);
        return true;
      }
      return false;
    } catch (error) {
      if (error.toString().includes('permission')) {
        await this.showPermissionError(shortcut.name);
      }
      return false;
    }
  }

  public async unregisterShortcut(shortcutId: string): Promise<boolean> {
    const shortcut = this.shortcuts.get(shortcutId);
    if (!shortcut) return false;

    try {
      if (this.registeredShortcuts.has(shortcut.key)) {
        const success = await invoke('unregister_global_shortcut', {
          shortcut: shortcut.key
        }) as boolean;
        
        if (success) {
          this.registeredShortcuts.delete(shortcut.key);
          return true;
        }
      }
      return true;
    } catch (error) {
      console.error(`注销快捷键失败: ${shortcut.name}`, error);
      return false;
    }
  }

  public async registerAllShortcuts(): Promise<void> {
    let successCount = 0;
    for (const [id] of this.shortcuts) {
      if (await this.registerShortcut(id)) successCount++;
    }
    console.log(`已注册 ${successCount} 个快捷键`);
  }

  private async showPermissionError(shortcutName: string): Promise<void> {
    await message(`无法注册快捷键 "${shortcutName}"。请检查辅助功能和输入监控权限。`, { title: '权限问题', type: 'error' });
  }

  private loadCustomShortcuts(): void {
    try {
      const stored = localStorage.getItem('custom_shortcuts');
      if (stored) {
        const custom = JSON.parse(stored);
        Object.entries(custom).forEach(([id, config]: [string, any]) => {
          const shortcut = this.shortcuts.get(id);
          if (shortcut) {
            shortcut.key = config.key;
            shortcut.enabled = config.enabled;
          }
        });
      }
    } catch (error) {
      console.error('加载自定义快捷键失败:', error);
    }
  }

  private saveCustomShortcuts(): void {
    const custom: { [key: string]: { key: string; enabled: boolean } } = {};
    this.shortcuts.forEach((s, id) => {
      custom[id] = { key: s.key, enabled: s.enabled };
    });
    localStorage.setItem('custom_shortcuts', JSON.stringify(custom));
  }

  public resetAllShortcuts(): void {
    this.shortcuts.clear();
    this.initializeDefaultShortcuts();
    this.saveCustomShortcuts();
  }

  private async emit(event: string, ...args: any[]): Promise<void> {
    const callbacks = this.listeners.get(event);
    if (callbacks) {
      for (const cb of callbacks) {
        try {
          await cb(...args);
        } catch (error) {
          console.error(`事件 ${event} 处理失败:`, error);
        }
      }
    }
  }

  public on(event: string, callback: Function) {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, []);
    }
    this.listeners.get(event)!.push(callback);
  }

  public off(event: string, callback: Function) {
    const callbacks = this.listeners.get(event);
    if (callbacks) {
      const index = callbacks.indexOf(callback);
      if (index > -1) callbacks.splice(index, 1);
    }
  }

  public getShortcuts(): Shortcut[] {
    return Array.from(this.shortcuts.values());
  }

  public updateShortcut(id: string, updates: Partial<Shortcut>): boolean {
    const shortcut = this.shortcuts.get(id);
    if (!shortcut) return false;
    
    const updatedShortcut = { ...shortcut, ...updates };
    this.shortcuts.set(id, updatedShortcut);
    
    // 如果快捷键改变了，需要重新注册
    if (updates.key && updates.key !== shortcut.key) {
      this.unregisterShortcut(id);
      this.registerShortcut(id);
    }
    
    this.saveCustomShortcuts();
    return true;
  }
}

// 导出实例
export const shortcutManager = new ShortcutManager();
