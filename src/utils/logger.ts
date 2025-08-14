// 生产环境日志管理器
type LogLevel = 'debug' | 'info' | 'warn' | 'error';

interface LogConfig {
  level: LogLevel;
  enabled: boolean;
  debugMode: boolean;
}

class Logger {
  private config: LogConfig;

  constructor() {
    this.config = {
      level: 'info',
      enabled: import.meta.env.DEV || import.meta.env.VITE_DEBUG_MODE === 'true',
      debugMode: import.meta.env.VITE_DEBUG_MODE === 'true'
    };
  }

  debug(message: string, ...args: any[]): void {
    if (this.shouldLog('debug')) {
      console.log(`🔍 [DEBUG] ${message}`, ...args);
    }
  }

  info(message: string, ...args: any[]): void {
    if (this.shouldLog('info')) {
      console.info(`ℹ️ [INFO] ${message}`, ...args);
    }
  }

  warn(message: string, ...args: any[]): void {
    if (this.shouldLog('warn')) {
      console.warn(`⚠️ [WARN] ${message}`, ...args);
    }
  }

  error(message: string, error?: Error, ...args: any[]): void {
    if (this.shouldLog('error')) {
      console.error(`❌ [ERROR] ${message}`, error, ...args);
    }
  }

  // 业务特定日志方法
  audio(message: string, ...args: any[]): void {
    if (this.config.debugMode) {
      console.log(`🎤 [AUDIO] ${message}`, ...args);
    }
  }

  transcription(message: string, ...args: any[]): void {
    if (this.config.debugMode) {
      console.log(`📝 [TRANSCRIPTION] ${message}`, ...args);
    }
  }

  ai(message: string, ...args: any[]): void {
    if (this.config.debugMode) {
      console.log(`🤖 [AI] ${message}`, ...args);
    }
  }

  api(message: string, ...args: any[]): void {
    if (this.config.debugMode) {
      console.log(`🌐 [API] ${message}`, ...args);
    }
  }

  private shouldLog(level: LogLevel): boolean {
    if (!this.config.enabled) return false;
    
    const levels = ['debug', 'info', 'warn', 'error'];
    const currentLevelIndex = levels.indexOf(this.config.level);
    const messageLevelIndex = levels.indexOf(level);
    
    return messageLevelIndex >= currentLevelIndex;
  }

  // 配置方法
  setLevel(level: LogLevel): void {
    this.config.level = level;
  }

  setEnabled(enabled: boolean): void {
    this.config.enabled = enabled;
  }

  setDebugMode(debug: boolean): void {
    this.config.debugMode = debug;
  }
}

// 导出单例
export const logger = new Logger();
export default logger;