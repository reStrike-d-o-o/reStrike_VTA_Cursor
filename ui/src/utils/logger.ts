export type LogLevel = 'silent' | 'error' | 'warn' | 'info' | 'debug';

let currentLevel: LogLevel = 'info';

// Keep references to the original console methods so we can call them even after patching
const originalConsole = {
  error: console.error.bind(console),
  warn: console.warn.bind(console),
  info: console.info.bind(console),
  log: console.log.bind(console),
  debug: (console as any).debug ? (console as any).debug.bind(console) : console.log.bind(console),
};

export const setLogLevel = (level: LogLevel) => {
  currentLevel = level;
};

export const getLogLevel = (): LogLevel => currentLevel;

const shouldLog = (level: LogLevel) => {
  const order: LogLevel[] = ['silent', 'error', 'warn', 'info', 'debug'];
  return order.indexOf(level) <= order.indexOf(currentLevel);
};

// Patch global console to respect configured log level even for direct console.* calls
export const applyConsolePatch = () => {
  // console.log -> debug level (noisy)
  console.log = (...args: any[]) => {
    if (shouldLog('debug')) originalConsole.log(...args);
  };
  // console.info -> info level
  console.info = (...args: any[]) => {
    if (shouldLog('info')) originalConsole.info(...args);
  };
  // console.warn -> warn level
  console.warn = (...args: any[]) => {
    if (shouldLog('warn')) originalConsole.warn(...args);
  };
  // console.error -> error level
  console.error = (...args: any[]) => {
    if (shouldLog('error')) originalConsole.error(...args);
  };
  // console.debug -> debug level
  (console as any).debug = (...args: any[]) => {
    if (shouldLog('debug')) originalConsole.debug(...args);
  };
};

export const initLogLevelFromStorage = () => {
  try {
    const stored = typeof localStorage !== 'undefined' ? localStorage.getItem('logLevel') : null;
    if (stored === 'silent' || stored === 'error' || stored === 'warn' || stored === 'info' || stored === 'debug') {
      currentLevel = stored as LogLevel;
    }
  } catch {
    // ignore
  }
};

export const logger = {
  error: (...args: any[]) => shouldLog('error') && originalConsole.error(...args),
  warn: (...args: any[]) => shouldLog('warn') && originalConsole.warn(...args),
  info: (...args: any[]) => shouldLog('info') && originalConsole.info(...args),
  debug: (...args: any[]) => shouldLog('debug') && originalConsole.log(...args),
};


