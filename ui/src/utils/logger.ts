// Comprehensive Logging System for reStrike VTA
// This system provides centralized logging with file output and automatic cleanup

export enum LogLevel {
  DEBUG = 0,
  INFO = 1,
  WARN = 2,
  ERROR = 3,
  CRITICAL = 4
}

export interface LogEntry {
  timestamp: string;
  level: LogLevel;
  message: string;
  data?: any;
  environment: string;
  component?: string;
  error?: Error;
}

class Logger {
  private static instance: Logger;
  private logBuffer: LogEntry[] = [];
  private maxBufferSize = 1000;
  private logLevel: LogLevel = LogLevel.INFO;
  private environment: string = 'unknown';
  private logFileName = 'restrike-vta.log';

  private constructor() {
    this.environment = this.detectEnvironment();
    this.cleanupLogFile();
    this.setupGlobalErrorHandling();
  }

  public static getInstance(): Logger {
    if (!Logger.instance) {
      Logger.instance = new Logger();
    }
    return Logger.instance;
  }

  private detectEnvironment(): string {
    if (typeof window !== 'undefined' && (window as any).__TAURI__) {
      return 'windows';
    }
    if (typeof process !== 'undefined' && process.env.REACT_APP_ENVIRONMENT) {
      return process.env.REACT_APP_ENVIRONMENT;
    }
    return 'web';
  }

  private cleanupLogFile(): void {
    try {
      // In a real implementation, this would clear the log file
      // For now, we'll just log the cleanup action
      console.log(`[LOGGER] Log file cleanup initiated for environment: ${this.environment}`);
    } catch (error) {
      console.error('[LOGGER] Failed to cleanup log file:', error);
    }
  }

  private setupGlobalErrorHandling(): void {
    // Capture unhandled promise rejections
    if (typeof window !== 'undefined') {
      window.addEventListener('unhandledrejection', (event) => {
        this.error('Unhandled Promise Rejection', {
          reason: event.reason,
          promise: event.promise
        });
      });

      // Capture global errors
      window.addEventListener('error', (event) => {
        this.error('Global Error', {
          message: event.message,
          filename: event.filename,
          lineno: event.lineno,
          colno: event.colno,
          error: event.error
        });
      });
    }
  }

  private formatLogEntry(entry: LogEntry): string {
    const levelStr = LogLevel[entry.level];
    const componentStr = entry.component ? `[${entry.component}]` : '';
    const dataStr = entry.data ? ` | Data: ${JSON.stringify(entry.data)}` : '';
    const errorStr = entry.error ? ` | Error: ${entry.error.message}` : '';
    
    return `[${entry.timestamp}] [${levelStr}] [${entry.environment}]${componentStr} ${entry.message}${dataStr}${errorStr}`;
  }

  private writeToFile(entry: LogEntry): void {
    try {
      const logLine = this.formatLogEntry(entry);
      
      // In a real implementation, this would write to a file
      // For now, we'll use console and store in buffer
      console.log(logLine);
      
      // Store in buffer for potential file writing
      this.logBuffer.push(entry);
      
      // Keep buffer size manageable
      if (this.logBuffer.length > this.maxBufferSize) {
        this.logBuffer = this.logBuffer.slice(-this.maxBufferSize);
      }
    } catch (error) {
      console.error('[LOGGER] Failed to write log entry:', error);
    }
  }

  public setLogLevel(level: LogLevel): void {
    this.logLevel = level;
    this.info(`Log level set to: ${LogLevel[level]}`);
  }

  public debug(message: string, data?: any, component?: string): void {
    if (this.logLevel <= LogLevel.DEBUG) {
      this.writeToFile({
        timestamp: new Date().toISOString(),
        level: LogLevel.DEBUG,
        message,
        data,
        environment: this.environment,
        component
      });
    }
  }

  public info(message: string, data?: any, component?: string): void {
    if (this.logLevel <= LogLevel.INFO) {
      this.writeToFile({
        timestamp: new Date().toISOString(),
        level: LogLevel.INFO,
        message,
        data,
        environment: this.environment,
        component
      });
    }
  }

  public warn(message: string, data?: any, component?: string): void {
    if (this.logLevel <= LogLevel.WARN) {
      this.writeToFile({
        timestamp: new Date().toISOString(),
        level: LogLevel.WARN,
        message,
        data,
        environment: this.environment,
        component
      });
    }
  }

  public error(message: string, data?: any, component?: string, error?: Error): void {
    if (this.logLevel <= LogLevel.ERROR) {
      this.writeToFile({
        timestamp: new Date().toISOString(),
        level: LogLevel.ERROR,
        message,
        data,
        environment: this.environment,
        component,
        error
      });
    }
  }

  public critical(message: string, data?: any, component?: string, error?: Error): void {
    if (this.logLevel <= LogLevel.CRITICAL) {
      this.writeToFile({
        timestamp: new Date().toISOString(),
        level: LogLevel.CRITICAL,
        message,
        data,
        environment: this.environment,
        component,
        error
      });
    }
  }

  public getLogBuffer(): LogEntry[] {
    return [...this.logBuffer];
  }

  public clearLogBuffer(): void {
    this.logBuffer = [];
    this.info('Log buffer cleared');
  }

  public exportLogs(): string {
    return this.logBuffer.map(entry => this.formatLogEntry(entry)).join('\n');
  }
}

// Export singleton instance
export const logger = Logger.getInstance();

// Convenience functions for common use cases
export const logDebug = (message: string, data?: any, component?: string) => 
  logger.debug(message, data, component);

export const logInfo = (message: string, data?: any, component?: string) => 
  logger.info(message, data, component);

export const logWarn = (message: string, data?: any, component?: string) => 
  logger.warn(message, data, component);

export const logError = (message: string, data?: any, component?: string, error?: Error) => 
  logger.error(message, data, component, error);

export const logCritical = (message: string, data?: any, component?: string, error?: Error) => 
  logger.critical(message, data, component, error);

// Environment-aware logging that integrates with the existing environment system
export const createComponentLogger = (componentName: string) => ({
  debug: (message: string, data?: any) => logger.debug(message, data, componentName),
  info: (message: string, data?: any) => logger.info(message, data, componentName),
  warn: (message: string, data?: any) => logger.warn(message, data, componentName),
  error: (message: string, data?: any, error?: Error) => logger.error(message, data, componentName, error),
  critical: (message: string, data?: any, error?: Error) => logger.critical(message, data, componentName, error),
}); 