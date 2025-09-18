import React, { createContext, useContext, useRef, useCallback, useEffect } from 'react';
import './InteractionFeedbackProvider.css';

interface InteractionMetrics {
  responseTime: number;
  interactionType: string;
  timestamp: number;
  elementId?: string;
}

interface InteractionFeedbackContextType {
  recordInteraction: (type: string, elementId?: string) => Promise<number>;
  getMetrics: () => InteractionMetrics[];
  clearMetrics: () => void;
  isHighPerformance: boolean;
}

const InteractionFeedbackContext = createContext<InteractionFeedbackContextType | null>(null);

interface InteractionFeedbackProviderProps {
  children: React.ReactNode;
  /** 性能目标（毫秒） */
  performanceTarget?: number;
  /** 是否启用ripple效果 */
  enableRipple?: boolean;
  /** 是否启用hover反馈 */
  enableHover?: boolean;
  /** 是否启用性能监控 */
  enableMetrics?: boolean;
  /** 性能警告回调 */
  onPerformanceWarning?: (metrics: InteractionMetrics) => void;
}

export function InteractionFeedbackProvider({
  children,
  performanceTarget = 100,
  enableRipple = true,
  enableHover = true,
  enableMetrics = true,
  onPerformanceWarning
}: InteractionFeedbackProviderProps) {
  const metricsRef = useRef<InteractionMetrics[]>([]);
  const rippleContainerRef = useRef<HTMLDivElement>(null);
  const interactionStartTimeRef = useRef<Map<string, number>>(new Map());

  // 记录交互开始时间和测量响应时间
  const recordInteraction = useCallback(async (type: string, elementId?: string): Promise<number> => {
    const startTime = performance.now();
    const interactionKey = `${type}-${elementId || 'unknown'}-${startTime}`;

    interactionStartTimeRef.current.set(interactionKey, startTime);

    // 使用requestAnimationFrame确保在下一帧测量响应时间
    return new Promise((resolve) => {
      requestAnimationFrame(() => {
        const endTime = performance.now();
        const responseTime = endTime - startTime;

        const metrics: InteractionMetrics = {
          responseTime,
          interactionType: type,
          timestamp: Date.now(),
          elementId
        };

        if (enableMetrics) {
          metricsRef.current.push(metrics);

          // 保持最近1000个交互记录
          if (metricsRef.current.length > 1000) {
            metricsRef.current = metricsRef.current.slice(-1000);
          }
        }

        // 性能警告
        if (responseTime > performanceTarget) {
          onPerformanceWarning?.(metrics);
          console.warn(`UI响应时间超过目标: ${responseTime.toFixed(2)}ms > ${performanceTarget}ms`, metrics);
        }

        interactionStartTimeRef.current.delete(interactionKey);
        resolve(responseTime);
      });
    });
  }, [performanceTarget, enableMetrics, onPerformanceWarning]);

  // 获取性能指标
  const getMetrics = useCallback(() => {
    return [...metricsRef.current];
  }, []);

  // 清除指标
  const clearMetrics = useCallback(() => {
    metricsRef.current = [];
  }, []);

  // 计算是否为高性能状态
  const isHighPerformance = metricsRef.current.length > 0
    ? metricsRef.current.slice(-10).every(m => m.responseTime <= performanceTarget)
    : true;

  // 创建ripple效果
  const createRipple = useCallback((event: MouseEvent | TouchEvent) => {
    if (!enableRipple || !rippleContainerRef.current) return;

    const target = event.target as HTMLElement;
    if (!target || target.closest('.no-ripple')) return;

    const rect = target.getBoundingClientRect();
    const size = Math.max(rect.width, rect.height);
    const x = (event as MouseEvent).clientX - rect.left - size / 2;
    const y = (event as MouseEvent).clientY - rect.top - size / 2;

    const ripple = document.createElement('div');
    ripple.className = 'interaction-ripple';
    ripple.style.cssText = `
      position: absolute;
      left: ${x}px;
      top: ${y}px;
      width: ${size}px;
      height: ${size}px;
      border-radius: 50%;
      background: rgba(255, 255, 255, 0.3);
      transform: scale(0);
      animation: ripple-animation 0.6s ease-out;
      pointer-events: none;
      z-index: 9999;
    `;

    // 将ripple添加到目标元素或容器
    const container = target.style.position === 'relative' || target.style.position === 'absolute'
      ? target
      : rippleContainerRef.current;

    if (container) {
      container.appendChild(ripple);

      // 清理ripple
      setTimeout(() => {
        if (ripple.parentNode) {
          ripple.parentNode.removeChild(ripple);
        }
      }, 600);
    }
  }, [enableRipple]);

  // 全局点击处理
  useEffect(() => {
    const handleGlobalClick = (event: MouseEvent) => {
      const target = event.target as HTMLElement;
      const elementId = target.id || target.className || 'unnamed-element';

      // 记录交互
      recordInteraction('click', elementId);

      // 创建ripple效果
      createRipple(event);
    };

    const handleGlobalTouch = (event: TouchEvent) => {
      const target = event.target as HTMLElement;
      const elementId = target.id || target.className || 'unnamed-element';

      recordInteraction('touch', elementId);

      if (event.touches.length > 0) {
        createRipple(event);
      }
    };

    if (enableRipple) {
      document.addEventListener('click', handleGlobalClick, { passive: true });
      document.addEventListener('touchstart', handleGlobalTouch, { passive: true });
    }

    return () => {
      document.removeEventListener('click', handleGlobalClick);
      document.removeEventListener('touchstart', handleGlobalTouch);
    };
  }, [recordInteraction, createRipple, enableRipple]);

  // Hover效果处理
  useEffect(() => {
    if (!enableHover) return;

    const handleMouseOver = (event: MouseEvent) => {
      const target = event.target as HTMLElement;
      if (target.matches('button, [role="button"], a, input, select, textarea, .hoverable')) {
        target.classList.add('interaction-hover');
        recordInteraction('hover', target.id || target.className);
      }
    };

    const handleMouseOut = (event: MouseEvent) => {
      const target = event.target as HTMLElement;
      target.classList.remove('interaction-hover');
    };

    document.addEventListener('mouseover', handleMouseOver, { passive: true });
    document.addEventListener('mouseout', handleMouseOut, { passive: true });

    return () => {
      document.removeEventListener('mouseover', handleMouseOver);
      document.removeEventListener('mouseout', handleMouseOut);
    };
  }, [enableHover, recordInteraction]);

  const contextValue: InteractionFeedbackContextType = {
    recordInteraction,
    getMetrics,
    clearMetrics,
    isHighPerformance
  };

  return (
    <InteractionFeedbackContext.Provider value={contextValue}>
      <div
        className={`interaction-feedback-provider ${isHighPerformance ? 'high-performance' : 'low-performance'}`}
        ref={rippleContainerRef}
      >
        {children}
      </div>
    </InteractionFeedbackContext.Provider>
  );
}

// Hook for using interaction feedback
export function useInteractionFeedback() {
  const context = useContext(InteractionFeedbackContext);
  if (!context) {
    throw new Error('useInteractionFeedback must be used within an InteractionFeedbackProvider');
  }
  return context;
}

// HOC for enhanced interactive components
export function withInteractionFeedback<T extends {}>(
  Component: React.ComponentType<T>,
  options?: {
    trackHover?: boolean;
    trackFocus?: boolean;
    customMetrics?: (props: T) => Record<string, any>;
  }
) {
  return React.forwardRef<any, T>((props, ref) => {
    const { recordInteraction } = useInteractionFeedback();
    const elementRef = useRef<HTMLElement>(null);

    useEffect(() => {
      const element = elementRef.current;
      if (!element) return;

      const handlers: { [key: string]: (event: Event) => void } = {};

      // Track clicks
      handlers.click = (event) => {
        recordInteraction('click', element.id || 'enhanced-component');
      };

      // Track hover if enabled
      if (options?.trackHover) {
        handlers.mouseenter = () => {
          recordInteraction('hover-enter', element.id || 'enhanced-component');
        };
      }

      // Track focus if enabled
      if (options?.trackFocus) {
        handlers.focus = () => {
          recordInteraction('focus', element.id || 'enhanced-component');
        };
      }

      // Add event listeners
      Object.entries(handlers).forEach(([event, handler]) => {
        element.addEventListener(event, handler, { passive: true });
      });

      return () => {
        Object.entries(handlers).forEach(([event, handler]) => {
          element.removeEventListener(event, handler);
        });
      };
    }, [recordInteraction]);

    return (
      <Component
        {...props}
        ref={(node: HTMLElement) => {
          elementRef.current = node;
          if (typeof ref === 'function') {
            ref(node);
          } else if (ref) {
            ref.current = node;
          }
        }}
        className={`${(props as any).className || ''} interaction-enhanced`}
      />
    );
  });
}

// Performance monitoring component
export function InteractionPerformanceMonitor() {
  const { getMetrics, clearMetrics, isHighPerformance } = useInteractionFeedback();
  const [showMonitor, setShowMonitor] = React.useState(false);
  const [metrics, setMetrics] = React.useState<InteractionMetrics[]>([]);

  useEffect(() => {
    const updateMetrics = () => {
      setMetrics(getMetrics());
    };

    const interval = setInterval(updateMetrics, 1000);
    return () => clearInterval(interval);
  }, [getMetrics]);

  if (process.env.NODE_ENV !== 'development') {
    return null;
  }

  const recentMetrics = metrics.slice(-10);
  const averageResponseTime = recentMetrics.length > 0
    ? recentMetrics.reduce((sum, m) => sum + m.responseTime, 0) / recentMetrics.length
    : 0;

  return (
    <div className="interaction-performance-monitor">
      <button
        className="monitor-toggle"
        onClick={() => setShowMonitor(!showMonitor)}
        title="交互性能监控"
      >
        📊 {averageResponseTime.toFixed(1)}ms
      </button>

      {showMonitor && (
        <div className="monitor-panel">
          <div className="monitor-header">
            <h3>交互性能监控</h3>
            <button onClick={clearMetrics}>清除</button>
          </div>

          <div className="monitor-stats">
            <div className={`stat ${isHighPerformance ? 'good' : 'warning'}`}>
              <label>平均响应时间:</label>
              <span>{averageResponseTime.toFixed(2)}ms</span>
            </div>
            <div className="stat">
              <label>总交互次数:</label>
              <span>{metrics.length}</span>
            </div>
            <div className="stat">
              <label>性能状态:</label>
              <span className={isHighPerformance ? 'good' : 'warning'}>
                {isHighPerformance ? '优秀' : '需要优化'}
              </span>
            </div>
          </div>

          <div className="recent-interactions">
            <h4>最近交互:</h4>
            <ul>
              {recentMetrics.slice(-5).reverse().map((metric, index) => (
                <li key={index} className={metric.responseTime > 100 ? 'slow' : 'fast'}>
                  <span className="type">{metric.interactionType}</span>
                  <span className="time">{metric.responseTime.toFixed(2)}ms</span>
                  <span className="element">{metric.elementId}</span>
                </li>
              ))}
            </ul>
          </div>
        </div>
      )}
    </div>
  );
}
