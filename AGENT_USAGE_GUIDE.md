# AI Agent 处理系统使用指南

## 概述

Spokenly Clone 现在集成了完整的 AI Agent 处理系统，支持多种文本处理类型、链式处理和批量处理。

## 功能特性

### 1. AI 提示词管理
- **获取所有提示词**: `get_ai_prompts()`
- **保存/更新提示词**: `save_ai_prompt(prompt)`
- **删除提示词**: `delete_ai_prompt(prompt_id)`
- **激活提示词**: `activate_ai_prompt(prompt_id, agent_type)`

### 2. 单个 Agent 处理
- **处理请求**: `process_with_agent(request)`

### 3. 链式处理
- **链式处理**: `process_with_chain(request)`
- **获取可用链**: `get_available_chains()`

### 4. 批量处理
- **批量处理**: `process_batch(request)`

### 5. 配置管理
- **设置 OpenAI API 密钥**: `set_openai_api_key(api_key)`
- **获取 Agent 类型**: `get_agent_types()`

## 支持的 Agent 类型

1. **speech-to-text** - 语音转文字
2. **text-enhancer** - 文本增强
3. **translator** - 翻译
4. **summarizer** - 摘要
5. **formatter** - 格式化
6. **grammar-check** - 语法检查
7. **tone-adjuster** - 语调调整
8. **auto-input** - 自动输入

## 使用示例

### TypeScript/JavaScript 前端调用示例

```typescript
import { invoke } from '@tauri-apps/api/tauri';

// 1. 设置 OpenAI API 密钥
await invoke('set_openai_api_key', { apiKey: 'your-openai-api-key' });

// 2. 单个 Agent 处理
const agentRequest = {
  agent_type: 'text-enhancer',
  input_text: '这是需要优化的文本内容',
  prompt_id: null, // 或指定特定的提示词ID
  additional_context: {
    'target_audience': '专业读者',
    'style': '正式'
  }
};

const result = await invoke('process_with_agent', { request: agentRequest });
console.log('处理结果:', result);

// 3. 链式处理
const chainRequest = {
  chain_id: 'enhance-translate-summarize',
  input_text: '原始文本内容',
  additional_context: {
    'target_language': '英语'
  }
};

const chainResult = await invoke('process_with_chain', { request: chainRequest });
console.log('链式处理结果:', chainResult);

// 4. 批量处理
const batchRequest = {
  agent_type: 'summarizer',
  input_texts: [
    '第一段需要总结的文本',
    '第二段需要总结的文本',
    '第三段需要总结的文本'
  ],
  prompt_id: null,
  additional_context: {
    'max_length': '100字'
  }
};

const batchResult = await invoke('process_batch', { request: batchRequest });
console.log('批量处理结果:', batchResult);

// 5. 管理AI提示词
const newPrompt = {
  id: '',
  name: '专业翻译',
  description: '专门用于技术文档翻译',
  agent_type: 'translator',
  prompt_text: '请将以下技术文档翻译为标准的英文，保持技术术语的准确性：',
  is_active: true,
  created_at: 0,
  updated_at: 0
};

const savedPrompt = await invoke('save_ai_prompt', { prompt: newPrompt });
console.log('保存的提示词:', savedPrompt);

// 激活提示词
await invoke('activate_ai_prompt', { 
  promptId: savedPrompt.id, 
  agentType: 'translator' 
});
```

## 预定义处理链

### 1. enhance-translate-summarize
- 文本增强 → 翻译 → 摘要
- 适用于需要优化并翻译后总结的内容

### 2. grammar-format
- 语法检查 → 格式化
- 适用于文档整理和校对

### 3. speech-enhance-input
- 语音转文字 → 文本增强 → 自动输入
- 适用于语音输入的内容处理

## 错误处理

系统提供完善的错误处理机制：

```typescript
const result = await invoke('process_with_agent', { request: agentRequest });

if (!result.success) {
  console.error('处理失败:', result.error);
  // 处理错误情况
} else {
  console.log('处理成功:', result.output_text);
  console.log('处理时间:', result.processing_time_ms + 'ms');
}
```

## 配置要求

1. **OpenAI API 密钥**: 必须设置有效的 OpenAI API 密钥
2. **网络连接**: 需要稳定的网络连接访问 OpenAI API
3. **环境变量**: 可选择在环境变量中设置 `OPENAI_API_KEY`

## 性能特性

- **异步处理**: 所有 AI 调用都是异步的，不会阻塞 UI
- **错误恢复**: 单个处理失败不会影响其他处理
- **处理时间跟踪**: 提供详细的性能统计
- **批量优化**: 批量处理针对大量数据进行了优化

## 扩展性

系统设计具有良好的扩展性：

1. **新增 Agent 类型**: 在 `get_default_prompt()` 函数中添加新类型
2. **自定义处理链**: 修改 `process_with_chain()` 函数添加新的处理链
3. **集成其他 AI 服务**: 修改 `call_openai_api()` 函数支持其他 AI 提供商

这个系统为 Spokenly Clone 提供了强大的 AI 文本处理能力，可以满足各种文本处理需求。