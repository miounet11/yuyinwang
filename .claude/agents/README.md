# Claude Code Agents

这个目录包含了专门为 Claude Code 设计的 Agent 配置文件。

## Agent 列表

### 1. Memory Network Builder
- **文件**: `memory-network-builder.md`
- **功能**: 构建和维护知识网络系统，用于记录技术决策、学习心得和实现笔记
- **用途**: 当需要记录重要的技术发现、架构决策或性能优化经验时使用

### 2. Library Usage Researcher
- **文件**: `library-usage-researcher.md`
- **功能**: 系统性地研究库、框架和技术的使用方法
- **用途**: 深入了解某个技术栈的最佳实践、API细节和真实案例

### 3. Product Manager
- **文件**: `product-manager.md`
- **功能**: 理解用户痛点、定义产品需求、规划功能优先级
- **用途**: 当需要从产品角度分析需求或规划开发时使用

### 4. UI/UX Interaction Designer
- **文件**: `ui-ux-designer.md`
- **功能**: 创建直观交互流程，确保优秀用户体验
- **用途**: 当需要设计界面或优化交互时使用

### 5. Software Developer
- **文件**: `software-developer.md`
- **功能**: 编写高效代码，实现复杂功能，确保质量
- **用途**: 当需要技术实现或代码审查时使用

### 6. macOS Software Engineer
- **文件**: `macos-software-engineer.md`
- **功能**: 处理macOS特定开发，熟悉Rust和TypeScript，与其他agents协作
- **用途**: 当需要macOS集成、权限管理或系统优化时使用

## 使用前提

### Library Usage Researcher 依赖配置

要使用 `library-usage-researcher` Agent，需要先安装以下两个 MCP（Model Context Protocol）服务：

#### 1. Context7 MCP
用于获取官方文档和 API 规范：
```bash
claude mcp add --transport http context7 https://mcp.context7.com/mcp
```

#### 2. Grep MCP  
用于搜索 GitHub 上的真实代码案例：
```bash
claude mcp add --transport http grep https://mcp.grep.app
```

### 安装验证

安装完成后，可以进入 claude code 后输入下方命令验证是否正确安装：
/mcp

确保列表中包含 `context7` 和 `grep` 两个服务。

## 使用方法

1. 将对应的 Agent 配置文件内容复制到你的 Claude Code 配置中
2. 确保已安装必要的 MCP 服务（如上所述）
3. 在对话中明确指出需要使用特定的 Agent 功能

## 注意事项

- 这些 Agent 配置是为 Claude Code 专门优化的
- 不同的 Agent 可能需要不同的 MCP 服务支持
- 使用前请确保理解每个 Agent 的具体功能和适用场景
