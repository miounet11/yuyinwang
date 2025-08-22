/**
 * Performance Optimization Utilities
 * Advanced performance monitoring and optimization for Recording King voice input animations
 */

interface PerformanceMetrics {
  fps: number;
  frameTime: number;
  memoryUsage: number;
  audioLatency: number;
  animationCount: number;
  renderTime: number;
  lastUpdate: number;
}

interface OptimizationConfig {
  targetFPS: number;
  maxAnimationCount: number;
  memoryThreshold: number;
  audioLatencyThreshold: number;
  enableAdaptiveQuality: boolean;
  enableDebugOverlay: boolean;
}

class PerformanceOptimizer {
  private metrics: PerformanceMetrics;
  private config: OptimizationConfig;
  private frameHistory: number[] = [];
  private animationRegistry: Set<string> = new Set();
  private rafId: number | null = null;
  private isMonitoring = false;
  private callbacks: Set<(metrics: PerformanceMetrics) => void> = new Set();

  constructor(config: Partial<OptimizationConfig> = {}) {
    this.config = {
      targetFPS: 60,
      maxAnimationCount: 20,
      memoryThreshold: 100, // MB
      audioLatencyThreshold: 100, // ms
      enableAdaptiveQuality: true,
      enableDebugOverlay: false,
      ...config,
    };

    this.metrics = {
      fps: 0,
      frameTime: 0,
      memoryUsage: 0,
      audioLatency: 0,
      animationCount: 0,
      renderTime: 0,
      lastUpdate: performance.now(),
    };

    this.setupMemoryMonitoring();
  }

  // Start performance monitoring
  startMonitoring(): void {
    if (this.isMonitoring) return;
    
    this.isMonitoring = true;
    this.updateMetrics();
    
    if (this.config.enableDebugOverlay) {
      this.createDebugOverlay();
    }
  }

  // Stop performance monitoring
  stopMonitoring(): void {
    this.isMonitoring = false;
    if (this.rafId) {
      cancelAnimationFrame(this.rafId);
      this.rafId = null;
    }
    this.removeDebugOverlay();
  }

  // Register animation for tracking
  registerAnimation(id: string): void {
    this.animationRegistry.add(id);
    this.metrics.animationCount = this.animationRegistry.size;
    
    if (this.metrics.animationCount > this.config.maxAnimationCount) {
      console.warn(`Animation count (${this.metrics.animationCount}) exceeds threshold (${this.config.maxAnimationCount})`);
      this.optimizeAnimations();
    }
  }

  // Unregister animation
  unregisterAnimation(id: string): void {
    this.animationRegistry.delete(id);
    this.metrics.animationCount = this.animationRegistry.size;
  }

  // Add performance metrics callback
  onMetricsUpdate(callback: (metrics: PerformanceMetrics) => void): () => void {
    this.callbacks.add(callback);
    return () => this.callbacks.delete(callback);
  }

  // Update audio latency metric
  updateAudioLatency(latency: number): void {
    this.metrics.audioLatency = latency;
    
    if (latency > this.config.audioLatencyThreshold) {
      console.warn(`Audio latency (${latency}ms) exceeds threshold (${this.config.audioLatencyThreshold}ms)`);
      this.optimizeAudioProcessing();
    }
  }

  // Get current metrics
  getMetrics(): PerformanceMetrics {
    return { ...this.metrics };
  }

  // Get optimization recommendations
  getOptimizationRecommendations(): string[] {
    const recommendations: string[] = [];
    
    if (this.metrics.fps < this.config.targetFPS * 0.8) {
      recommendations.push('Reduce animation complexity or count');
    }
    
    if (this.metrics.memoryUsage > this.config.memoryThreshold) {
      recommendations.push('Optimize memory usage - consider cleanup');
    }
    
    if (this.metrics.audioLatency > this.config.audioLatencyThreshold) {
      recommendations.push('Optimize audio processing pipeline');
    }
    
    if (this.metrics.animationCount > this.config.maxAnimationCount) {
      recommendations.push('Reduce concurrent animations');
    }
    
    return recommendations;
  }

  private updateMetrics(): void {
    if (!this.isMonitoring) return;

    const now = performance.now();
    const deltaTime = now - this.metrics.lastUpdate;
    
    // Update frame time and FPS
    this.frameHistory.push(deltaTime);
    if (this.frameHistory.length > 60) {
      this.frameHistory.shift();
    }
    
    const avgFrameTime = this.frameHistory.reduce((a, b) => a + b, 0) / this.frameHistory.length;
    this.metrics.frameTime = avgFrameTime;
    this.metrics.fps = Math.round(1000 / avgFrameTime);
    this.metrics.lastUpdate = now;

    // Update memory usage
    this.updateMemoryMetrics();

    // Notify callbacks
    this.callbacks.forEach(callback => callback(this.metrics));

    // Adaptive quality adjustment
    if (this.config.enableAdaptiveQuality) {
      this.adjustQuality();
    }

    this.rafId = requestAnimationFrame(() => this.updateMetrics());
  }

  private setupMemoryMonitoring(): void {
    if ('memory' in performance) {
      setInterval(() => {
        this.updateMemoryMetrics();
      }, 1000);
    }
  }

  private updateMemoryMetrics(): void {
    if ('memory' in performance) {
      const memory = (performance as any).memory;
      this.metrics.memoryUsage = Math.round(memory.usedJSHeapSize / 1024 / 1024);
    }
  }

  private optimizeAnimations(): void {
    // Implement animation optimization strategies
    document.documentElement.style.setProperty('--animation-duration-scale', '0.5');
    
    // Disable complex animations on low-performance devices
    if (this.metrics.fps < 30) {
      document.documentElement.classList.add('low-performance-mode');
    }
  }

  private optimizeAudioProcessing(): void {
    // Reduce audio visualization complexity
    const audioCanvases = document.querySelectorAll('.audio-canvas');
    audioCanvases.forEach(canvas => {
      canvas.classList.add('optimized-rendering');
    });
    
    // Reduce update frequency
    document.documentElement.style.setProperty('--audio-update-frequency', '30fps');
  }

  private adjustQuality(): void {
    const performanceRatio = this.metrics.fps / this.config.targetFPS;
    
    if (performanceRatio < 0.8) {
      // Reduce quality
      document.documentElement.style.setProperty('--animation-quality', 'low');
      document.documentElement.style.setProperty('--particle-count-scale', '0.5');
    } else if (performanceRatio > 1.1) {
      // Increase quality
      document.documentElement.style.setProperty('--animation-quality', 'high');
      document.documentElement.style.setProperty('--particle-count-scale', '1.0');
    }
  }

  private createDebugOverlay(): void {
    if (document.getElementById('performance-debug-overlay')) return;

    const overlay = document.createElement('div');
    overlay.id = 'performance-debug-overlay';
    overlay.style.cssText = `
      position: fixed;
      top: 10px;
      right: 10px;
      background: rgba(0, 0, 0, 0.8);
      color: white;
      padding: 10px;
      border-radius: 8px;
      font-family: monospace;
      font-size: 12px;
      line-height: 1.4;
      z-index: 10000;
      pointer-events: none;
      min-width: 200px;
    `;

    document.body.appendChild(overlay);

    // Update overlay content
    const updateOverlay = () => {
      if (!this.isMonitoring) return;
      
      const recommendations = this.getOptimizationRecommendations();
      
      overlay.innerHTML = `
        <div><strong>Performance Monitor</strong></div>
        <div>FPS: ${this.metrics.fps} / ${this.config.targetFPS}</div>
        <div>Frame Time: ${this.metrics.frameTime.toFixed(1)}ms</div>
        <div>Memory: ${this.metrics.memoryUsage}MB</div>
        <div>Audio Latency: ${this.metrics.audioLatency}ms</div>
        <div>Animations: ${this.metrics.animationCount}</div>
        ${recommendations.length > 0 ? `
          <div style="margin-top: 8px; color: #fbbf24;">
            <strong>Recommendations:</strong>
            ${recommendations.map(rec => `<div>• ${rec}</div>`).join('')}
          </div>
        ` : ''}
      `;
    };

    this.onMetricsUpdate(updateOverlay);
    updateOverlay();
  }

  private removeDebugOverlay(): void {
    const overlay = document.getElementById('performance-debug-overlay');
    if (overlay) {
      overlay.remove();
    }
  }
}

// Utility functions for performance optimization
export const performanceUtils = {
  // Throttle function calls for performance
  throttle<T extends (...args: any[]) => any>(
    func: T,
    delay: number
  ): (...args: Parameters<T>) => void {
    let timeoutId: NodeJS.Timeout;
    let lastExecTime = 0;
    
    return (...args: Parameters<T>) => {
      const currentTime = Date.now();
      
      if (currentTime - lastExecTime > delay) {
        func(...args);
        lastExecTime = currentTime;
      } else {
        clearTimeout(timeoutId);
        timeoutId = setTimeout(() => {
          func(...args);
          lastExecTime = Date.now();
        }, delay - (currentTime - lastExecTime));
      }
    };
  },

  // Debounce function calls
  debounce<T extends (...args: any[]) => any>(
    func: T,
    delay: number
  ): (...args: Parameters<T>) => void {
    let timeoutId: NodeJS.Timeout;
    
    return (...args: Parameters<T>) => {
      clearTimeout(timeoutId);
      timeoutId = setTimeout(() => func(...args), delay);
    };
  },

  // Check if device has good performance characteristics
  isHighPerformanceDevice(): boolean {
    const canvas = document.createElement('canvas');
    const gl = canvas.getContext('webgl') || canvas.getContext('experimental-webgl');
    
    if (!gl) return false;
    
    const debugInfo = gl.getExtension('WEBGL_debug_renderer_info');
    const renderer = debugInfo ? gl.getParameter(debugInfo.UNMASKED_RENDERER_WEBGL) : '';
    
    // Simple heuristic for high-performance devices
    return renderer.includes('Apple') || renderer.includes('NVIDIA') || renderer.includes('AMD');
  },

  // Get optimal animation settings based on device
  getOptimalAnimationSettings(): Partial<OptimizationConfig> {
    const isHighPerf = this.isHighPerformanceDevice();
    const isMobile = /Android|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(navigator.userAgent);
    
    return {
      targetFPS: isHighPerf ? 60 : 30,
      maxAnimationCount: isHighPerf ? 20 : 10,
      enableAdaptiveQuality: !isHighPerf || isMobile,
      enableDebugOverlay: process.env.NODE_ENV === 'development',
    };
  },

  // Preload critical resources
  preloadCriticalResources(): Promise<void[]> {
    const criticalResources = [
      // Add paths to critical audio/animation assets
      '/audio/notification.wav',
      '/images/waveform-pattern.svg',
    ];

    return Promise.all(
      criticalResources.map(url => {
        return new Promise<void>((resolve, reject) => {
          if (url.endsWith('.wav') || url.endsWith('.mp3')) {
            const audio = new Audio();
            audio.oncanplaythrough = () => resolve();
            audio.onerror = reject;
            audio.src = url;
          } else {
            const img = new Image();
            img.onload = () => resolve();
            img.onerror = reject;
            img.src = url;
          }
        });
      })
    );
  },

  // Cleanup unused animations and resources
  cleanupResources(): void {
    // Cancel any orphaned animation frames
    const highestId = setTimeout(() => {}, 0);
    for (let i = 0; i < highestId; i++) {
      clearTimeout(i);
      cancelAnimationFrame(i);
    }

    // Force garbage collection if available
    if ('gc' in window && typeof (window as any).gc === 'function') {
      (window as any).gc();
    }
  },
};

import React from 'react';

// React hook for performance monitoring
export const usePerformanceOptimizer = (config?: Partial<OptimizationConfig>) => {
  const [optimizer] = React.useState(() => new PerformanceOptimizer({
    ...performanceUtils.getOptimalAnimationSettings(),
    ...config,
  }));
  
  const [metrics, setMetrics] = React.useState<PerformanceMetrics>(optimizer.getMetrics());

  React.useEffect(() => {
    optimizer.startMonitoring();
    const unsubscribe = optimizer.onMetricsUpdate(setMetrics);

    return () => {
      optimizer.stopMonitoring();
      unsubscribe();
    };
  }, [optimizer]);

  return {
    metrics,
    optimizer,
    registerAnimation: optimizer.registerAnimation.bind(optimizer),
    unregisterAnimation: optimizer.unregisterAnimation.bind(optimizer),
    updateAudioLatency: optimizer.updateAudioLatency.bind(optimizer),
    getRecommendations: optimizer.getOptimizationRecommendations.bind(optimizer),
  };
};

export default PerformanceOptimizer;