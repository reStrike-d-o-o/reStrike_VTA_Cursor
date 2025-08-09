export type LogLevel = 'silent' | 'error' | 'warn' | 'info' | 'debug';

let currentLevel: LogLevel = 'info';

export const setLogLevel = (level: LogLevel) => {
  currentLevel = level;
};

const shouldLog = (level: LogLevel) => {
  const order: LogLevel[] = ['silent', 'error', 'warn', 'info', 'debug'];
  return order.indexOf(level) <= order.indexOf(currentLevel);
};

export const logger = {
  error: (...args: any[]) => shouldLog('error') && console.error(...args),
  warn: (...args: any[]) => shouldLog('warn') && console.warn(...args),
  info: (...args: any[]) => shouldLog('info') && console.info(...args),
  debug: (...args: any[]) => shouldLog('debug') && console.log(...args),
};


