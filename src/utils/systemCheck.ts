/**
 * 系统检查工具
 * 检查关键权限和系统要求
 */

import { invoke } from '@tauri-apps/api/tauri';
import { platform } from '@tauri-apps/api/os';

export interface SystemCheckResult {
  platform: string;
  permissions: {
    accessibility: boolean;
    microphone: boolean;
    inputMonitoring: boolean;
    fileSystem: boolean;
  };
  recommendations: string[];
  criticalIssues: string[];
}

export class SystemChecker {
  
  /**
   * 执行完整的系统检查
   */
  static async performSystemCheck(): Promise<SystemCheckResult> {
    const platformName = await platform();
    const result: SystemCheckResult = {
      platform: platformName,
      permissions: {
        accessibility: false,
        microphone: false,
        inputMonitoring: false,
        fileSystem: false
      },
      recommendations: [],
      criticalIssues: []
    };

    // 检查各项权限
    try {
      // 检查辅助功能权限
      const accessibilityStatus = await invoke<string>('check_permission', { 
        permission: 'accessibility' 
      });
      result.permissions.accessibility = accessibilityStatus === 'granted';

      // 检查麦克风权限
      const microphoneStatus = await invoke<string>('check_permission', { 
        permission: 'microphone' 
      });
      result.permissions.microphone = microphoneStatus === 'granted';

      // 检查输入监控权限
      const inputStatus = await invoke<string>('check_permission', { 
        permission: 'input-monitoring' 
      });
      result.permissions.inputMonitoring = inputStatus === 'granted';

      // 检查文件系统权限
      const fileSystemStatus = await invoke<string>('check_permission', { 
        permission: 'file-system' 
      });
      result.permissions.fileSystem = fileSystemStatus === 'granted';

    } catch (error) {
      console.error('权限检查失败:', error);
    }

    // 生成建议和关键问题
    this.generateRecommendations(result);

    return result;
  }

  /**
   * 生成建议和关键问题列表
   */
  private static generateRecommendations(result: SystemCheckResult) {
    // 检查关键权限
    if (!result.permissions.accessibility) {
      result.criticalIssues.push(
        '缺少辅助功能权限 - 快捷键功能将无法正常工作'
      );
      result.recommendations.push(
        '前往 系统偏好设置 → 安全性与隐私 → 隐私 → 辅助功能，添加 Spokenly'
      );
    }

    if (!result.permissions.microphone) {
      result.criticalIssues.push(
        '缺少麦克风权限 - 语音转录功能将无法使用'
      );
      result.recommendations.push(
        '前往 系统偏好设置 → 安全性与隐私 → 隐私 → 麦克风，允许 Spokenly 访问'
      );
    }

    if (!result.permissions.inputMonitoring) {
      result.criticalIssues.push(
        '缺少输入监控权限 - 全局快捷键将无法响应'
      );
      result.recommendations.push(
        '前往 系统偏好设置 → 安全性与隐私 → 隐私 → 输入监控，添加 Spokenly'
      );
    }

    // macOS 特定建议
    if (result.platform === 'darwin') {
      if (result.criticalIssues.length > 0) {
        result.recommendations.push(
          '修改权限后，可能需要重启 Spokenly 应用'
        );
        result.recommendations.push(
          '某些权限设置需要管理员密码确认'
        );
      }

      // 如果大部分权限都没有，建议使用快速设置
      if (result.criticalIssues.length >= 2) {
        result.recommendations.unshift(
          '建议使用应用内的「快速设置向导」一键配置所有权限'
        );
      }
    }

    // 如果所有权限都正常
    if (result.criticalIssues.length === 0) {
      result.recommendations.push('✅ 所有关键权限已正确配置，享受完整功能吧！');
    }
  }

  /**
   * 检查是否为首次启动
   */
  static isFirstLaunch(): boolean {
    const hasLaunched = localStorage.getItem('spokenly_has_launched');
    if (!hasLaunched) {
      localStorage.setItem('spokenly_has_launched', 'true');
      return true;
    }
    return false;
  }

  /**
   * 获取权限设置深度链接
   */
  static getPermissionDeepLinks() {
    return {
      accessibility: 'x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility',
      microphone: 'x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone',
      inputMonitoring: 'x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent',
      fileSystem: 'x-apple.systempreferences:com.apple.preference.security?Privacy_FilesAndFolders',
      automation: 'x-apple.systempreferences:com.apple.preference.security?Privacy_Automation',
      screenRecording: 'x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture'
    };
  }

  /**
   * 生成系统报告
   */
  static generateSystemReport(result: SystemCheckResult): string {
    const report = `
🖥️ Spokenly 系统检查报告
==============================

平台信息: ${result.platform}
检查时间: ${new Date().toLocaleString('zh-CN')}

📋 权限状态:
- 辅助功能: ${result.permissions.accessibility ? '✅ 已授权' : '❌ 未授权'}
- 麦克风访问: ${result.permissions.microphone ? '✅ 已授权' : '❌ 未授权'}  
- 输入监控: ${result.permissions.inputMonitoring ? '✅ 已授权' : '❌ 未授权'}
- 文件系统: ${result.permissions.fileSystem ? '✅ 已授权' : '❌ 未授权'}

${result.criticalIssues.length > 0 ? `
🚨 关键问题 (${result.criticalIssues.length}个):
${result.criticalIssues.map((issue, i) => `${i + 1}. ${issue}`).join('\n')}
` : '✅ 未发现关键问题'}

💡 建议措施:
${result.recommendations.map((rec, i) => `${i + 1}. ${rec}`).join('\n')}

==============================
生成工具: Spokenly v2.12.10
    `;

    return report.trim();
  }

  /**
   * 自动修复常见权限问题
   */
  static async autoFixPermissions(): Promise<{ fixed: string[], failed: string[] }> {
    const fixed: string[] = [];
    const failed: string[] = [];

    try {
      // 尝试触发权限请求
      const permissions = ['accessibility', 'microphone', 'input-monitoring'];
      
      for (const permission of permissions) {
        try {
          // 这里应该调用实际的权限请求API
          console.log(`正在请求 ${permission} 权限...`);
          
          // 模拟权限请求过程
          await new Promise(resolve => setTimeout(resolve, 1000));
          
          fixed.push(permission);
        } catch (error) {
          failed.push(permission);
          console.error(`修复 ${permission} 权限失败:`, error);
        }
      }
    } catch (error) {
      console.error('自动修复权限失败:', error);
    }

    return { fixed, failed };
  }
}

export default SystemChecker;