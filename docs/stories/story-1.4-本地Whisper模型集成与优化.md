# Story 1.4: 本地Whisper模型集成与优化

**故事ID**: STORY-1.4  
**史诗**: 史诗1 - Recording King 核心功能完善与体验优化  
**优先级**: 高 (核心MVP)  
**估算**: 12 Story Points  
**标签**: `本地模型` `Whisper集成` `离线转录` `GPU加速`  
**依赖**: Story 1.3 (实时语音转录引擎)

---

## 📝 用户故事

**作为一个** Recording King用户，  
**我希望** 能够使用本地Whisper模型进行离线转录，  
**这样我就能** 在没有网络或保护隐私时继续使用转录功能。

---

## 🎯 验收标准

### AC1: 多规模Whisper模型集成
- [ ] **Given** 用户访问模型管理界面
- [ ] **When** 查看可用的本地模型选项
- [ ] **Then** 支持Whisper Small/Medium/Large v3三种规模
- [ ] **And** 显示每种模型的性能指标（准确度、速度、文件大小）
- [ ] **And** 提供模型选择建议（根据设备性能）

### AC2: 模型下载与管理
- [ ] **Given** 用户选择下载本地模型
- [ ] **When** 启动模型下载过程
- [ ] **Then** 显示实时下载进度（百分比、速度、剩余时间）
- [ ] **And** 支持下载暂停、恢复和取消功能
- [ ] **And** 下载完成后自动验证模型完整性
- [ ] **And** 提供模型版本更新通知和管理

### AC3: GPU加速支持（Metal for macOS）
- [ ] **Given** 用户的设备支持Metal GPU加速
- [ ] **When** 使用本地Whisper模型进行转录
- [ ] **Then** 自动检测并启用Metal GPU加速
- [ ] **And** 转录速度比CPU模式提升50%以上
- [ ] **And** 显示当前使用的计算资源（CPU/GPU）
- [ ] **And** 提供手动GPU加速开关选项

### AC4: 本地/云端模式无缝切换
- [ ] **Given** 用户有可用的本地模型和云端API配置
- [ ] **When** 在模型选择界面进行切换
- [ ] **Then** 无需重启应用即可切换模式
- [ ] **And** 保持转录历史记录的连续性和一致性
- [ ] **And** 自动保存用户的模式偏好设置
- [ ] **And** 网络不可用时自动回退到本地模式

### AC5: 模型性能监控
- [ ] **Given** 用户正在使用本地Whisper模型
- [ ] **When** 进行转录任务时
- [ ] **Then** 实时显示转录性能指标：
  - 转录速度 (实时倍率)
  - 内存使用量
  - CPU/GPU使用率
  - 平均置信度
- [ ] **And** 提供性能对比功能（本地vs云端）
- [ ] **And** 记录性能历史数据用于优化建议

---

## 🔧 技术实现要求

### 当前系统分析

**已有基础设施**：
- ✅ `WhisperTranscriber` - Whisper模型封装和优化 [Source: src-tauri/src/transcription/whisper.rs]
- ✅ `ModelManagementSystem` - 完整的模型管理架构 [Source: docs/MODEL_MANAGEMENT_SYSTEM.md]
- ✅ `TranscriptionModel` 接口 - 统一的模型配置接口
- ✅ `LocalModelManager` UI组件 - 本地模型下载管理界面
- ⚠️ **问题**: 当前Whisper集成需要完善GPU加速和性能优化

**需要完善的部分**：
1. **Metal GPU加速**: 启用whisper-rs的Metal后端支持
2. **模型下载管理**: 实现断点续传和完整性验证
3. **性能监控**: 实时性能指标收集和展示
4. **模式切换**: 本地/云端无缝切换机制

### 核心组件实现

#### 1. Whisper模型集成优化
```rust
pub struct EnhancedWhisperTranscriber {
    // 增强的模型加载器，支持多模型并行
    model_loader: Arc<Mutex<ModelLoader>>,
    // Metal GPU加速管理器
    metal_accelerator: Option<MetalAccelerator>,
    // 性能监控器
    performance_monitor: Arc<Mutex<PerformanceMonitor>>,
    // 模型缓存管理器
    model_cache: Arc<Mutex<LRUCache<String, WhisperContext>>>,
}

impl EnhancedWhisperTranscriber {
    // 智能模型选择：根据设备性能和任务需求
    pub async fn select_optimal_model(&self, audio_duration: f64, quality_preference: QualityLevel) -> AppResult<String> {
        let device_info = self.get_device_capabilities().await?;
        
        match (device_info.metal_support, audio_duration, quality_preference) {
            (true, duration, QualityLevel::High) if duration > 300.0 => Ok("whisper-large-v3".to_string()),
            (true, duration, _) if duration > 60.0 => Ok("whisper-medium".to_string()),
            (_, _, _) => Ok("whisper-small".to_string()),
        }
    }
    
    // Metal加速转录
    pub async fn transcribe_with_metal(&self, audio_data: &[f32], model_id: &str) -> AppResult<TranscriptionResult> {
        let start_time = Instant::now();
        
        // 获取或加载模型
        let model = self.get_or_load_model(model_id).await?;
        
        // 启用Metal加速
        if let Some(ref accelerator) = self.metal_accelerator {
            accelerator.prepare_audio_buffer(audio_data)?;
        }
        
        // 执行转录，实时监控性能
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_n_threads(self.optimal_thread_count());
        
        if self.metal_accelerator.is_some() {
            params.set_use_gpu(true);
        }
        
        let result = model.full(params, audio_data)?;
        
        // 记录性能指标
        let elapsed = start_time.elapsed();
        self.performance_monitor.lock().record_transcription(
            model_id,
            audio_data.len() as f64 / 16000.0, // 音频时长
            elapsed,
            self.get_current_resource_usage(),
        );
        
        Ok(TranscriptionResult {
            text: result.get_segment_text(0),
            confidence: result.get_segment_prob(0),
            processing_time: elapsed,
            used_gpu: self.metal_accelerator.is_some(),
        })
    }
}
```

#### 2. 模型下载管理器
```rust
pub struct ModelDownloadManager {
    download_sessions: Arc<Mutex<HashMap<String, DownloadSession>>>,
    storage_manager: Arc<StorageManager>,
    event_emitter: Arc<dyn EventEmitter>,
}

impl ModelDownloadManager {
    // 启动模型下载（支持断点续传）
    pub async fn start_download(&self, model_id: &str, resume: bool) -> AppResult<DownloadHandle> {
        let model_info = self.get_model_info(model_id)?;
        let download_path = self.storage_manager.get_model_path(model_id);
        
        // 检查是否有部分下载的文件
        let existing_size = if resume && download_path.exists() {
            std::fs::metadata(&download_path)?.len()
        } else {
            0
        };
        
        let session = DownloadSession::new(
            model_id.to_string(),
            model_info.download_url.clone(),
            download_path,
            existing_size,
        );
        
        let handle = session.start().await?;
        
        // 监听下载进度
        let progress_handler = {
            let emitter = Arc::clone(&self.event_emitter);
            let model_id = model_id.to_string();
            
            move |progress: DownloadProgress| {
                let _ = emitter.emit("download_progress", json!({
                    "model_id": model_id,
                    "progress": progress.percentage,
                    "speed": progress.speed_mbps,
                    "eta": progress.eta_seconds,
                }));
            }
        };
        
        session.on_progress(progress_handler);
        
        Ok(handle)
    }
    
    // 验证下载的模型完整性
    pub async fn verify_model(&self, model_id: &str) -> AppResult<bool> {
        let model_path = self.storage_manager.get_model_path(model_id);
        let expected_hash = self.get_model_info(model_id)?.sha256_hash;
        
        let actual_hash = self.calculate_file_hash(&model_path).await?;
        
        Ok(actual_hash == expected_hash)
    }
}
```

#### 3. 性能监控与指标展示
```typescript
interface LocalModelPerformanceProps {
  modelId: string;
  isActive: boolean;
}

export const LocalModelPerformance: React.FC<LocalModelPerformanceProps> = ({
  modelId,
  isActive
}) => {
  const [metrics, setMetrics] = useState<PerformanceMetrics | null>(null);
  const [history, setHistory] = useState<PerformanceHistory[]>([]);
  
  // 实时性能监控
  useEffect(() => {
    if (!isActive) return;
    
    const unsubscribe = modelApi.subscribeToPerformanceMetrics(
      modelId,
      (newMetrics: PerformanceMetrics) => {
        setMetrics(newMetrics);
        setHistory(prev => [...prev.slice(-29), newMetrics]); // 保留最近30条记录
      }
    );
    
    return unsubscribe;
  }, [modelId, isActive]);
  
  return (
    <div className="performance-monitor">
      <div className="metrics-grid">
        <MetricCard
          title="转录速度"
          value={`${metrics?.speed_ratio || 0}x`}
          unit="实时倍率"
          trend={getTrend(history, 'speed_ratio')}
        />
        <MetricCard
          title="内存使用"
          value={`${metrics?.memory_usage_mb || 0}`}
          unit="MB"
          max={2048}
          trend={getTrend(history, 'memory_usage_mb')}
        />
        <MetricCard
          title="GPU使用率"
          value={`${metrics?.gpu_usage_percent || 0}`}
          unit="%"
          max={100}
          isGpuMetric={true}
        />
        <MetricCard
          title="平均置信度"
          value={`${((metrics?.avg_confidence || 0) * 100).toFixed(1)}`}
          unit="%"
          trend={getTrend(history, 'avg_confidence')}
        />
      </div>
      
      <PerformanceChart
        data={history}
        metrics={['speed_ratio', 'memory_usage_mb']}
        timeRange="30min"
      />
    </div>
  );
};
```

#### 4. 模式切换协调器
```rust
pub struct TranscriptionModeCoordinator {
    local_transcriber: Arc<EnhancedWhisperTranscriber>,
    cloud_transcriber: Arc<CloudTranscriber>,
    mode_preference: Arc<Mutex<TranscriptionMode>>,
    network_monitor: Arc<NetworkMonitor>,
}

impl TranscriptionModeCoordinator {
    // 智能模式选择
    pub async fn select_transcription_mode(&self, context: TranscriptionContext) -> TranscriptionMode {
        let current_preference = *self.mode_preference.lock();
        let network_available = self.network_monitor.is_connected().await;
        
        match (current_preference, network_available, context.privacy_mode) {
            (TranscriptionMode::CloudPreferred, true, false) => TranscriptionMode::Cloud,
            (TranscriptionMode::LocalPreferred, _, _) => TranscriptionMode::Local,
            (_, false, _) => TranscriptionMode::Local, // 网络不可用时强制本地
            (_, _, true) => TranscriptionMode::Local,   // 隐私模式强制本地
            _ => current_preference,
        }
    }
    
    // 无缝模式切换
    pub async fn switch_mode(&self, new_mode: TranscriptionMode) -> AppResult<()> {
        let old_mode = *self.mode_preference.lock();
        
        // 预热新模式
        match new_mode {
            TranscriptionMode::Local => {
                self.local_transcriber.preload_common_models().await?;
            }
            TranscriptionMode::Cloud => {
                self.cloud_transcriber.test_connection().await?;
            }
        }
        
        // 原子性切换
        *self.mode_preference.lock() = new_mode;
        
        // 触发UI更新事件
        self.emit_mode_change_event(old_mode, new_mode).await?;
        
        Ok(())
    }
}
```

### 数据库Schema扩展
```sql
-- 本地模型信息表
CREATE TABLE local_models (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    version TEXT NOT NULL,
    size_bytes INTEGER NOT NULL,
    download_status TEXT NOT NULL, -- 'not_downloaded', 'downloading', 'downloaded', 'corrupted'
    download_progress REAL DEFAULT 0.0,
    file_path TEXT,
    sha256_hash TEXT NOT NULL,
    metal_support BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 模型性能历史表
CREATE TABLE model_performance_history (
    id INTEGER PRIMARY KEY,
    model_id TEXT NOT NULL,
    session_id TEXT NOT NULL,
    transcription_duration_sec REAL NOT NULL,
    processing_time_ms INTEGER NOT NULL,
    speed_ratio REAL NOT NULL,
    memory_usage_mb INTEGER NOT NULL,
    gpu_usage_percent INTEGER,
    avg_confidence REAL NOT NULL,
    used_gpu BOOLEAN DEFAULT FALSE,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (model_id) REFERENCES local_models(id)
);

-- 模式切换历史表
CREATE TABLE transcription_mode_switches (
    id INTEGER PRIMARY KEY,
    from_mode TEXT NOT NULL,
    to_mode TEXT NOT NULL,
    switch_reason TEXT, -- 'user_manual', 'network_unavailable', 'privacy_mode', 'auto_optimization'
    context_info TEXT, -- JSON格式的上下文信息
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

---

## 🔗 集成验证标准

### IV1: 本地模式不影响现有云端API功能
- **验证目标**: 云端API转录功能保持完全正常
- **测试要求**:
  - 现有OpenAI、Deepgram等API配置继续工作
  - API响应时间和准确率不受影响
  - 云端模式下的所有现有功能正常

### IV2: 模型切换保持转录历史记录连续性
- **验证目标**: 不同模式的转录结果统一存储和管理
- **测试要求**:
  - 本地和云端转录结果格式一致
  - 历史记录查看和搜索功能正常
  - 转录结果标注包含使用的模型信息

### IV3: 本地模式性能符合系统资源要求
- **验证目标**: 本地模型运行不影响系统整体性能
- **测试要求**:
  - 内存使用增长<30%
  - CPU使用率峰值<80%
  - GPU使用不影响其他应用
  - 长时间运行稳定性>99%

---

## 🔄 回滚验收标准

### RB1: 本地Whisper模型可完全移除
- **回滚目标**: 恢复纯云端转录模式
- **验证标准**:
  - 本地模型文件完全清理
  - 相关配置和缓存清除
  - 系统恢复到安装前状态

### RB2: 回滚后现有AI模型配置和历史保持不变
- **回滚目标**: 用户数据和设置完整保留
- **验证标准**:
  - 云端API配置完全恢复
  - 转录历史记录无损失
  - 用户偏好设置保持不变

### RB3: 模型文件清理不影响系统其他部分
- **回滚目标**: 系统其他功能完全正常
- **验证标准**:
  - 音频录制功能正常
  - 快捷键和权限系统正常
  - UI界面和用户体验无变化

---

## 📋 任务分解

### Phase 1: 基础模型集成 (4 SP)
- [ ] 1.4.1 升级whisper-rs依赖，启用Metal GPU加速支持
- [ ] 1.4.2 实现ModelDownloadManager模型下载管理器
- [ ] 1.4.3 集成三种规模Whisper模型（Small/Medium/Large v3）
- [ ] 1.4.4 实现模型完整性验证和错误恢复机制

### Phase 2: 性能优化与监控 (3 SP)  
- [ ] 1.4.5 实现Metal GPU加速和自动检测
- [ ] 1.4.6 创建PerformanceMonitor性能监控系统
- [ ] 1.4.7 实现智能模型选择算法
- [ ] 1.4.8 优化模型加载和内存管理

### Phase 3: 模式切换与UI集成 (3 SP)
- [ ] 1.4.9 实现TranscriptionModeCoordinator模式协调器
- [ ] 1.4.10 创建本地模型管理UI界面
- [ ] 1.4.11 实现实时性能监控显示组件
- [ ] 1.4.12 集成模式切换用户界面

### Phase 4: 测试与优化 (2 SP)
- [ ] 1.4.13 性能基准测试和优化调整
- [ ] 1.4.14 集成测试和兼容性验证
- [ ] 1.4.15 用户体验测试和界面优化
- [ ] 1.4.16 文档更新和回滚测试

---

## 🎬 Demo场景

### 场景1: 首次设置本地模型
1. 用户打开模型管理界面
2. 系统显示推荐的模型规格（基于设备性能）
3. 用户选择Whisper Medium模型下载
4. 显示下载进度：进度条、速度、预估时间
5. 下载完成，自动验证文件完整性
6. 提示用户可以开始使用本地模式

### 场景2: GPU加速转录体验
1. 用户启动语音转录（本地模式）
2. 系统自动检测到Metal GPU支持
3. 显示"GPU加速已启用"提示
4. 转录过程中显示实时性能指标
5. 完成转录，显示性能对比：GPU vs CPU模式
6. 转录速度达到3.5x实时处理

### 场景3: 智能模式切换
1. 用户在有网络的环境下使用云端模式
2. 网络连接突然中断
3. 系统检测到网络异常，自动切换到本地模式
4. 无缝继续转录，用户几乎无感知
5. 网络恢复后，询问用户是否切换回云端
6. 所有转录历史记录保持连续

### 场景4: 隐私保护模式
1. 用户启用隐私保护模式
2. 系统自动切换到本地Whisper模型
3. 显示"隐私模式：所有处理在本地完成"
4. 转录过程中显示"无数据上传"状态
5. 完成转录，确认所有数据仅存储在本地

---

## 📊 完成标准

### 功能完成度
- [ ] 三种Whisper模型可正常下载和使用
- [ ] Metal GPU加速功能正常工作
- [ ] 本地/云端模式切换成功率>99%
- [ ] 模型下载支持断点续传和完整性验证

### 性能指标
- [ ] 本地转录速度：GPU模式>3x实时，CPU模式>1.5x实时
- [ ] 模型加载时间：Small<10秒，Medium<30秒，Large<60秒
- [ ] 内存使用：Small<1GB，Medium<2GB，Large<4GB
- [ ] GPU加速提升：转录速度提升>50%

### 质量标准
- [ ] 本地模型转录准确率：与云端API差异<5%
- [ ] 系统稳定性：连续运行>4小时无异常
- [ ] 模式切换延迟<3秒
- [ ] 网络异常自动切换成功率>95%

### 用户体验
- [ ] 模型下载过程直观清晰
- [ ] 性能指标实时显示准确
- [ ] 模式切换操作简单直观
- [ ] 隐私保护状态明确可见

---

## 🧪 测试标准

### 单元测试要求
- [ ] WhisperTranscriber核心功能测试 >90% 覆盖率
- [ ] ModelDownloadManager下载逻辑测试
- [ ] PerformanceMonitor指标收集测试
- [ ] TranscriptionModeCoordinator切换逻辑测试

### 集成测试要求
- [ ] 本地模型端到端转录流程测试
- [ ] 模式切换集成测试
- [ ] 性能监控数据流测试
- [ ] GPU加速功能测试

### 性能测试要求
- [ ] 不同规模模型性能基准测试
- [ ] GPU vs CPU性能对比测试
- [ ] 长时间运行稳定性测试
- [ ] 内存泄漏和资源使用测试

---

**Story状态**: 🟡 Draft  
**分配给**: 待分配  
**创建日期**: 2025-09-17  
**更新日期**: 2025-09-17

---

## 📚 Dev Notes

### 技术上下文

**数据模型和API** [Source: docs/MODEL_MANAGEMENT_SYSTEM.md]:
- `TranscriptionModel` 接口已定义，支持本地模型配置
- `LocalModelInfo` 结构包含下载状态和路径信息
- `ModelDownloadManager` UI组件已实现基础下载管理

**现有Whisper集成** [Source: src-tauri/src/transcription/whisper.rs]:
- `WhisperTranscriber` 已实现基础模型缓存和性能优化
- 支持模型预加载和哈希验证
- 已集成`PerformanceOptimizer`进行性能监控

**音频处理架构** [Source: docs/architecture.md]:
- 现有音频处理流程与Whisper集成兼容
- 需要扩展`RealtimeAudioStreamer`支持本地模型
- 保持与现有权限管理系统的集成

**UI组件系统** [Source: docs/MODEL_MANAGEMENT_SYSTEM.md]:
- `LocalModelManager.tsx` 组件已实现下载管理界面
- 支持下载进度追踪和状态管理
- 遵循现有React + TailwindCSS设计系统

### 关键实现细节

**Metal GPU加速集成**:
- 需要在Cargo.toml中启用whisper-rs的metal特性
- 使用`params.set_use_gpu(true)`启用GPU加速
- 实现设备兼容性检测和性能监控

**模型下载管理**:
- 实现断点续传机制，支持网络中断恢复
- 使用SHA256哈希验证模型文件完整性
- 提供下载速度和进度的实时反馈

**性能监控系统**:
- 收集转录速度、内存使用、GPU使用率等指标
- 实现历史数据存储和趋势分析
- 提供性能对比功能（本地vs云端）

**模式切换协调**:
- 实现网络状态监控和自动切换逻辑
- 保持转录历史记录的连续性
- 支持用户手动模式选择和偏好保存

### 集成要求

**数据库迁移**:
- 新增local_models表存储模型信息
- 新增model_performance_history表记录性能数据
- 扩展现有配置表支持模式偏好设置

**API接口扩展**:
- 保持现有Tauri命令接口兼容性
- 新增本地模型管理相关命令
- 扩展转录接口支持模式参数

**UI组件集成**:
- 扩展现有模型管理界面
- 新增性能监控显示组件
- 保持与现有设计系统一致性

### 测试策略

**单元测试重点**:
- Whisper模型加载和转录逻辑
- 模型下载和验证功能
- 性能监控数据收集准确性

**集成测试重点**:
- 端到端本地转录流程
- 模式切换的无缝性验证
- 与现有系统的兼容性测试

**性能测试重点**:
- 不同模型规格的性能基准
- GPU加速效果验证
- 长时间运行稳定性测试

---

*本Story将为Recording King提供完整的本地Whisper模型支持，实现真正的离线转录能力，同时通过智能模式切换确保最佳的用户体验和隐私保护。*