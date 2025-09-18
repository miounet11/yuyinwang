# Story 1.1: 权限管理系统统一与优化

**故事ID**: STORY-1.1  
**史诗**: 史诗1 - Recording King 核心功能完善与体验优化  
**优先级**: 高 (核心MVP)  
**估算**: 8 Story Points  
**标签**: `权限管理` `系统重构` `用户体验`

---

## 📝 用户故事

**作为一个** Recording King用户，  
**我希望** 应用能够统一管理所有必需权限并提供清晰的引导流程，  
**这样我就能** 快速完成应用设置并确保所有功能正常工作。

---

## 🎯 验收标准

### AC1: 权限状态检测
- [x] **Given** 用户启动Recording King应用
- [x] **When** 应用完成初始化
- [x] **Then** 自动检测以下权限状态：
  - 麦克风访问权限
  - 辅助功能权限
  - 输入监控权限
  - 录音设备权限
- [x] **And** 权限状态在<2秒内完成检测并显示

### AC2: 统一权限引导
- [x] **Given** 检测到缺失的权限
- [x] **When** 用户查看权限状态页面
- [x] **Then** 显示统一的权限引导向导，包含：
  - 每个权限的作用说明
  - 清晰的步骤指导
  - 一键跳转系统设置按钮
  - 权限配置完成确认
- [x] **And** 支持跳过可选权限的配置

### AC3: 实时权限状态更新
- [x] **Given** 用户在系统设置中修改了权限
- [x] **When** 用户返回Recording King应用
- [x] **Then** 权限状态自动更新，无需重启应用
- [x] **And** UI状态在<1秒内反映最新权限状态

### AC4: 降级功能支持
- [x] **Given** 用户拒绝授予某些权限
- [x] **When** 用户尝试使用需要权限的功能
- [x] **Then** 提供清晰的降级功能说明：
  - 说明哪些功能将受限
  - 提供替代操作方案
  - 显示重新授权的入口
- [x] **And** 应用仍能正常运行基础功能

### AC5: 权限配置持久化
- [x] **Given** 用户已完成权限配置
- [x] **When** 应用重新启动
- [x] **Then** 记住权限引导完成状态
- [x] **And** 不重复显示已确认的权限引导

---

## 🔧 技术实现要求

### 技术债务解决
- **统一三套快捷键管理器**:
  - `shortcuts::ShortcutManager` 
  - `commands::shortcut_management::ShortcutManager`
  - `shortcuts::EnhancedShortcutManager`
- **实现** `UnifiedPermissionManager` 架构
- **建立** 中央权限状态存储

### 核心组件开发

#### 1. UnifiedPermissionManager (Rust)
```rust
pub struct UnifiedPermissionManager {
    permission_state: Arc<RwLock<PermissionState>>,
    permission_checkers: HashMap<PermissionType, Box<dyn PermissionChecker>>,
    state_listeners: Vec<Box<dyn PermissionStateListener>>,
}

impl UnifiedPermissionManager {
    pub async fn check_all_permissions() -> PermissionReport;
    pub async fn request_permission(permission: PermissionType) -> Result<bool>;
    pub fn register_state_listener(listener: Box<dyn PermissionStateListener>);
    pub fn get_permission_guidance(permission: PermissionType) -> GuidanceInfo;
}
```

#### 2. PermissionWizard (React组件)
- 权限引导向导UI
- 步骤式权限配置流程
- 实时状态更新
- 降级功能说明

#### 3. PermissionState Store (Zustand)
```typescript
interface PermissionState {
  permissions: Record<PermissionType, PermissionStatus>;
  wizardCompleted: boolean;
  checkingInProgress: boolean;
  updatePermissionStatus: (type: PermissionType, status: PermissionStatus) => void;
  markWizardCompleted: () => void;
}
```

### 数据库Schema扩展
```sql
CREATE TABLE permission_states (
    id INTEGER PRIMARY KEY,
    permission_type TEXT NOT NULL,
    status TEXT NOT NULL,
    granted_at TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE permission_wizard_progress (
    id INTEGER PRIMARY KEY,
    wizard_completed BOOLEAN DEFAULT FALSE,
    completed_at TIMESTAMP,
    skipped_permissions TEXT, -- JSON array
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

---

## 🔗 集成验证标准

### IV1: 现有功能兼容性
- **验证目标**: 现有权限检查功能保持正常，无功能回归
- **测试要求**:
  - 所有现有Tauri权限命令正常工作
  - 现有录音功能不受影响
  - 历史权限配置完整迁移

### IV2: API兼容性
- **验证目标**: 新权限系统与现有Tauri命令无冲突
- **测试要求**:
  - 现有`check_microphone_permission`命令继续工作
  - 新增`unified_permission_check`命令可用
  - API响应格式保持一致

### IV3: 核心功能完整性
- **验证目标**: 权限状态变更不影响现有录音和转录功能
- **测试要求**:
  - 权限状态检测不阻塞录音启动
  - 转录功能在权限变更时保持稳定
  - 快捷键功能与新权限系统协同工作

---

## 🔄 回滚验收标准

### RB1: 配置回滚能力
- **回滚目标**: 权限配置可一键回滚至系统重构前状态
- **验证标准**:
  - 提供`rollback_to_legacy_permissions()`命令
  - 回滚操作<30秒完成
  - 回滚后三套独立管理器重新激活

### RB2: 用户设置保护
- **回滚目标**: 回滚过程中现有用户权限设置保持不变
- **验证标准**:
  - 用户的macOS系统权限设置不受影响
  - 现有快捷键配置完整保留
  - 个人偏好设置无损迁移

### RB3: 功能完整性恢复
- **回滚目标**: 回滚完成后原有权限检查功能完全恢复
- **验证标准**:
  - 所有原有权限检查逻辑重新激活
  - 录音、转录、快捷键功能100%恢复
  - 系统性能回到重构前水平

---

## 📋 任务分解

### Phase 1: 架构重构 (3 SP)
- [ ] 1.1.1 设计UnifiedPermissionManager架构
- [ ] 1.1.2 实现权限检测器抽象接口
- [ ] 1.1.3 整合现有三套快捷键管理器
- [ ] 1.1.4 建立中央权限状态存储

### Phase 2: UI组件开发 (2 SP)
- [ ] 1.1.5 创建PermissionWizard组件
- [ ] 1.1.6 实现权限状态实时更新UI
- [ ] 1.1.7 设计降级功能提示界面
- [ ] 1.1.8 添加一键跳转系统设置功能

### Phase 3: 数据持久化 (2 SP)
- [ ] 1.1.9 扩展SQLite数据库schema
- [ ] 1.1.10 实现权限状态持久化
- [ ] 1.1.11 添加权限引导进度追踪
- [ ] 1.1.12 创建数据迁移脚本

### Phase 4: 集成测试 (1 SP)
- [ ] 1.1.13 集成验证测试
- [ ] 1.1.14 回滚功能测试
- [ ] 1.1.15 性能影响评估
- [ ] 1.1.16 用户体验测试

---

## 🎬 Demo场景

### 场景1: 首次使用用户
1. 用户启动Recording King
2. 应用检测权限状态（2秒内完成）
3. 显示权限引导向导
4. 用户跟随引导完成权限配置
5. 引导状态持久化，下次启动不再显示

### 场景2: 权限被撤销的用户
1. 用户启动应用，发现录音功能不可用
2. 应用自动检测到麦克风权限被撤销
3. 显示权限状态和重新授权引导
4. 用户重新授权后，应用立即恢复功能

### 场景3: 部分权限用户
1. 用户拒绝辅助功能权限
2. 应用提供降级功能说明
3. 快捷键功能受限，但录音功能正常
4. 提供重新授权的明确入口

---

## 📊 完成标准

### 功能完成度
- [ ] 所有验收标准100%通过
- [ ] 集成验证标准全部满足
- [ ] 回滚验收标准验证通过
- [ ] 性能要求达标（权限检查<2秒，UI响应<1秒）

### 代码质量
- [ ] 代码覆盖率>85%
- [ ] 所有静态分析检查通过
- [ ] Rust unsafe代码审查通过
- [ ] TypeScript类型检查无错误

### 文档完整性
- [ ] API文档更新完成
- [ ] 架构决策记录(ADR)创建
- [ ] 用户手册权限部分更新
- [ ] 开发者文档更新

---

**Story状态**: 🟡 Ready for Development  
**分配给**: 待分配  
**创建日期**: 2025-09-17  
**更新日期**: 2025-09-17

---

*这个故事是Recording King增强项目的基础，解决了系统的核心权限管理问题。完成后将为后续所有功能增强奠定稳固的基础。*