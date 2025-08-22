/**
 * Performance Monitor for Recording King Voice Input
 * Monitors animation performance and voice input latency
 */

interface PerformanceMetrics {
  animationFrameRate: number;
  voiceInputLatency: number;
  memoryUsage: number;
  lastMeasurement: number;
}

class PerformanceMonitor {
  private frameCount: number = 0;
  private lastFrameTime: number = performance.now();
  private frameRateHistory: number[] = [];
  private latencyHistory: number[] = [];
  private isMonitoring: boolean = false;
  private metrics: PerformanceMetrics = {
    animationFrameRate: 60,
    voiceInputLatency: 0,
    memoryUsage: 0,
    lastMeasurement: performance.now(),
  };

  constructor() {
    this.measureFrame = this.measureFrame.bind(this);
    this.startMonitoring = this.startMonitoring.bind(this);
    this.stopMonitoring = this.stopMonitoring.bind(this);
  }

  private measureFrame(): void {
    if (!this.isMonitoring) return;

    const now = performance.now();
    const deltaTime = now - this.lastFrameTime;
    
    if (deltaTime > 0) {
      const currentFps = 1000 / deltaTime;
      this.frameRateHistory.push(currentFps);
      
      // Keep only last 60 frames for rolling average
      if (this.frameRateHistory.length > 60) {
        this.frameRateHistory.shift();
      }
      
      // Calculate average FPS
      this.metrics.animationFrameRate = this.frameRateHistory.reduce((sum, fps) => sum + fps, 0) / this.frameRateHistory.length;
    }
    
    this.lastFrameTime = now;
    this.frameCount++;

    // Measure memory if available
    if ('memory' in performance) {
      const memInfo = (performance as any).memory;
      this.metrics.memoryUsage = memInfo.usedJSHeapSize / (1024 * 1024); // MB
    }

    requestAnimationFrame(this.measureFrame);
  }

  public startMonitoring(): void {
    if (this.isMonitoring) return;
    
    this.isMonitoring = true;
    this.frameCount = 0;
    this.frameRateHistory = [];
    this.lastFrameTime = performance.now();
    requestAnimationFrame(this.measureFrame);
    
    console.log('🎯 Performance monitoring started');
  }

  public stopMonitoring(): void {
    this.isMonitoring = false;
    console.log('🛑 Performance monitoring stopped');
  }

  public measureVoiceInputLatency(startTime: number): number {
    const latency = performance.now() - startTime;
    this.latencyHistory.push(latency);
    
    // Keep only last 10 measurements
    if (this.latencyHistory.length > 10) {
      this.latencyHistory.shift();
    }
    
    this.metrics.voiceInputLatency = this.latencyHistory.reduce((sum, lat) => sum + lat, 0) / this.latencyHistory.length;
    
    // Warn if latency exceeds 100ms requirement
    if (latency > 100) {
      console.warn(`⚠️ Voice input latency exceeded 100ms: ${latency.toFixed(2)}ms`);
    }
    
    return latency;
  }

  public getMetrics(): PerformanceMetrics {
    return {
      ...this.metrics,
      lastMeasurement: performance.now(),
    };
  }

  public isPerformanceGood(): boolean {
    const metrics = this.getMetrics();
    return metrics.animationFrameRate >= 50 && metrics.voiceInputLatency <= 100;
  }

  public getPerformanceReport(): string {
    const metrics = this.getMetrics();
    const memoryDisplay = metrics.memoryUsage > 0 ? `${metrics.memoryUsage.toFixed(2)}MB` : 'N/A';
    
    return `
📊 Performance Report:
• Animation FPS: ${metrics.animationFrameRate.toFixed(1)}
• Voice Latency: ${metrics.voiceInputLatency.toFixed(2)}ms
• Memory Usage: ${memoryDisplay}
• Status: ${this.isPerformanceGood() ? '✅ Good' : '⚠️ Needs Attention'}
    `.trim();
  }

  public logPerformance(): void {
    console.log(this.getPerformanceReport());
  }
}

// Global performance monitor instance
export const performanceMonitor = new PerformanceMonitor();

// Utility functions for easy performance measurement
export const measureVoiceInputLatency = (startTime: number): number => {
  return performanceMonitor.measureVoiceInputLatency(startTime);
};

export const startPerformanceMonitoring = (): void => {
  performanceMonitor.startMonitoring();
};

export const stopPerformanceMonitoring = (): void => {
  performanceMonitor.stopMonitoring();
};

export const getPerformanceMetrics = (): PerformanceMetrics => {
  return performanceMonitor.getMetrics();
};

export const isPerformanceGood = (): boolean => {
  return performanceMonitor.isPerformanceGood();
};

export const logPerformanceReport = (): void => {
  performanceMonitor.logPerformance();
};

export default performanceMonitor;