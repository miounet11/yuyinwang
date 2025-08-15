export interface RecordingSession {
  id: string;
  startTime: number;
  endTime?: number;
  duration: number;
  isActive: boolean;
  model?: string;
  device?: string;
}

export interface TimerEventData {
  sessionId: string;
  duration: number;
  isActive: boolean;
}

export class RecordingTimer {
  private currentSession: RecordingSession | null = null;
  private intervalId: NodeJS.Timeout | null = null;
  private listeners: ((data: TimerEventData) => void)[] = [];
  private updateInterval: number = 100; // 100ms 更新频率

  constructor(updateInterval: number = 100) {
    this.updateInterval = updateInterval;
  }

  // 开始录音计时
  public startRecording(model?: string, device?: string): string {
    if (this.currentSession?.isActive) {
      console.warn('录音会话已在进行中');
      return this.currentSession.id;
    }

    const sessionId = this.generateSessionId();
    const startTime = Date.now();

    this.currentSession = {
      id: sessionId,
      startTime,
      duration: 0,
      isActive: true,
      model,
      device
    };

    this.startTimer();
    console.log(`🎙️ 录音计时开始: ${sessionId}`);
    
    return sessionId;
  }

  // 停止录音计时
  public stopRecording(): RecordingSession | null {
    if (!this.currentSession?.isActive) {
      console.warn('没有活动的录音会话');
      return null;
    }

    const endTime = Date.now();
    const finalDuration = endTime - this.currentSession.startTime;

    this.currentSession.endTime = endTime;
    this.currentSession.duration = finalDuration;
    this.currentSession.isActive = false;

    this.stopTimer();

    // 最后一次通知
    this.notifyListeners({
      sessionId: this.currentSession.id,
      duration: finalDuration,
      isActive: false
    });

    console.log(`⏹️ 录音计时结束: ${this.currentSession.id}, 时长: ${(finalDuration / 1000).toFixed(1)}秒`);
    
    const completedSession = { ...this.currentSession };
    this.currentSession = null;
    
    return completedSession;
  }

  // 暂停录音计时
  public pauseRecording(): boolean {
    if (!this.currentSession?.isActive) {
      console.warn('没有活动的录音会话可暂停');
      return false;
    }

    this.stopTimer();
    console.log(`⏸️ 录音计时暂停: ${this.currentSession.id}`);
    return true;
  }

  // 恢复录音计时
  public resumeRecording(): boolean {
    if (!this.currentSession || this.currentSession.isActive) {
      console.warn('无法恢复录音计时');
      return false;
    }

    // 调整开始时间，减去已暂停的时长
    const pausedDuration = this.currentSession.duration;
    this.currentSession.startTime = Date.now() - pausedDuration;
    this.currentSession.isActive = true;

    this.startTimer();
    console.log(`▶️ 录音计时恢复: ${this.currentSession.id}`);
    return true;
  }

  // 获取当前录音时长（秒）
  public getCurrentDuration(): number {
    if (!this.currentSession) {
      return 0;
    }

    if (this.currentSession.isActive) {
      return (Date.now() - this.currentSession.startTime) / 1000;
    } else {
      return this.currentSession.duration / 1000;
    }
  }

  // 获取当前会话信息
  public getCurrentSession(): RecordingSession | null {
    return this.currentSession ? { ...this.currentSession } : null;
  }

  // 检查是否正在录音
  public isRecording(): boolean {
    return this.currentSession?.isActive || false;
  }

  // 添加时长更新监听器
  public addListener(listener: (data: TimerEventData) => void): () => void {
    this.listeners.push(listener);
    
    // 返回移除监听器的函数
    return () => {
      const index = this.listeners.indexOf(listener);
      if (index > -1) {
        this.listeners.splice(index, 1);
      }
    };
  }

  // 移除监听器
  public removeListener(listener: (data: TimerEventData) => void) {
    const index = this.listeners.indexOf(listener);
    if (index > -1) {
      this.listeners.splice(index, 1);
    }
  }

  // 清除所有监听器
  public clearListeners() {
    this.listeners = [];
  }

  // 格式化时长显示
  public static formatDuration(seconds: number): string {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = Math.floor(seconds % 60);
    const ms = Math.floor((seconds % 1) * 10);

    if (hours > 0) {
      return `${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
    } else {
      return `${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}.${ms}`;
    }
  }

  // 获取录音统计信息
  public getStats(): {
    isActive: boolean;
    currentDuration: number;
    sessionId: string | null;
    startTime: number | null;
    model: string | null;
    device: string | null;
  } {
    const session = this.currentSession;
    return {
      isActive: session?.isActive || false,
      currentDuration: this.getCurrentDuration(),
      sessionId: session?.id || null,
      startTime: session?.startTime || null,
      model: session?.model || null,
      device: session?.device || null
    };
  }

  // 设置更新频率
  public setUpdateInterval(interval: number) {
    this.updateInterval = Math.max(50, interval); // 最小50ms
    
    if (this.intervalId) {
      this.stopTimer();
      this.startTimer();
    }
  }

  // 私有方法：生成会话ID
  private generateSessionId(): string {
    return `recording_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  // 私有方法：开始定时器
  private startTimer() {
    if (this.intervalId) {
      return;
    }

    this.intervalId = setInterval(() => {
      this.updateCurrentDuration();
    }, this.updateInterval);
  }

  // 私有方法：停止定时器
  private stopTimer() {
    if (this.intervalId) {
      clearInterval(this.intervalId);
      this.intervalId = null;
    }
  }

  // 私有方法：更新当前时长
  private updateCurrentDuration() {
    if (!this.currentSession?.isActive) {
      return;
    }

    const currentTime = Date.now();
    const duration = currentTime - this.currentSession.startTime;
    this.currentSession.duration = duration;

    this.notifyListeners({
      sessionId: this.currentSession.id,
      duration: duration / 1000, // 转换为秒
      isActive: true
    });
  }

  // 私有方法：通知监听器
  private notifyListeners(data: TimerEventData) {
    this.listeners.forEach(listener => {
      try {
        listener(data);
      } catch (error) {
        console.error('录音计时器监听器执行失败:', error);
      }
    });
  }

  // 清理资源
  public cleanup() {
    this.stopTimer();
    this.clearListeners();
    this.currentSession = null;
    console.log('✅ 录音计时器已清理');
  }
}

// 创建全局录音计时器实例
export const recordingTimer = new RecordingTimer(100);