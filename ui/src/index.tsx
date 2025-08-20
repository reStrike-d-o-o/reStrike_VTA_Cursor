import React from 'react';
import ReactDOM from 'react-dom/client';
import './index.css';
import App from './App';
import { initLogLevelFromStorage, applyConsolePatch } from './utils/logger';
import { diagLogsCommands } from './utils/tauriCommands';
import { useMessageCenter } from './stores/messageCenter';
import { I18nProvider } from './i18n';

// Initialize logging level and patch console globally before rendering
initLogLevelFromStorage();
applyConsolePatch();

// On startup, attempt to perform scheduled auto-archive if enabled in config
(async () => {
  try {
    const cfgResp: any = await diagLogsCommands.getAutoArchiveConfig();
    const config = cfgResp?.data;
    if (cfgResp?.success && config?.enabled) {
      const statusResp: any = await diagLogsCommands.checkAutoArchiveStatus(config);
      const shouldArchive = statusResp?.success && statusResp?.data?.should_archive;
      if (shouldArchive) {
        const runResp: any = await diagLogsCommands.performAutoArchive(config);
        if (runResp?.success) {
          const msg = runResp?.message || 'Logs archived successfully';
          try { useMessageCenter.getState().showSuccess('Auto-archive completed', msg); } catch {}
        }
      }
    }
  } catch {
    // ignore startup archive errors
  }
})();

// Performance optimizations for development
if (process.env.NODE_ENV === 'development') {
  // Disable React.StrictMode in development for faster renders
  const root = ReactDOM.createRoot(document.getElementById('root') as HTMLElement);
  root.render(
    <I18nProvider>
      <App />
    </I18nProvider>
  );
} else {
  const root = ReactDOM.createRoot(document.getElementById('root') as HTMLElement);
  root.render(
    <React.StrictMode>
      <I18nProvider>
        <App />
      </I18nProvider>
    </React.StrictMode>
  );
}