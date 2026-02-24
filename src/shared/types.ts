/**
 * 全局类型定义
 * Spokenly UI 重设计项目的核心类型系统
 */

// ============================================================================
// 页面导航类型
// ============================================================================

export type Page =
  | 'general'      // 常规设置
  | 'shortcuts'    // 快捷键设置（新增）
  | 'models'       // 听写模型
  | 'transcribe'   // 转录文件
  | 'ai-prompts'   // AI 提示（新增）
  | 'history'      // 历史记录
  | 'permissions'  // 权限管理
  | 'recording'    // 语音输入
  | 'onboarding';  // 入门指南（新增）

export interface NavItem {
  key: Page;
  icon: React.ReactNode;  // SVG 图标组件替代 emoji
  label: string;
  badge?: number;         // 可选角标数字
}

// ============================================================================
// 快捷键类型
// ============================================================================

export type ShortcutPreset =
  | 'none'           // 未指定
  | 'right-cmd'      // 右⌘
  | 'right-opt'      // 右⌥
  | 'right-shift'    // 右⇧
  | 'right-ctrl'     // 右⌃
  | 'opt-cmd'        // ⌥+⌘
  | 'ctrl-cmd'       // ⌃+⌘
  | 'ctrl-opt'       // ⌃+⌥
  | 'shift-cmd'      // ⇧+⌘
  | 'opt-shift'      // ⌥+⇧
  | 'ctrl-shift'     // ⌃+⇧
  | 'fn'             // Fn
  | 'custom';        // 自定义录制

export type ModifierKey = 'cmd' | 'opt' | 'shift' | 'ctrl' | 'fn';

export interface CustomShortcut {
  type: 'custom';
  modifiers: ModifierKey[];
  key: string;
  displayLabel: string;
}

export type ActivationMode =
  | 'hold-or-toggle'  // 按住或切换（自动检测）
  | 'toggle'          // 切换（点击开始/停止）
  | 'hold'            // 按住（按下时录音）
  | 'double-click';   // 双击（快速按两次）

export interface ShortcutSettingsState {
  selectedShortcut: ShortcutPreset | CustomShortcut;
  activationMode: ActivationMode;
  escToCancel: boolean;
  testText: string;
}

// ============================================================================
// 模型相关类型
// ============================================================================

export type ModelFilter =
  | 'all'          // 全部
  | 'online'       // 在线
  | 'local'        // 本地
  | 'api'          // API
  | 'fast'         // 快速
  | 'accurate'     // 准确
  | 'punctuation'  // 标点符号
  | 'subtitle';    // 字幕

export interface ModelCardData {
  id: string;
  name: string;
  icon: string | React.ReactNode;
  provider: string;
  description: string;
  accuracy: number;       // 1-5
  speed: number;          // 1-5
  languages: string;
  type: 'online' | 'local' | 'api';
  tags: ModelFilter[];    // 支持多标签筛选
  isRealtime: boolean;    // 实时标签
  isMultilingual: boolean; // 多语言标签
  free?: boolean;
  size?: string;
  badge?: string;
  available: boolean;
  unavailableReason?: string;
}

export interface WordReplacement {
  id: string;
  from: string;   // 原始词
  to: string;     // 替换词
  enabled: boolean;
}

export interface ModelSettingsState {
  activeFilters: ModelFilter[];
  searchQuery: string;
  wordReplacements: WordReplacement[];
  showWordReplacePanel: boolean;
}

// ============================================================================
// 转录文件类型
// ============================================================================

export interface TranscribeFileState {
  selectedFile: string | null;
  fileName: string;
  isDragOver: boolean;
  isTranscribing: boolean;
  progress: number;
  result: string;
  currentModel: string;
}

export interface ActionBarButton {
  icon: string;
  label: string;
  onClick: () => void;
  variant?: 'primary' | 'secondary';
}

// ============================================================================
// AI 提示类型
// ============================================================================

export type PromptAction =
  | { type: 'google-search'; query: string }
  | { type: 'launch-app'; appName: string }
  | { type: 'close-app'; appName: string }
  | { type: 'ask-chatgpt'; prompt: string }
  | { type: 'ask-claude'; prompt: string }
  | { type: 'youtube-search'; query: string }
  | { type: 'open-website'; url: string }
  | { type: 'apple-shortcut'; shortcutName: string }
  | { type: 'shell-command'; command: string }
  | { type: 'keypress'; keys: string };

export interface AdvancedPromptSettings {
  model?: string;
  temperature?: number;
  maxTokens?: number;
}

export interface AIPrompt {
  id: string;
  name: string;
  shortcut?: CustomShortcut;
  instruction: string;
  actions: PromptAction[];
  advancedSettings?: AdvancedPromptSettings;
  enabled: boolean;
}

export interface AIPromptsState {
  prompts: AIPrompt[];
  editingPrompt: AIPrompt | null;
  showEditModal: boolean;
}

// ============================================================================
// 入门引导类型
// ============================================================================

export interface OnboardingStep {
  id: number;
  title: string;
  description: string;
  icon: string;
  action: () => Promise<boolean>;  // 返回是否完成
  isCompleted: boolean;
}

export interface OnboardingState {
  currentStep: number;
  totalSteps: number;
  completedSteps: Set<number>;
}

// ============================================================================
// 应用设置类型（扩展）
// ============================================================================

export interface AppSettings {
  // 现有字段
  openai_api_key?: string;
  luyin_token?: string;
  selected_model: string;
  auto_inject: boolean;
  inject_delay_ms: number;
  shortcut_key?: string;

  // 新增：界面设置
  display_style: 'panel' | 'notch';
  appearance: 'system' | 'dark' | 'light';
  ui_language: 'system' | 'zh-CN' | 'en';

  // 新增：行为设置
  launch_at_login: boolean;
  show_in_dock: boolean;
  show_in_menu_bar: boolean;
  esc_to_cancel: boolean;

  // 新增：快捷键设置
  shortcut_preset: ShortcutPreset;
  custom_shortcut?: CustomShortcut;
  activation_mode: ActivationMode;

  // 新增：麦克风优先级
  microphone_priority: string[];  // 设备 ID 排序列表

  // 新增：入门引导
  onboarding_complete: boolean;

  // 新增：词替换
  word_replacements: WordReplacement[];

  // 新增：转录设置
  transcription_language: string;
  transcription_prompt: string;
}

// ============================================================================
// 音频设备类型（从 useAppStore 提取）
// ============================================================================

export interface AudioDevice {
  id: string;
  name: string;
  is_default: boolean;
  is_available: boolean;
}

// ============================================================================
// 转录历史类型（从 useAppStore 提取）
// ============================================================================

export interface TranscriptionEntry {
  id: string;
  text: string;
  timestamp: number;
  duration: number;
  model: string;
  confidence: number;
  audio_file_path?: string;
}

// ============================================================================
// 录制快捷键弹窗状态
// ============================================================================

export interface RecordShortcutModalState {
  pressedModifiers: Set<ModifierKey>;
  pressedKey: string | null;
  isValid: boolean;
}

export interface RecordShortcutModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSave: (shortcut: CustomShortcut) => void;
  currentShortcut?: CustomShortcut;
}

// ============================================================================
// 常规设置状态
// ============================================================================

export interface GeneralSettingsState {
  // 界面区域
  displayStyle: 'panel' | 'notch';
  appearance: 'system' | 'dark' | 'light';
  language: 'system' | 'zh-CN' | 'en';

  // 行为区域
  launchAtLogin: boolean;
  showInDock: boolean;
  showInMenuBar: boolean;
  escToCancelRecording: boolean;

  // 麦克风优先级
  microphonePriority: AudioDevice[];
}
