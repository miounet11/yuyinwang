/**
 * Recording King 权限管理器
 * 处理所有系统权限请求和状态管理
 */

import { invoke } from '@tauri-apps/api/tauri';
import { open as openUrl } from '@tauri-apps/api/shell';
import { ask, message } from '@tauri-apps/api/dialog';
import { platform } from '@tauri-apps/api/os';

export interface Permission {
  id: string;
  name: string;
  description: string;
  status: 'granted' | 'denied' | 'not-determined';
  required: boolean;
  category: 'system' | 'audio' | 'file' | 'notification' | 'screen';
  icon: string;
}

export interface PermissionCheckResult {
  permission: string;
  status: 'granted' | 'denied' | 'not-determined';
  message?: string;
}

export class PermissionManager {
  private permissions: Map<string, Permission> = new Map();
  private listeners: Map<string, Function[]> = new Map();
  private currentPlatform: string = 'unknown';

  constructor() {
    this.initializePlatform();
    this.initializePermissions();
  }

  private async initializePlatform() {
    try {
      this.currentPlatform = await platform();
    } catch (error) {
      console.error('Failed to detect platform:', error);
      this.currentPlatform = 'unknown';
    }
  }

  private isWindows(): boolean {
    return this.currentPlatform === 'win32';
  }

  private isMacOS(): boolean {
    return this.currentPlatform === 'darwin';
  }

  private initializePermissions() {
    // 跨平台权限：麦克风访问
    this.addPermission({
      id: 'microphone',
      name: '麦克风',
      description: '允许 Recording King 录制音频并进行语音转录',
      status: 'not-determined',
      required: true,
      category: 'audio',
      icon: '🎤'
    });

    // 平台特定权限
    if (this.isMacOS()) {
      // macOS 辅助功能权限（用于全局快捷键）
      this.addPermission({
        id: 'accessibility',
        name: '辅助功能',
        description: '允许 Recording King 使用全局快捷键和系统集成功能',
        status: 'not-determined',
        required: true,
        category: 'system',
        icon: '♿️'
      });

      // macOS 输入监控权限
      this.addPermission({
        id: 'input-monitoring',
        name: '输入监控',
        description: '允许 Recording King 监听键盘快捷键',
        status: 'not-determined',
        required: true,
        category: 'system',
        icon: '⌨️'
      });
    } else if (this.isWindows()) {
      // Windows UAC权限（用于全局快捷键）
      this.addPermission({
        id: 'uac-bypass',
        name: '系统访问',
        description: '允许 Recording King 注册全局快捷键',
        status: 'not-determined',
        required: true,
        category: 'system',
        icon: '🛡️'
      });
    }

    // 文件系统访问权限
    this.addPermission({
      id: 'file-system',
      name: '文件访问',
      description: '允许 Recording King 读取和保存转录文件',
      status: 'not-determined',
      required: true,
      category: 'file',
      icon: '📁'
    });

    // 通知权限
    this.addPermission({
      id: 'notifications',
      name: '通知',
      description: '允许 Recording King 发送转录完成通知',
      status: 'not-determined',
      required: false,
      category: 'notification',
      icon: '🔔'
    });

    // 屏幕录制权限（可选，用于高级功能）
    this.addPermission({
      id: 'screen-recording',
      name: '屏幕录制',
      description: '允许 Recording King 录制屏幕内容（可选功能）',
      status: 'not-determined',
      required: false,
      category: 'screen',
      icon: '🖥️'
    });

    // 自动化权限（用于控制其他应用）
    this.addPermission({
      id: 'automation',
      name: '自动化',
      description: '允许 Recording King 与其他应用程序交互',
      status: 'not-determined',
      required: false,
      category: 'system',
      icon: '🤖'
    });

    // 输入监控权限
    this.addPermission({
      id: 'input-monitoring',
      name: '输入监控',
      description: '允许 Recording King 监听键盘快捷键',
      status: 'not-determined',
      required: true,
      category: 'system',
      icon: '⌨️'
    });
  }

  private addPermission(permission: Permission) {
    this.permissions.set(permission.id, permission);
  }

  /**
   * 检查单个权限状态
   */
  async checkPermission(permissionId: string): Promise<PermissionCheckResult> {
    const permission = this.permissions.get(permissionId);
    if (!permission) {
      return {
        permission: permissionId,
        status: 'denied',
        message: '未知权限类型'
      };
    }

    try {
      // 调用 Tauri 后端检查权限
      const status = await invoke<string>('check_permission', { 
        permission: permissionId 
      });

      // 更新本地状态
      permission.status = status as Permission['status'];

      return {
        permission: permissionId,
        status: permission.status,
        message: this.getStatusMessage(permission.status)
      };
    } catch (error) {
      console.error(`检查权限失败: ${permissionId}`, error);
      
      // 模拟权限检查（开发环境）
      return this.simulatePermissionCheck(permissionId);
    }
  }

  /**
   * 模拟权限检查（用于开发环境）
   */
  private simulatePermissionCheck(permissionId: string): PermissionCheckResult {
    // 模拟不同权限状态
    const simulatedStatuses: { [key: string]: Permission['status'] } = {
      'microphone': 'granted',
      'file-system': 'granted',
      'accessibility': 'not-determined',
      'notifications': 'granted',
      'screen-recording': 'not-determined',
      'automation': 'not-determined',
      'input-monitoring': 'not-determined'
    };

    const status = simulatedStatuses[permissionId] || 'not-determined';
    const permission = this.permissions.get(permissionId);
    
    if (permission) {
      permission.status = status;
    }

    return {
      permission: permissionId,
      status,
      message: this.getStatusMessage(status)
    };
  }

  /**
   * 请求权限
   */
  async requestPermission(permissionId: string): Promise<boolean> {
    const permission = this.permissions.get(permissionId);
    if (!permission) {
      console.error(`未知权限: ${permissionId}`);
      return false;
    }

    // 如果已授权，直接返回
    if (permission.status === 'granted') {
      return true;
    }

    // 显示权限请求对话框
    const shouldRequest = await this.showPermissionDialog(permission);
    if (!shouldRequest) {
      return false;
    }

    try {
      // 根据权限类型执行不同的请求流程
      switch (permissionId) {
        case 'accessibility':
          return await this.requestAccessibilityPermission();
        
        case 'microphone':
          return await this.requestMicrophonePermission();
        
        case 'file-system':
          return await this.requestFileSystemPermission();
        
        case 'notifications':
          return await this.requestNotificationPermission();
        
        case 'screen-recording':
          return await this.requestScreenRecordingPermission();
        
        case 'automation':
          return await this.requestAutomationPermission();
        
        case 'input-monitoring':
          return await this.requestInputMonitoringPermission();
        
        default:
          return false;
      }
    } catch (error) {
      console.error(`请求权限失败: ${permissionId}`, error);
      return false;
    }
  }

  /**
   * 显示权限请求对话框
   */
  private async showPermissionDialog(permission: Permission): Promise<boolean> {
    const result = await ask(
      `${permission.icon} ${permission.name}权限请求\n\n` +
      `${permission.description}\n\n` +
      `此权限${permission.required ? '是必需的' : '是可选的'}。\n` +
      `是否前往系统设置授予权限？`,
      {
        title: 'Recording King 需要您的授权',
        type: 'info'
      }
    );

    return result;
  }

  /**
   * 请求辅助功能权限（跨平台）
   */
  private async requestAccessibilityPermission(): Promise<boolean> {
    try {
      // 使用后端命令打开系统设置
      await invoke('open_system_preferences', { 
        preferencePane: 'accessibility' 
      });
      
      // 显示简短提示
      await message(
        '系统设置已打开\n\n' +
        '请找到 Recording King 并勾选复选框授予权限',
        {
          title: '辅助功能权限',
          type: 'info'
        }
      );

      // 等待用户设置
      setTimeout(async () => {
        const result = await this.checkPermission('accessibility');
        if (result.status === 'granted') {
          this.emit('permission-granted', 'accessibility');
        }
      }, 1000);

      return true;
    } catch (error) {
      console.error('打开系统设置失败:', error);
      // 如果后端方法失败，回退到shell open
      try {
        await openUrl('x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility');
        return true;
      } catch (fallbackError) {
        console.error('回退方法也失败:', fallbackError);
        return false;
      }
    }
  }

  /**
   * 请求麦克风权限
   */
  private async requestMicrophonePermission(): Promise<boolean> {
    try {
      // 优先尝试触发系统权限弹窗
      await navigator.mediaDevices.getUserMedia({ audio: true });
      return true;
    } catch (getUserMediaError) {
      console.log('getUserMedia 触发失败，引导到系统设置:', getUserMediaError);
      
      // 如果失败，打开系统设置
      try {
        await invoke('open_system_preferences', { 
          preferencePane: 'microphone' 
        });
        
        await message(
          '系统设置已打开\n\n' +
          '请找到 Recording King 并开启麦克风权限',
          {
            title: '麦克风权限',
            type: 'info'
          }
        );

        return true;
      } catch (error) {
        console.error('请求麦克风权限失败:', error);
        // 回退方法
        try {
          await openUrl('x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone');
          return true;
        } catch {
          return false;
        }
      }
    }
  }

  /**
   * 请求文件系统权限
   */
  private async requestFileSystemPermission(): Promise<boolean> {
    try {
      // macOS: 打开文件和文件夹权限
      await openUrl('x-apple.systempreferences:com.apple.preference.security?Privacy_FilesAndFolders');
      
      await message(
        '请在系统设置中：\n\n' +
        '1. 找到 Spokenly 应用\n' +
        '2. 授予文件和文件夹访问权限\n' +
        '3. 选择需要访问的文件夹\n' +
        '4. 完成后返回应用',
        {
          title: '设置文件访问权限',
          type: 'info'
        }
      );

      return true;
    } catch (error) {
      console.error('请求文件系统权限失败:', error);
      return false;
    }
  }

  /**
   * 请求通知权限
   */
  private async requestNotificationPermission(): Promise<boolean> {
    try {
      // 使用 Web Notification API
      if ('Notification' in window) {
        const permission = await Notification.requestPermission();
        const granted = permission === 'granted';
        
        if (granted) {
          const perm = this.permissions.get('notifications');
          if (perm) perm.status = 'granted';
          this.emit('permission-granted', 'notifications');
        }
        
        return granted;
      }
      
      return false;
    } catch (error) {
      console.error('请求通知权限失败:', error);
      return false;
    }
  }

  /**
   * 请求屏幕录制权限
   */
  private async requestScreenRecordingPermission(): Promise<boolean> {
    try {
      // macOS: 打开屏幕录制权限
      await openUrl('x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture');
      
      await message(
        '如果您需要屏幕录制功能：\n\n' +
        '1. 找到 Spokenly 应用\n' +
        '2. 开启屏幕录制权限\n' +
        '3. 可能需要重启应用\n' +
        '4. 完成后返回应用',
        {
          title: '设置屏幕录制权限（可选）',
          type: 'info'
        }
      );

      return true;
    } catch (error) {
      console.error('请求屏幕录制权限失败:', error);
      return false;
    }
  }

  /**
   * 请求自动化权限
   */
  private async requestAutomationPermission(): Promise<boolean> {
    try {
      // macOS: 打开自动化权限
      await openUrl('x-apple.systempreferences:com.apple.preference.security?Privacy_Automation');
      
      await message(
        '如果您需要与其他应用交互：\n\n' +
        '1. 找到 Spokenly 应用\n' +
        '2. 选择要控制的应用\n' +
        '3. 开启相应权限\n' +
        '4. 完成后返回应用',
        {
          title: '设置自动化权限（可选）',
          type: 'info'
        }
      );

      return true;
    } catch (error) {
      console.error('请求自动化权限失败:', error);
      return false;
    }
  }

  /**
   * 请求输入监控权限
   */
  private async requestInputMonitoringPermission(): Promise<boolean> {
    try {
      // 使用后端命令打开系统设置
      await invoke('open_system_preferences', { 
        preferencePane: 'input-monitoring' 
      });
      
      await message(
        '系统设置已打开\n\n' +
        '请找到 Recording King 并开启输入监控权限以使用快捷键',
        {
          title: '输入监控权限',
          type: 'info'
        }
      );

      return true;
    } catch (error) {
      console.error('请求输入监控权限失败:', error);
      // 回退方法
      try {
        await openUrl('x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent');
        return true;
      } catch {
        return false;
      }
    }
  }

  /**
   * 检查所有权限
   */
  async checkAllPermissions(): Promise<Map<string, PermissionCheckResult>> {
    const results = new Map<string, PermissionCheckResult>();
    
    for (const [id] of this.permissions) {
      const result = await this.checkPermission(id);
      results.set(id, result);
    }
    
    return results;
  }

  /**
   * 获取必需但未授权的权限
   */
  async getMissingRequiredPermissions(): Promise<Permission[]> {
    const missing: Permission[] = [];
    
    for (const [id, permission] of this.permissions) {
      if (permission.required) {
        const result = await this.checkPermission(id);
        if (result.status !== 'granted') {
          missing.push(permission);
        }
      }
    }
    
    return missing;
  }

  /**
   * 显示权限设置向导
   */
  async showPermissionWizard(): Promise<boolean> {
    const missing = await this.getMissingRequiredPermissions();
    
    if (missing.length === 0) {
      await message('所有必需权限已授予！', {
        title: '权限状态',
        type: 'info'
      });
      return true;
    }
    
    const result = await ask(
      `检测到 ${missing.length} 个必需权限未授予：\n\n` +
      missing.map(p => `${p.icon} ${p.name}`).join('\n') +
      '\n\n是否现在设置这些权限？',
      {
        title: '权限设置向导',
        type: 'warning'
      }
    );
    
    if (result) {
      for (const permission of missing) {
        await this.requestPermission(permission.id);
        // 给用户时间设置每个权限
        await new Promise(resolve => setTimeout(resolve, 1000));
      }
    }
    
    return result;
  }

  /**
   * 获取权限状态消息
   */
  private getStatusMessage(status: Permission['status']): string {
    switch (status) {
      case 'granted':
        return '✅ 已授权';
      case 'denied':
        return '❌ 已拒绝';
      case 'not-determined':
        return '⏳ 待确定';
      default:
        return '❓ 未知';
    }
  }

  /**
   * 获取所有权限
   */
  getPermissions(): Permission[] {
    return Array.from(this.permissions.values());
  }

  /**
   * 获取分类权限
   */
  getPermissionsByCategory(category: Permission['category']): Permission[] {
    return Array.from(this.permissions.values())
      .filter(p => p.category === category);
  }

  /**
   * 事件系统
   */
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

  private emit(event: string, ...args: any[]) {
    const callbacks = this.listeners.get(event);
    if (callbacks) {
      callbacks.forEach(callback => callback(...args));
    }
  }

  /**
   * 监控权限变化
   */
  async startPermissionMonitoring(interval: number = 5000) {
    setInterval(async () => {
      for (const [id, permission] of this.permissions) {
        const oldStatus = permission.status;
        const result = await this.checkPermission(id);
        
        if (oldStatus !== result.status) {
          console.log(`权限状态变化: ${id} ${oldStatus} -> ${result.status}`);
          this.emit('permission-changed', id, oldStatus, result.status);
          
          if (result.status === 'granted') {
            this.emit('permission-granted', id);
          } else if (result.status === 'denied') {
            this.emit('permission-denied', id);
          }
        }
      }
    }, interval);
  }
}

// 导出单例实例
export const permissionManager = new PermissionManager();
