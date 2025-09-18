import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import "./TextInjectionSettings.css";

interface TextInjectionConfig {
  auto_inject_enabled: boolean;
  inject_delay_ms: number;
  use_keyboard_simulation: boolean;
  preserve_clipboard: boolean;
  duplicate_detection: boolean;
  shortcut_delay_ms: number;
  target_app_filter: string[];
}

interface AppInfo {
  name: string;
  bundle_id: string;
  process_id: number;
}

interface TextInjectionSettingsProps {
  isVisible: boolean;
  onClose: () => void;
  onConfigChange?: (config: TextInjectionConfig) => void;
}

export default function TextInjectionSettings({
  isVisible,
  onClose,
  onConfigChange,
}: TextInjectionSettingsProps) {
  const [config, setConfig] = useState<TextInjectionConfig>({
    auto_inject_enabled: false,
    inject_delay_ms: 100,
    use_keyboard_simulation: false,
    preserve_clipboard: true,
    duplicate_detection: true,
    shortcut_delay_ms: 50,
    target_app_filter: [],
  });

  const [hasPermission, setHasPermission] = useState<boolean>(false);
  const [activeApp, setActiveApp] = useState<AppInfo | null>(null);
  const [testText, setTestText] = useState("Hello from Recording King! 📝");
  const [testResult, setTestResult] = useState<string>("");
  const [loading, setLoading] = useState(false);
  const [newAppFilter, setNewAppFilter] = useState("");
  const [healthStatus, setHealthStatus] = useState<any>(null);
  const [environmentStatus, setEnvironmentStatus] = useState<any>(null);
  const [autoFixing, setAutoFixing] = useState(false);

  // 加载默认配置
  useEffect(() => {
    if (isVisible) {
      loadDefaultConfig();
      checkPermission();
      getActiveAppInfo();
    }
  }, [isVisible]);

  const loadDefaultConfig = async () => {
    try {
      const defaultConfig = await invoke<TextInjectionConfig>(
        "get_default_text_injection_config",
      );
      setConfig(defaultConfig);
    } catch (error) {
      console.error("加载默认配置失败:", error);
    }
  };

  const checkPermission = async () => {
    try {
      const permission = await invoke<boolean>(
        "check_text_injection_permission",
      );
      setHasPermission(permission);
    } catch (error) {
      console.error("检查权限失败:", error);
      setHasPermission(false);
    }
  };

  const getActiveAppInfo = async () => {
    try {
      const appInfo = await invoke<AppInfo>("get_active_app_info");
      setActiveApp(appInfo);
    } catch (error) {
      console.error("获取活动应用信息失败:", error);
    }
  };

  const validateConfig = async (
    configToValidate: TextInjectionConfig,
  ): Promise<boolean> => {
    try {
      const isValid = await invoke<boolean>("validate_text_injection_config", {
        config: configToValidate,
      });
      return isValid;
    } catch (error) {
      console.error("配置验证失败:", error);
      alert(`配置验证失败: ${error}`);
      return false;
    }
  };

  const testTextInjection = async () => {
    if (!hasPermission) {
      alert("请先在系统偏好设置中授予辅助功能权限");
      return;
    }

    setLoading(true);
    setTestResult("");

    try {
      const result = await invoke<string>("test_text_injection");
      setTestResult(result);
    } catch (error) {
      setTestResult(`测试失败: ${error}`);
    } finally {
      setLoading(false);
    }
  };

  const testCustomText = async () => {
    if (!hasPermission) {
      alert("请先在系统偏好设置中授予辅助功能权限");
      return;
    }

    if (!testText.trim()) {
      alert("请输入测试文本");
      return;
    }

    setLoading(true);

    try {
      const success = await invoke<boolean>("inject_text_to_cursor", {
        text: testText,
      });

      if (success) {
        setTestResult(`✅ 自定义文本注入成功: ${testText}`);
      } else {
        setTestResult("❌ 自定义文本注入失败");
      }
    } catch (error) {
      setTestResult(`❌ 自定义文本注入失败: ${error}`);
    } finally {
      setLoading(false);
    }
  };

  // 健康检查
  const performHealthCheck = async () => {
    try {
      setLoading(true);
      const health = await invoke("text_injection_health_check");
      setHealthStatus(health);

      const environment = await invoke("validate_injection_environment");
      setEnvironmentStatus(environment);

      console.log("健康检查完成:", health);
    } catch (error) {
      console.error("健康检查失败:", error);
      setTestResult(`❌ 健康检查失败: ${error}`);
    } finally {
      setLoading(false);
    }
  };

  // 自动修复问题
  const autoFixIssues = async () => {
    try {
      setAutoFixing(true);
      const fixes = await invoke("fix_text_injection_issues");

      if (fixes.length > 0) {
        setTestResult(`🔧 修复建议:\n${fixes.join("\n")}`);
      } else {
        setTestResult("✅ 系统运行正常，无需修复");
      }

      // 重新进行健康检查
      await performHealthCheck();
    } catch (error) {
      console.error("自动修复失败:", error);
      setTestResult(`❌ 自动修复失败: ${error}`);
    } finally {
      setAutoFixing(false);
    }
  };

  const smartInjectText = async () => {
    if (!hasPermission) {
      alert("请先在系统偏好设置中授予辅助功能权限");
      return;
    }

    if (!testText.trim()) {
      alert("请输入测试文本");
      return;
    }

    setLoading(true);

    try {
      const injected = await invoke<boolean>("smart_inject_text", {
        text: testText,
        config,
      });

      if (injected) {
        setTestResult(`✅ 智能文本注入成功: ${testText}`);
      } else {
        setTestResult("ℹ️ 文本注入被跳过（重复或禁用）");
      }
    } catch (error) {
      setTestResult(`❌ 智能文本注入失败: ${error}`);
    } finally {
      setLoading(false);
    }
  };

  const handleConfigChange = (key: keyof TextInjectionConfig, value: any) => {
    const newConfig = { ...config, [key]: value };
    setConfig(newConfig);
    onConfigChange?.(newConfig);
  };

  const addAppFilter = () => {
    if (
      newAppFilter.trim() &&
      !config.target_app_filter.includes(newAppFilter.trim())
    ) {
      const newFilter = [...config.target_app_filter, newAppFilter.trim()];
      handleConfigChange("target_app_filter", newFilter);
      setNewAppFilter("");
    }
  };

  const removeAppFilter = (index: number) => {
    const newFilter = config.target_app_filter.filter((_, i) => i !== index);
    handleConfigChange("target_app_filter", newFilter);
  };

  const clearHistory = async () => {
    try {
      await invoke("clear_text_injection_history");
      alert("文本注入历史已清除");
    } catch (error) {
      alert(`清除历史失败: ${error}`);
    }
  };

  const handleSave = async () => {
    const isValid = await validateConfig(config);
    if (isValid) {
      alert("配置已保存");
      onClose();
    }
  };

  if (!isVisible) return null;

  return (
    <div className="text-injection-overlay">
      <div className="text-injection-modal">
        <div className="text-injection-header">
          <h2>🎯 文本注入设置</h2>
          <button className="close-btn" onClick={onClose}>
            ×
          </button>
        </div>

        <div className="text-injection-content">
          {/* 权限状态 */}
          <div className="permission-status">
            <div
              className={`permission-indicator ${hasPermission ? "granted" : "denied"}`}
            >
              {hasPermission ? "✅ 辅助功能权限已授予" : "❌ 需要辅助功能权限"}
            </div>
            {!hasPermission && (
              <div className="permission-help">
                请在 系统偏好设置 → 安全性与隐私 → 隐私 → 辅助功能 中启用此应用
              </div>
            )}
          </div>

          {/* 当前活动应用 */}
          {activeApp && (
            <div className="active-app-info">
              <h3>🎯 当前活动应用</h3>
              <div className="app-details">
                <div>
                  <strong>应用名称:</strong> {activeApp.name}
                </div>
                <div>
                  <strong>Bundle ID:</strong> {activeApp.bundle_id}
                </div>
                <div>
                  <strong>进程ID:</strong> {activeApp.process_id}
                </div>
              </div>
              <button onClick={getActiveAppInfo} className="refresh-btn">
                🔄 刷新
              </button>
            </div>
          )}

          {/* 配置选项 */}
          <div className="config-section">
            <h3>⚙️ 注入配置</h3>

            <div className="config-group">
              <label className="config-item">
                <input
                  type="checkbox"
                  checked={config.auto_inject_enabled}
                  onChange={(e) =>
                    handleConfigChange("auto_inject_enabled", e.target.checked)
                  }
                />
                启用自动注入
              </label>

              <label className="config-item">
                <input
                  type="checkbox"
                  checked={config.use_keyboard_simulation}
                  onChange={(e) =>
                    handleConfigChange(
                      "use_keyboard_simulation",
                      e.target.checked,
                    )
                  }
                />
                使用键盘模拟（否则使用剪贴板）
              </label>

              <label className="config-item">
                <input
                  type="checkbox"
                  checked={config.preserve_clipboard}
                  onChange={(e) =>
                    handleConfigChange("preserve_clipboard", e.target.checked)
                  }
                />
                保留剪贴板内容
              </label>

              <label className="config-item">
                <input
                  type="checkbox"
                  checked={config.duplicate_detection}
                  onChange={(e) =>
                    handleConfigChange("duplicate_detection", e.target.checked)
                  }
                />
                启用重复检测
              </label>
            </div>

            <div className="config-group">
              <div className="config-item">
                <label>注入延迟 (毫秒):</label>
                <input
                  type="number"
                  min="0"
                  max="10000"
                  value={config.inject_delay_ms}
                  onChange={(e) =>
                    handleConfigChange(
                      "inject_delay_ms",
                      parseInt(e.target.value),
                    )
                  }
                />
              </div>

              <div className="config-item">
                <label>快捷键延迟 (毫秒):</label>
                <input
                  type="number"
                  min="0"
                  max="5000"
                  value={config.shortcut_delay_ms}
                  onChange={(e) =>
                    handleConfigChange(
                      "shortcut_delay_ms",
                      parseInt(e.target.value),
                    )
                  }
                />
              </div>
            </div>
          </div>

          {/* 目标应用过滤器 */}
          <div className="app-filter-section">
            <h3>🎯 目标应用过滤器</h3>
            <div className="filter-input">
              <input
                type="text"
                placeholder="输入应用Bundle ID或名称"
                value={newAppFilter}
                onChange={(e) => setNewAppFilter(e.target.value)}
                onKeyPress={(e) => e.key === "Enter" && addAppFilter()}
              />
              <button onClick={addAppFilter}>添加</button>
            </div>
            <div className="filter-list">
              {config.target_app_filter.map((filter, index) => (
                <div key={index} className="filter-item">
                  <span>{filter}</span>
                  <button onClick={() => removeAppFilter(index)}>×</button>
                </div>
              ))}
            </div>
          </div>

          {/* 测试区域 */}
          <div className="test-section">
            <h3>🧪 功能测试</h3>

            <div className="test-input">
              <input
                type="text"
                placeholder="输入测试文本"
                value={testText}
                onChange={(e) => setTestText(e.target.value)}
              />
            </div>

            <div className="test-buttons">
              <button
                onClick={testTextInjection}
                disabled={loading || !hasPermission}
                className="test-btn"
              >
                {loading ? "测试中..." : "🎯 快速测试"}
              </button>

              <button
                onClick={testCustomText}
                disabled={loading || !hasPermission}
                className="test-btn"
              >
                {loading ? "注入中..." : "📝 简单注入"}
              </button>

              <button
                onClick={smartInjectText}
                disabled={loading || !hasPermission}
                className="test-btn"
              >
                {loading ? "智能注入中..." : "🧠 智能注入"}
              </button>
            </div>

            {testResult && (
              <div className="test-result">
                <pre>{testResult}</pre>
              </div>
            )}
          </div>

          {/* Story 1.6: 健康检查和修复功能 */}
          <div className="health-section">
            <h3>🏥 系统诊断</h3>

            <div className="diagnostic-buttons">
              <button
                onClick={performHealthCheck}
                disabled={loading}
                className="diagnostic-btn health-check"
              >
                {loading ? "检查中..." : "🏥 健康检查"}
              </button>
              <button
                onClick={autoFixIssues}
                disabled={autoFixing}
                className="diagnostic-btn auto-fix"
              >
                {autoFixing ? "修复中..." : "🔧 自动修复"}
              </button>
            </div>

            {/* 健康状态显示 */}
            {healthStatus && (
              <div className="health-status-panel">
                <h4>📊 系统状态 (评分: {healthStatus.health_score}/100)</h4>
                <div className="status-indicators">
                  <div
                    className={`status-indicator ${healthStatus.accessibility_permission ? "good" : "bad"}`}
                  >
                    <span className="status-icon">
                      {healthStatus.accessibility_permission ? "✅" : "❌"}
                    </span>
                    <span className="status-label">辅助功能权限</span>
                  </div>
                  {healthStatus.current_app && (
                    <div className="status-indicator good">
                      <span className="status-icon">🎯</span>
                      <span className="status-label">
                        当前应用: {healthStatus.current_app.name}
                      </span>
                    </div>
                  )}
                </div>
              </div>
            )}

            {/* 环境状态显示 */}
            {environmentStatus && (
              <div className="environment-status-panel">
                <h4>🌍 环境检测</h4>
                <div className="env-grid">
                  <div
                    className={`env-item ${environmentStatus.has_accessibility_permission ? "good" : "bad"}`}
                  >
                    <span>🔓 权限</span>
                    <span>
                      {environmentStatus.has_accessibility_permission
                        ? "✅"
                        : "❌"}
                    </span>
                  </div>
                  <div
                    className={`env-item ${environmentStatus.active_app_detected ? "good" : "bad"}`}
                  >
                    <span>🎯 应用检测</span>
                    <span>
                      {environmentStatus.active_app_detected ? "✅" : "❌"}
                    </span>
                  </div>
                  <div
                    className={`env-item ${environmentStatus.clipboard_available ? "good" : "bad"}`}
                  >
                    <span>📋 剪贴板</span>
                    <span>
                      {environmentStatus.clipboard_available ? "✅" : "❌"}
                    </span>
                  </div>
                  <div
                    className={`env-item ${environmentStatus.applescript_available ? "good" : "bad"}`}
                  >
                    <span>🍎 脚本</span>
                    <span>
                      {environmentStatus.applescript_available ? "✅" : "❌"}
                    </span>
                  </div>
                </div>

                {environmentStatus.errors &&
                  environmentStatus.errors.length > 0 && (
                    <div className="error-panel">
                      <h5>⚠️ 检测到的问题:</h5>
                      <ul className="error-list">
                        {environmentStatus.errors.map(
                          (error: string, index: number) => (
                            <li key={index} className="error-item">
                              {error}
                            </li>
                          ),
                        )}
                      </ul>
                    </div>
                  )}
              </div>
            )}
          </div>

          {/* 操作按钮 */}
          <div className="action-buttons">
            <button onClick={clearHistory} className="action-btn">
              🧹 清除历史
            </button>
            <button onClick={checkPermission} className="action-btn">
              🔍 检查权限
            </button>
            <button onClick={handleSave} className="save-btn">
              💾 保存配置
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
