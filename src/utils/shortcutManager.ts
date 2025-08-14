/**
 * Recording King 快捷键管理器
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
  private safeMode: boolean = true; // 安全模式：跳过会隐藏/最小化/重载窗口的系统快捷键

  constructor() {
    this.initializeDefaultShortcuts();
    this.loadCustomShortcuts();
    // 不在构造阶段自动注册系统快捷键，避免首次载入触发权限问题。
  }

  // 允许外部切换安全模式
  setSafeMode(enabled: boolean) {
    this.safeMode = enabled;
  }

  // 搜索快捷键
  searchShortcuts(query: string): Shortcut[] {
    const lowerQuery = query.toLowerCase();
    return Array.from(this.shortcuts.values()).filter(shortcut => 
      shortcut.name.toLowerCase().includes(lowerQuery) || 
      shortcut.description.toLowerCase().includes(lowerQuery) ||
      shortcut.key.toLowerCase().includes(lowerQuery)
    );
  }

  // 获取系统快捷键（模拟，实际需后端实现）
  private async getSystemShortcuts(): Promise<string[]> {
    // TODO: 从系统获取已注册全局快捷键
    return ['CommandOrControl+Shift+R']; // 示例
  }

  // 改进冲突检测
  private async detectConflicts(key: string): Promise<boolean> {
    const systemShortcuts = await this.getSystemShortcuts();
    return systemShortcuts.includes(key) || this.isShortcutInUse(key);
  }

  private initializeDefaultShortcuts() {
    // 添加参考飞书的快捷键
    this.addShortcut({
      id: 'search-history',
      name: '搜索历史记录',
      description: '快速搜索转录历史',
      key: 'CommandOrControl+F',
      action: async () => {
        await this.emit('search-history');
        console.log('🔍 搜索历史记录');
      },
      category: 'navigation',
      enabled: true
    });

    this.addShortcut({
      id: 'format-bold',
      name: '加粗文本',
      description: '在编辑中加粗选中文本',
      key: 'CommandOrControl+B',
      action: async () => {
        await this.emit('format-bold');
        console.log('📝 加粗文本');
      },
      category: 'editing',
      enabled: true
    });

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

    // macOS 特殊快捷键
    this.addShortcut({
      id: 'fn-key-recording',
      name: 'Fn键录音',
      description: '使用Fn键进行录音（需要系统设置支持）',
      key: 'Fn', // 注意：这需要特殊处理
      action: async () => {
        await this.emit('toggle-recording');
        console.log('🎤 Fn键录音切换');
      },
      category: 'recording',
      enabled: false // 默认禁用，用户需要手动启用
    });

    // 媒体键支持（作为备选方案）
    this.addShortcut({
      id: 'media-key-recording',
      name: '播放/暂停键录音',
      description: '使用媒体播放/暂停键进行录音',
      key: 'MediaPlayPause',
      action: async () => {
        await this.emit('toggle-recording');
        console.log('⏯️ 媒体键录音切换');
      },
      category: 'recording',
      enabled: false // 默认禁用
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


    // 媒体键录音选项
    this.addShortcut({
      id: 'media-next-recording',
      name: '下一首键录音',
      description: '使用媒体下一首键进行录音',
      key: 'MediaNextTrack',
      action: async () => {
        await this.emit('media-next-recording');
        console.log('🎙️ 媒体键录音');
      },
      category: 'recording',
      enabled: false
    });

    this.addShortcut({
      id: 'media-prev-recording',
      name: '上一首键录音',
      description: '使用媒体上一首键进行录音',
      key: 'MediaPreviousTrack',
      action: async () => {
        await this.emit('media-prev-recording');
        console.log('🎙️ 媒体键录音');
      },
      category: 'recording',
      enabled: false
    });

    this.addShortcut({
      id: 'media-stop-recording',
      name: '媒体停止键录音',
      description: '使用媒体停止键进行录音',
      key: 'MediaStop',
      action: async () => {
        await this.emit('media-stop-recording');
        console.log('🎙️ 媒体停止键录音');
      },
      category: 'recording',
      enabled: false
    });
  }

  private addShortcut(shortcut: Shortcut) {
    this.shortcuts.set(shortcut.id, shortcut);
  }

  public async registerShortcut(shortcutId: string): Promise<boolean> {
    const shortcut = this.shortcuts.get(shortcutId);
    if (!shortcut || !shortcut.enabled) return false;

    try {
      // 安全模式下跳过系统类快捷键，避免给用户造成“闪退”（窗口隐藏/最小化/重载）错觉
      if (this.safeMode && shortcut.category === 'system') {
        console.log(`⏭️ 安全模式：跳过系统快捷键注册 ${shortcut.name}`);
        return false;
      }
      // 若仍存在未映射的 Fn，注册前进行映射
      if (shortcut.key === 'Fn') {
        shortcut.key = this.mapUnsupportedToPreferred(shortcutId);
      }

      // 先注销已存在的快捷键
      if (this.registeredShortcuts.has(shortcut.key)) {
        await unregister(shortcut.key);
        this.registeredShortcuts.delete(shortcut.key);
      }

      // 注册新快捷键
      await register(shortcut.key, () => {
        console.log(`🔑 快捷键触发: ${shortcut.name} (${shortcut.key})`);
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

  public async unregisterShortcut(shortcutId: string): Promise<boolean> {
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

  public async registerAllShortcuts(): Promise<void> {
    console.log('🔧 注册所有快捷键...');
    
    let successCount = 0;
    let failureCount = 0;
    const failedShortcuts: string[] = [];
    
    // 先进行快速权限检查
    const hasBasicPermissions = await this.checkBasicPermissions();
    if (!hasBasicPermissions) {
      console.log('⚠️ 缺少基本权限，建议用户检查权限设置');
      this.emit('suggest-permission-check');
      // 仍然尝试注册，以防权限检查有误
    }
    
    // 注册前，统一将不受支持的键位映射到推荐组合
    for (const [sid, s] of this.shortcuts) {
      if (s.key === 'Fn') {
        s.key = this.mapUnsupportedToPreferred(sid);
      }
    }

    for (const [id, shortcut] of this.shortcuts) {
      if (shortcut.enabled) {
        if (this.safeMode && shortcut.category === 'system') {
          // 跳过系统快捷键
          continue;
        }
        const success = await this.registerShortcut(id);
        if (success) {
          successCount++;
        } else {
          failureCount++;
          failedShortcuts.push(shortcut.name);
        }
      }
    }
    
    console.log(`✅ 已注册 ${successCount} 个快捷键`);
    
    if (failureCount > 0) {
      console.warn(`⚠️ ${failureCount} 个快捷键注册失败:`, failedShortcuts);
      
      // 记录失败的快捷键以供调试
      localStorage.setItem('spokenly_failed_shortcuts', JSON.stringify(failedShortcuts));
      
      // 智能处理失败情况
      await this.handleShortcutFailures(failureCount, failedShortcuts);
    } else {
      // 如果所有快捷键都成功注册，清除之前的失败记录
      localStorage.removeItem('spokenly_failed_shortcuts');
    }
  }

  private async checkBasicPermissions(): Promise<boolean> {
    try {
      // 检查是否能访问系统API
      // 这是一个简单的检查，不会触发权限弹窗
      const hasAccessibility = await this.checkAccessibilityPermission();
      const hasInputMonitoring = await this.checkInputMonitoringPermission();
      
      return hasAccessibility && hasInputMonitoring;
    } catch (error) {
      console.log('权限检查异常:', error);
      return false; // 发生错误时假设没有权限
    }
  }

  private async checkAccessibilityPermission(): Promise<boolean> {
    // 这里应该调用后端API检查辅助功能权限
    // 暂时返回true，避免阻塞
    return true;
  }

  private async checkInputMonitoringPermission(): Promise<boolean> {
    // 这里应该调用后端API检查输入监控权限
    // 暂时返回true，避免阻塞
    return true;
  }

  private async handleShortcutFailures(failureCount: number, failedShortcuts: string[]) {
    const isFirstLaunch = !localStorage.getItem('spokenly_setup_completed');
    
    if (isFirstLaunch) {
      // 首次使用，启动向导
      console.log('🚀 启动首次设置向导');
      this.emit('show-first-launch-wizard');
    } else {
      // 检查失败模式
      const totalShortcuts = this.shortcuts.size;
      const failureRate = failureCount / totalShortcuts;
      
      if (failureRate > 0.5) {
        // 超过50%失败，可能是权限问题
        console.log('💡 大量快捷键失败，建议检查权限设置');
        this.emit('suggest-permission-check');
      } else {
        // 少量失败，可能是快捷键冲突
        console.log('⚠️ 部分快捷键失败，可能存在冲突');
        await this.detectAndResolveConflicts(failedShortcuts);
        this.emit('shortcut-conflicts-detected', failedShortcuts);
      }
    }
  }

  private async detectAndResolveConflicts(failedShortcuts: string[]) {
    console.log('🔍 检测快捷键冲突...');
    
    // 获取系统快捷键列表（如果可能）
    // const systemShortcuts = await this.getSystemShortcuts();
    
    // 建议替代快捷键
    const suggestions = this.generateAlternativeShortcuts(failedShortcuts);
    
    console.log('💡 建议的替代快捷键:', suggestions);
    
    // 将建议保存到localStorage，供用户界面显示
    localStorage.setItem('spokenly_shortcut_suggestions', JSON.stringify(suggestions));
  }

  private generateAlternativeShortcuts(failedShortcuts: string[]): {[key: string]: string} {
    const suggestions: {[key: string]: string} = {};
    
    // 为失败的快捷键生成替代方案
    for (const shortcutName of failedShortcuts) {
      const shortcut = Array.from(this.shortcuts.values()).find(s => s.name === shortcutName);
      if (shortcut) {
        suggestions[shortcutName] = this.generateAlternativeKey(shortcut.key);
      }
    }
    
    return suggestions;
  }

  private generateAlternativeKey(originalKey: string): string {
    // 简单的替代方案生成逻辑
    if (originalKey === 'Fn') {
      // 对于 Fn，直接建议 CommandOrControl+Shift+R
      return 'CommandOrControl+Shift+R';
    }
    if (originalKey.includes('CommandOrControl+Shift+')) {
      // 尝试使用Alt替代
      return originalKey.replace('CommandOrControl+Shift+', 'CommandOrControl+Alt+');
    } else if (originalKey.includes('CommandOrControl+')) {
      // 尝试添加Shift
      return originalKey.replace('CommandOrControl+', 'CommandOrControl+Shift+');
    } else {
      // 添加修饰键
      return `CommandOrControl+${originalKey}`;
    }
  }

  // 将不受支持的键位映射为首选组合（避免冲突）
  private mapUnsupportedToPreferred(excludeId: string): string {
    const candidates = [
      'CommandOrControl+Shift+R',
      'CommandOrControl+Shift+Space',
      'CommandOrControl+Alt+R'
    ];
    for (const candidate of candidates) {
      if (!this.isShortcutInUse(candidate, excludeId)) {
        return candidate;
      }
    }
    // 兜底返回第一候选
    return 'CommandOrControl+Shift+R';
  }

  // 提供快捷键状态检查方法
  getShortcutStatus(): {registered: number, failed: number, total: number} {
    const total = this.shortcuts.size;
    const registered = this.registeredShortcuts.size;
    const failed = total - registered;
    
    return { registered, failed, total };
  }

  // 获取失败的快捷键列表
  getFailedShortcuts(): string[] {
    const stored = localStorage.getItem('spokenly_failed_shortcuts');
    return stored ? JSON.parse(stored) : [];
  }

  // 获取快捷键建议
  getShortcutSuggestions(): {[key: string]: string} {
    const stored = localStorage.getItem('spokenly_shortcut_suggestions');
    return stored ? JSON.parse(stored) : {};
  }

  public async unregisterAllShortcuts(): Promise<void> {
    await unregisterAll();
    this.registeredShortcuts.clear();
    console.log('✅ 已注销所有快捷键');
  }

  updateShortcut(shortcutId: string, newKey: string): boolean {
    const shortcut = this.shortcuts.get(shortcutId);
    if (!shortcut) return false;

    // 将 Fn 映射为推荐组合
    if (newKey === 'Fn') {
      newKey = this.mapUnsupportedToPreferred(shortcutId);
    }

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

    // 保存配置
    this.saveCustomShortcuts();

    return true;
  }

  toggleShortcut(shortcutId: string): boolean {
    const shortcut = this.shortcuts.get(shortcutId);
    if (!shortcut) return false;

    shortcut.enabled = !shortcut.enabled;

    // 若启用且为 Fn，先映射
    if (shortcut.enabled && shortcut.key === 'Fn') {
      shortcut.key = this.mapUnsupportedToPreferred(shortcutId);
      this.saveCustomShortcuts();
    }

    if (shortcut.enabled) {
      this.registerShortcut(shortcutId);
    } else {
      this.unregisterShortcut(shortcutId);
    }

    // 保存配置
    this.saveCustomShortcuts();

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
    
    // macOS 使用 metaKey (⌘)，Windows/Linux 使用 ctrlKey
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
    if (event.key && !['Control', 'Shift', 'Alt', 'Meta', 'Command'].includes(event.key)) {
      // 转换特殊键
      let key = event.key;
      
      // 特殊键映射
      const keyMap: { [key: string]: string } = {
        ' ': 'Space',
        'ArrowUp': 'Up',
        'ArrowDown': 'Down',
        'ArrowLeft': 'Left',
        'ArrowRight': 'Right',
        'Enter': 'Return',
        'Backspace': 'Backspace',
        'Delete': 'Delete',
        'Escape': 'Escape',
        'Tab': 'Tab',
        ',': 'Comma',
        '.': 'Period',
        '/': 'Slash',
        ';': 'Semicolon',
        "'": 'Quote',
        '[': 'BracketLeft',
        ']': 'BracketRight',
        '\\': 'Backslash',
        '-': 'Minus',
        '=': 'Equal',
        '`': 'Backquote'
      };
      
      // 使用映射或转换为大写
      if (keyMap[key]) {
        key = keyMap[key];
      } else if (key.length === 1) {
        key = key.toUpperCase();
      }
      
      keys.push(key);
    }
    
    return keys.join('+');
  }

  // 验证快捷键是否有效
  isValidShortcut(key: string): boolean {
    // 检查是否包含至少一个修饰键和一个主键，或者是特殊单键
    const parts = key.split('+');
    const hasModifier = parts.some(p => 
      ['CommandOrControl', 'Shift', 'Alt', 'Ctrl', 'Meta'].includes(p)
    );
    const hasMainKey = parts.some(p => 
      !['CommandOrControl', 'Shift', 'Alt', 'Ctrl', 'Meta'].includes(p)
    );
    
    // 允许特殊单键（媒体键、F键等）
    const specialSingleKeys = [
      'MediaPlayPause', 'MediaNextTrack', 'MediaPreviousTrack', 'MediaStop',
      'F13', 'F14', 'F15', 'F16', 'F17', 'F18', 'F19', 'F20', 'CapsLock', 'Fn'
    ];
    const isSingleSpecialKey = parts.length === 1 && specialSingleKeys.includes(parts[0]);
    
    return (hasModifier && hasMainKey && parts.length >= 2) || isSingleSpecialKey;
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
      `请确保已授予 Recording King 以下权限：\n` +
      `• 辅助功能（Accessibility）\n` +
      `• 输入监控（Input Monitoring）\n\n` +
      `您可以在应用中点击 🔐 图标打开权限设置。`,
      {
        title: '快捷键权限需求',
        type: 'error'
      }
    );
  }

  // 保存自定义快捷键到本地存储
  private saveCustomShortcuts(): void {
    try {
      const customShortcuts: { [key: string]: { key: string; enabled: boolean } } = {};
      
      this.shortcuts.forEach((shortcut, id) => {
        customShortcuts[id] = {
          key: shortcut.key,
          enabled: shortcut.enabled
        };
      });
      
      localStorage.setItem('custom_shortcuts', JSON.stringify(customShortcuts));
      console.log('✅ 快捷键配置已保存');
    } catch (error) {
      console.error('❌ 保存快捷键配置失败:', error);
    }
  }

  // 从本地存储加载自定义快捷键
  private loadCustomShortcuts(): void {
    try {
      const stored = localStorage.getItem('custom_shortcuts');
      if (stored) {
        const customShortcuts = JSON.parse(stored);
        
        Object.entries(customShortcuts).forEach(([id, config]: [string, any]) => {
          const shortcut = this.shortcuts.get(id);
          if (shortcut) {
            shortcut.key = config.key;
            shortcut.enabled = config.enabled;
          }
        });
        
        console.log('✅ 已加载自定义快捷键配置');
      }
    } catch (error) {
      console.error('❌ 加载快捷键配置失败:', error);
    }
  }

  // 重置所有快捷键到默认值
  resetAllShortcuts(): void {
    this.shortcuts.clear();
    this.initializeDefaultShortcuts();
    localStorage.removeItem('custom_shortcuts');
    console.log('✅ 所有快捷键已重置为默认值');
  }
}

// 导出单例实例
export const shortcutManager = new ShortcutManager();
