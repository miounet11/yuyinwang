/**
 * Spokenly Clone 快捷键管理器
 * 提供完整的快捷键功能实现
 */

// import { invoke } from '@tauri-apps/api/tauri';
import { register, unregister, unregisterAll } from '@tauri-apps/api/globalShortcut';
import { appWindow } from '@tauri-apps/api/window';
import { ask, message } from '@tauri-apps/api/dialog';

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

  constructor() {
    this.initializeDefaultShortcuts();
  }

  private initializeDefaultShortcuts() {
    // 录音相关快捷键
    this.addShortcut({
      id: 'toggle-recording',
      name: '开始/停止录音',
      description: '切换录音状态',
      key: 'CommandOrControl+Shift+R',
      action: async () => {
        await this.emit('toggle-recording');
        console.log('🎤 切换录音状态');
      },
      category: 'recording',
      enabled: true
    });

    this.addShortcut({
      id: 'quick-transcribe',
      name: '快速转录',
      description: '快速开始转录并自动停止',
      key: 'CommandOrControl+Shift+Space',
      action: async () => {
        await this.emit('quick-transcribe');
        console.log('⚡ 快速转录');
      },
      category: 'recording',
      enabled: true
    });

    this.addShortcut({
      id: 'pause-recording',
      name: '暂停录音',
      description: '暂停当前录音',
      key: 'CommandOrControl+Shift+P',
      action: async () => {
        await this.emit('pause-recording');
        console.log('⏸️ 暂停录音');
      },
      category: 'recording',
      enabled: true
    });

    // 导航快捷键
    this.addShortcut({
      id: 'open-ai-assistant',
      name: '打开AI助手',
      description: '快速打开AI助手对话框',
      key: 'CommandOrControl+Shift+A',
      action: async () => {
        await this.emit('open-ai-assistant');
        console.log('🤖 打开AI助手');
      },
      category: 'navigation',
      enabled: true
    });

    this.addShortcut({
      id: 'switch-to-history',
      name: '切换到历史记录',
      description: '快速切换到历史记录页面',
      key: 'CommandOrControl+H',
      action: async () => {
        await this.emit('switch-to-history');
        console.log('📋 切换到历史记录');
      },
      category: 'navigation',
      enabled: true
    });

    this.addShortcut({
      id: 'switch-to-models',
      name: '切换到模型管理',
      description: '快速切换到模型管理页面',
      key: 'CommandOrControl+M',
      action: async () => {
        await this.emit('switch-to-models');
        console.log('🎯 切换到模型管理');
      },
      category: 'navigation',
      enabled: true
    });

    this.addShortcut({
      id: 'switch-to-settings',
      name: '打开设置',
      description: '快速打开设置页面',
      key: 'CommandOrControl+Comma',
      action: async () => {
        await this.emit('switch-to-settings');
        console.log('⚙️ 打开设置');
      },
      category: 'navigation',
      enabled: true
    });

    // 编辑快捷键
    this.addShortcut({
      id: 'copy-transcription',
      name: '复制转录文本',
      description: '复制最新的转录文本到剪贴板',
      key: 'CommandOrControl+Shift+C',
      action: async () => {
        await this.emit('copy-transcription');
        console.log('📋 复制转录文本');
      },
      category: 'editing',
      enabled: true
    });

    this.addShortcut({
      id: 'export-transcription',
      name: '导出转录',
      description: '快速导出当前转录',
      key: 'CommandOrControl+Shift+E',
      action: async () => {
        await this.emit('export-transcription');
        console.log('💾 导出转录');
      },
      category: 'editing',
      enabled: true
    });

    this.addShortcut({
      id: 'delete-selected',
      name: '删除选中项',
      description: '删除选中的历史记录',
      key: 'CommandOrControl+Delete',
      action: async () => {
        await this.emit('delete-selected');
        console.log('🗑️ 删除选中项');
      },
      category: 'editing',
      enabled: true
    });

    // 系统快捷键
    this.addShortcut({
      id: 'toggle-window',
      name: '显示/隐藏窗口',
      description: '切换应用窗口显示状态',
      key: 'CommandOrControl+Shift+S',
      action: async () => {
        const isVisible = await appWindow.isVisible();
        if (isVisible) {
          await appWindow.hide();
        } else {
          await appWindow.show();
          await appWindow.setFocus();
        }
        console.log('🪟 切换窗口显示');
      },
      category: 'system',
      enabled: true
    });

    this.addShortcut({
      id: 'minimize-window',
      name: '最小化窗口',
      description: '最小化应用窗口',
      key: 'CommandOrControl+Shift+M',
      action: async () => {
        await appWindow.minimize();
        console.log('📥 最小化窗口');
      },
      category: 'system',
      enabled: true
    });

    this.addShortcut({
      id: 'reload-app',
      name: '重新加载应用',
      description: '重新加载应用界面',
      key: 'CommandOrControl+R',
      action: async () => {
        window.location.reload();
        console.log('🔄 重新加载应用');
      },
      category: 'system',
      enabled: true
    });

    // 特殊功能键
    this.addShortcut({
      id: 'fn-key-recording',
      name: 'Fn键录音',
      description: '使用Fn键进行录音（需要系统设置）',
      key: 'Fn',
      action: async () => {
        await this.emit('fn-key-recording');
        console.log('🎙️ Fn键录音');
      },
      category: 'recording',
      enabled: false // 默认禁用，需要用户手动启用
    });

    this.addShortcut({
      id: 'double-tap-recording',
      name: '双击录音',
      description: '双击Option键开始录音',
      key: 'Alt+Alt', // 双击Alt/Option键
      action: async () => {
        await this.emit('double-tap-recording');
        console.log('🎤 双击录音');
      },
      category: 'recording',
      enabled: false
    });
  }

  private addShortcut(shortcut: Shortcut) {
    this.shortcuts.set(shortcut.id, shortcut);
  }

  async registerShortcut(shortcutId: string): Promise<boolean> {
    const shortcut = this.shortcuts.get(shortcutId);
    if (!shortcut || !shortcut.enabled) return false;

    try {
      // 先注销已存在的快捷键
      if (this.registeredShortcuts.has(shortcut.key)) {
        await unregister(shortcut.key);
        this.registeredShortcuts.delete(shortcut.key);
      }

      // 注册新快捷键
      await register(shortcut.key, () => {
        shortcut.action();
      });

      this.registeredShortcuts.add(shortcut.key);
      console.log(`✅ 已注册快捷键: ${shortcut.name} (${shortcut.key})`);
      return true;
    } catch (error) {
      console.error(`❌ 注册快捷键失败: ${shortcut.name}`, error);
      
      // 检查是否是权限问题
      if (error.toString().includes('permission') || error.toString().includes('accessibility')) {
        await this.showPermissionError(shortcut.name);
      }
      
      return false;
    }
  }

  async unregisterShortcut(shortcutId: string): Promise<boolean> {
    const shortcut = this.shortcuts.get(shortcutId);
    if (!shortcut) return false;

    try {
      if (this.registeredShortcuts.has(shortcut.key)) {
        await unregister(shortcut.key);
        this.registeredShortcuts.delete(shortcut.key);
        console.log(`✅ 已注销快捷键: ${shortcut.name}`);
      }
      return true;
    } catch (error) {
      console.error(`❌ 注销快捷键失败: ${shortcut.name}`, error);
      return false;
    }
  }

  async registerAllShortcuts(): Promise<void> {
    console.log('🔧 注册所有快捷键...');
    
    let successCount = 0;
    let failureCount = 0;
    
    for (const [id, shortcut] of this.shortcuts) {
      if (shortcut.enabled) {
        const success = await this.registerShortcut(id);
        if (success) {
          successCount++;
        } else {
          failureCount++;
        }
      }
    }
    
    console.log(`✅ 已注册 ${successCount} 个快捷键`);
    
    if (failureCount > 0) {
      console.warn(`⚠️ ${failureCount} 个快捷键注册失败`);
      
      // 优雅地触发首次启动向导，而不是显示错误弹窗
      const isFirstLaunch = !localStorage.getItem('spokenly_setup_completed');
      
      if (isFirstLaunch) {
        // 首次使用，启动向导
        console.log('🚀 启动首次设置向导');
        this.emit('show-first-launch-wizard');
      } else {
        // 非首次使用，温和提醒
        console.log('💡 建议检查权限设置');
        this.emit('suggest-permission-check');
      }
    }
  }

  async unregisterAllShortcuts(): Promise<void> {
    await unregisterAll();
    this.registeredShortcuts.clear();
    console.log('✅ 已注销所有快捷键');
  }

  updateShortcut(shortcutId: string, newKey: string): boolean {
    const shortcut = this.shortcuts.get(shortcutId);
    if (!shortcut) return false;

    // 先注销旧快捷键
    if (this.registeredShortcuts.has(shortcut.key)) {
      unregister(shortcut.key).then(() => {
        this.registeredShortcuts.delete(shortcut.key);
      });
    }

    // 更新快捷键
    shortcut.key = newKey;

    // 如果启用，注册新快捷键
    if (shortcut.enabled) {
      this.registerShortcut(shortcutId);
    }

    return true;
  }

  toggleShortcut(shortcutId: string): boolean {
    const shortcut = this.shortcuts.get(shortcutId);
    if (!shortcut) return false;

    shortcut.enabled = !shortcut.enabled;

    if (shortcut.enabled) {
      this.registerShortcut(shortcutId);
    } else {
      this.unregisterShortcut(shortcutId);
    }

    return shortcut.enabled;
  }

  getShortcuts(): Shortcut[] {
    return Array.from(this.shortcuts.values());
  }

  getShortcutsByCategory(category: Shortcut['category']): Shortcut[] {
    return Array.from(this.shortcuts.values()).filter(s => s.category === category);
  }

  getShortcut(shortcutId: string): Shortcut | undefined {
    return this.shortcuts.get(shortcutId);
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

  // 检测按键组合
  detectKeyCombo(event: KeyboardEvent): string {
    const keys: string[] = [];
    
    if (event.metaKey || event.ctrlKey) {
      keys.push('CommandOrControl');
    }
    if (event.shiftKey) {
      keys.push('Shift');
    }
    if (event.altKey) {
      keys.push('Alt');
    }
    
    // 添加主键
    if (event.key && !['Control', 'Shift', 'Alt', 'Meta'].includes(event.key)) {
      // 转换特殊键
      let key = event.key;
      if (key === ' ') key = 'Space';
      if (key === 'ArrowUp') key = 'Up';
      if (key === 'ArrowDown') key = 'Down';
      if (key === 'ArrowLeft') key = 'Left';
      if (key === 'ArrowRight') key = 'Right';
      if (key.length === 1) key = key.toUpperCase();
      
      keys.push(key);
    }
    
    return keys.join('+');
  }

  // 验证快捷键是否有效
  isValidShortcut(key: string): boolean {
    // 检查是否包含至少一个修饰键和一个主键
    const parts = key.split('+');
    const hasModifier = parts.some(p => 
      ['CommandOrControl', 'Shift', 'Alt', 'Ctrl', 'Meta'].includes(p)
    );
    const hasMainKey = parts.some(p => 
      !['CommandOrControl', 'Shift', 'Alt', 'Ctrl', 'Meta'].includes(p)
    );
    
    return hasModifier && hasMainKey && parts.length >= 2;
  }

  // 检查快捷键是否已被使用
  isShortcutInUse(key: string, excludeId?: string): boolean {
    for (const [id, shortcut] of this.shortcuts) {
      if (id !== excludeId && shortcut.key === key) {
        return true;
      }
    }
    return false;
  }
  
  // 显示权限错误对话框
  private async showPermissionError(shortcutName: string): Promise<void> {
    await message(
      `无法注册快捷键 "${shortcutName}"。\n\n` +
      `请确保已授予 Spokenly 以下权限：\n` +
      `• 辅助功能（Accessibility）\n` +
      `• 输入监控（Input Monitoring）\n\n` +
      `您可以在应用中点击 🔐 图标打开权限设置。`,
      {
        title: '快捷键权限需求',
        type: 'error'
      }
    );
  }
}

// 导出单例实例
export const shortcutManager = new ShortcutManager();