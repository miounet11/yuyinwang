/**
 * 工具函数库
 * Spokenly UI 重设计项目的核心工具函数
 */

import type {
  ModifierKey,
  CustomShortcut,
  ShortcutPreset,
  ModelCardData,
  ModelFilter,
  AudioDevice,
  WordReplacement,
} from './types';

// ============================================================================
// 快捷键验证
// ============================================================================

/**
 * 验证自定义快捷键组合的有效性
 *
 * 前置条件：
 * - modifiers 是 ModifierKey 的集合（可为空）
 * - key 是字符串或 null
 *
 * 后置条件：
 * - isValid = true 当且仅当 modifiers.size >= 1 且 key !== null 且 key 不是修饰键
 * - 如果 modifiers.size === 0，errorMessage 为 "需要至少一个修饰键"
 * - 如果 key === null，errorMessage 为 "需要一个普通键"
 * - 无副作用
 *
 * 需求: 4.5, 4.6, 4.7
 */
export function validateCustomShortcut(
  modifiers: Set<ModifierKey>,
  key: string | null
): { isValid: boolean; errorMessage?: string } {
  const MODIFIER_KEYS = new Set(['Meta', 'Control', 'Shift', 'Alt', 'Fn']);

  if (modifiers.size === 0) {
    return { isValid: false, errorMessage: '需要至少一个修饰键' };
  }

  if (key === null || key === '') {
    return { isValid: false, errorMessage: '需要一个普通键' };
  }

  // 检查 key 是否是修饰键
  if (MODIFIER_KEYS.has(key)) {
    return { isValid: false, errorMessage: '普通键不能是修饰键' };
  }

  return { isValid: true };
}

// ============================================================================
// 模型筛选
// ============================================================================

/**
 * 筛选模型列表
 *
 * 前置条件：
 * - models 是非空的模型数组
 * - activeFilters 是有效的筛选标签数组
 *
 * 后置条件：
 * - 如果 activeFilters 包含 'all'，返回全部模型
 * - 否则返回的模型集合中，每个模型的 tags 至少包含 activeFilters 中的一个标签
 * - 返回数组的长度 ≤ 输入 models 的长度
 * - 不修改输入数组
 *
 * 需求: 5.3, 5.4
 */
export function filterModels(
  models: ModelCardData[],
  activeFilters: ModelFilter[]
): ModelCardData[] {
  // 如果包含 'all' 或筛选条件为空，返回全部模型
  if (activeFilters.length === 0 || activeFilters.includes('all')) {
    return [...models];
  }

  // 筛选：模型的 tags 至少包含一个 activeFilters 中的标签
  return models.filter((model) =>
    activeFilters.some((filter) => model.tags.includes(filter))
  );
}

/**
 * 筛选并排序模型列表
 *
 * 排序规则：
 * 1. 当前选中的模型排在最前
 * 2. 可用模型排在不可用模型之前
 *
 * 需求: 5.6
 */
export function filterAndSortModels(
  models: ModelCardData[],
  filters: ModelFilter[],
  selectedModelId: string
): ModelCardData[] {
  // 步骤 1：应用筛选
  let filtered = filterModels(models, filters);

  // 步骤 2：排序
  filtered.sort((a, b) => {
    // 选中的模型排最前
    if (a.id === selectedModelId) return -1;
    if (b.id === selectedModelId) return 1;

    // 可用模型排在不可用模型之前
    if (a.available && !b.available) return -1;
    if (!a.available && b.available) return 1;

    return 0;
  });

  return filtered;
}

// ============================================================================
// 麦克风排序
// ============================================================================

/**
 * 重新排列麦克风设备优先级
 *
 * 前置条件：
 * - devices 是非空数组
 * - 0 ≤ fromIndex < devices.length
 * - 0 ≤ toIndex < devices.length
 * - fromIndex ≠ toIndex
 *
 * 后置条件：
 * - 返回新数组，长度与输入相同
 * - devices[fromIndex] 移动到 toIndex 位置
 * - 其他元素相对顺序保持不变
 * - 不修改原始数组
 *
 * 需求: 2.7
 */
export function reorderMicrophones(
  devices: AudioDevice[],
  fromIndex: number,
  toIndex: number
): AudioDevice[] {
  // 前置条件检查
  if (fromIndex === toIndex) return devices;
  if (fromIndex < 0 || fromIndex >= devices.length) return devices;
  if (toIndex < 0 || toIndex >= devices.length) return devices;

  // 创建新数组，移动元素
  const result = [...devices];
  const [removed] = result.splice(fromIndex, 1);
  result.splice(toIndex, 0, removed);

  return result;
}

// ============================================================================
// 文件拖放处理
// ============================================================================

/**
 * 处理文件拖放事件
 *
 * 前置条件：
 * - event 是有效的 DragEvent，包含 dataTransfer.files
 * - supportedFormats 是非空的文件扩展名数组（如 ['.mp3', '.wav']）
 *
 * 后置条件：
 * - 如果拖入的文件扩展名在 supportedFormats 中，返回 { filePath, fileName }
 * - 如果文件格式不支持，返回 { error: "不支持的文件格式: .xxx" }
 * - 如果没有文件，返回 { error: "未检测到文件" }
 * - 只处理第一个文件（忽略多文件拖入）
 *
 * 需求: 6.3, 6.4, 6.5, 6.6, 6.9
 */
export async function handleFileDrop(
  event: DragEvent,
  supportedFormats: string[]
): Promise<{ filePath: string; fileName: string } | { error: string }> {
  const files = event.dataTransfer?.files;

  if (!files || files.length === 0) {
    return { error: '未检测到文件' };
  }

  // 只处理第一个文件
  const file = files[0];
  const fileName = file.name;
  const ext = '.' + fileName.split('.').pop()?.toLowerCase();

  // 大小写不敏感的格式验证
  const normalizedFormats = supportedFormats.map((f) => f.toLowerCase());
  if (!normalizedFormats.includes(ext)) {
    return { error: `不支持的文件格式: ${ext}` };
  }

  // 返回文件路径和文件名
  // 注意：在浏览器环境中，File 对象没有真实路径，需要通过 Tauri API 处理
  return {
    filePath: file.path || fileName, // Tauri 环境下 file.path 可用
    fileName,
  };
}

// ============================================================================
// 快捷键预设映射
// ============================================================================

/**
 * 将快捷键预设转换为 Tauri 键值
 *
 * 需求: 3.2
 */
export function presetToTauriKey(preset: ShortcutPreset): string | null {
  const mapping: Record<ShortcutPreset, string | null> = {
    none: null,
    'right-cmd': 'RightCommand',
    'right-opt': 'RightOption',
    'right-shift': 'RightShift',
    'right-ctrl': 'RightControl',
    'opt-cmd': 'Option+Command',
    'ctrl-cmd': 'Control+Command',
    'ctrl-opt': 'Control+Option',
    'shift-cmd': 'Shift+Command',
    'opt-shift': 'Option+Shift',
    'ctrl-shift': 'Control+Shift',
    fn: 'Fn',
    custom: null,
  };

  return mapping[preset];
}

/**
 * 将 Tauri 键值转换为快捷键预设
 *
 * 需求: 3.2
 */
export function tauriKeyToPreset(key: string): ShortcutPreset {
  const reverseMapping: Record<string, ShortcutPreset> = {
    RightCommand: 'right-cmd',
    RightOption: 'right-opt',
    RightShift: 'right-shift',
    RightControl: 'right-ctrl',
    'Option+Command': 'opt-cmd',
    'Control+Command': 'ctrl-cmd',
    'Control+Option': 'ctrl-opt',
    'Shift+Command': 'shift-cmd',
    'Option+Shift': 'opt-shift',
    'Control+Shift': 'ctrl-shift',
    Fn: 'fn',
  };

  return reverseMapping[key] || 'none';
}

/**
 * 将自定义快捷键转换为 Tauri 键值
 */
export function customShortcutToTauriKey(shortcut: CustomShortcut): string {
  const modifierMap: Record<ModifierKey, string> = {
    cmd: 'Command',
    opt: 'Option',
    shift: 'Shift',
    ctrl: 'Control',
    fn: 'Fn',
  };

  const modifiers = shortcut.modifiers.map((m) => modifierMap[m]).join('+');
  return `${modifiers}+${shortcut.key}`;
}

// ============================================================================
// XSS 转义
// ============================================================================

/**
 * 转义 HTML 特殊字符以防止 XSS 注入
 *
 * 后置条件：
 * - 返回的字符串不包含未转义的 HTML 特殊字符（<, >, &, ", '）
 *
 * 需求: 12.3
 */
export function escapeHtml(text: string): string {
  const map: Record<string, string> = {
    '<': '&lt;',
    '>': '&gt;',
    '&': '&amp;',
    '"': '&quot;',
    "'": '&#39;',
  };

  return text.replace(/[<>&"']/g, (char) => map[char]);
}

// ============================================================================
// 词替换
// ============================================================================

/**
 * 应用词替换规则到文本
 *
 * 后置条件：
 * - 连续应用两次的结果与应用一次相同（幂等性，假设规则不产生循环替换）
 *
 * 需求: 5.8
 */
export function applyWordReplacements(
  text: string,
  rules: WordReplacement[]
): string {
  let result = text;

  // 只应用启用的规则
  const enabledRules = rules.filter((rule) => rule.enabled);

  for (const rule of enabledRules) {
    // 使用全局正则替换，支持词边界匹配
    const regex = new RegExp(`\\b${escapeRegExp(rule.from)}\\b`, 'g');
    result = result.replace(regex, rule.to);
  }

  return result;
}

/**
 * 转义正则表达式特殊字符
 */
function escapeRegExp(text: string): string {
  return text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

// ============================================================================
// 系统快捷键冲突检测
// ============================================================================

/**
 * 检测快捷键是否与系统关键快捷键冲突
 *
 * 需求: 12.2
 */
export function isSystemShortcutConflict(shortcut: CustomShortcut): boolean {
  const systemShortcuts = [
    { modifiers: ['cmd'], key: 'Q' }, // 退出应用
    { modifiers: ['cmd'], key: 'W' }, // 关闭窗口
    { modifiers: ['cmd'], key: 'H' }, // 隐藏窗口
    { modifiers: ['cmd', 'opt'], key: 'Escape' }, // 强制退出
    { modifiers: ['cmd', 'ctrl'], key: 'Q' }, // 锁定屏幕
  ];

  return systemShortcuts.some(
    (sys) =>
      sys.modifiers.length === shortcut.modifiers.length &&
      sys.modifiers.every((m) => shortcut.modifiers.includes(m as ModifierKey)) &&
      sys.key.toLowerCase() === shortcut.key.toLowerCase()
  );
}

// ============================================================================
// 快捷键显示标签生成
// ============================================================================

/**
 * 生成快捷键的显示标签
 */
export function generateShortcutLabel(shortcut: CustomShortcut): string {
  const symbolMap: Record<ModifierKey, string> = {
    cmd: '⌘',
    opt: '⌥',
    shift: '⇧',
    ctrl: '⌃',
    fn: 'Fn',
  };

  const modifiers = shortcut.modifiers.map((m) => symbolMap[m]).join('');
  return `${modifiers}${shortcut.key.toUpperCase()}`;
}
