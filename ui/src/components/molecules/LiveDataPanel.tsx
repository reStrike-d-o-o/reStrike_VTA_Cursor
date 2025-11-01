import React, { useEffect, useMemo, useRef, useState } from 'react';
import { useEnvironment } from '../../hooks/useEnvironment';
import { useLiveDataStore, LiveLogEntry } from '../../stores/liveDataStore';
import Toggle from '../atoms/Toggle';
import { useLiveDataEvents } from '../../hooks/useLiveDataEvents';
import { useI18n } from '../../i18n/index';

const invoke = async (command: string, args?: any) => {
  if (typeof window === 'undefined' || !window.__TAURI__?.core) {
    throw new Error('Tauri v2 core module not available - ensure app is running in desktop mode');
  }
  return window.__TAURI__.core.invoke(command, args);
};

const levelColors: Record<string, string> = {
  INFO: 'text-emerald-300',
  WARN: 'text-yellow-300',
  ERROR: 'text-red-400',
  DEBUG: 'text-sky-300',
  TRACE: 'text-slate-300',
};

const parseLogLine = (entry: LiveLogEntry) => {
  const pattern = /^\[(?<stamp>[^\]]+)\]\s+\[(?<level>[^\]]+)\]\s+-\s+(?<body>.*)$/;
  const match = pattern.exec(entry.message);

  if (match?.groups) {
    return {
      stamp: match.groups.stamp,
      level: match.groups.level.toUpperCase(),
      body: match.groups.body,
    };
  }

  return {
    stamp: new Date(entry.timestamp).toLocaleTimeString(),
    level: entry.level.toUpperCase(),
    body: entry.message,
  };
};

const LiveDataPanel: React.FC = () => {
  const { tauriAvailable } = useEnvironment();
  const logs = useLiveDataStore((state) => state.logs);
  const clearLogs = useLiveDataStore((state) => state.clearLogs);
  const { isConnected: wsConnected } = useLiveDataEvents();
  const { t } = useI18n();

  const liveDataRef = useRef<HTMLDivElement>(null);
  const [autoScroll, setAutoScroll] = useState(true);

  useEffect(() => {
    if (autoScroll && liveDataRef.current) {
      liveDataRef.current.scrollTop = liveDataRef.current.scrollHeight;
    }
  }, [logs, autoScroll]);

  const handleToggle = async () => {
    if (!tauriAvailable) {
      return;
    }

    try {
      const shouldEnable = !wsConnected;
      await invoke('set_live_data_streaming', {
        subsystem: 'pss',
        enabled: shouldEnable,
      });
      if (!shouldEnable) {
        clearLogs();
      }
    } catch (error) {
      console.error('Failed to toggle live data streaming:', error);
    }
  };

  const renderedLogs = useMemo(() => {
    return logs.map((entry) => {
      const parsed = parseLogLine(entry);
      const levelClass =
        levelColors[parsed.level] ?? levelColors[entry.level] ?? levelColors.INFO;

      return (
        <div
          key={entry.id}
          className="whitespace-pre leading-relaxed font-mono text-sm flex gap-2"
        >
          <span className="text-slate-500 min-w-[7.5rem]">{parsed.stamp}</span>
          <span className={`${levelClass} min-w-[3.5rem]`}>{`[${parsed.level}]`}</span>
          <span className="text-gray-200 flex-1">{parsed.body}</span>
        </div>
      );
    });
  }, [logs]);

  return (
    <div className="theme-card p-4 shadow-lg relative">
      <h3 className="text-lg font-semibold mb-2 text-blue-300">{t('live.title', 'LIVE DATA')}</h3>

      <div className="flex flex-wrap items-center justify-between gap-3 mb-3">
        <div className="flex items-center gap-3">
          <Toggle
            checked={wsConnected}
            onChange={handleToggle}
            label={t('live.feed_enabled', 'Feed Enabled')}
            labelPosition="right"
          />
          <span className="flex items-center gap-2 text-sm">
            <span
              className={`w-2 h-2 rounded-full ${
                wsConnected ? 'bg-green-500' : 'bg-red-500'
              }`}
            />
            <span className={wsConnected ? 'text-emerald-300' : 'text-red-300'}>
              {wsConnected ? t('common.connected', 'Connected') : t('common.disconnected', 'Disconnected')}
            </span>
          </span>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={clearLogs}
            className="px-2 py-1 text-xs rounded bg-gray-700 hover:bg-gray-600 text-gray-100"
          >
            {t('common.clear', 'Clear')}
          </button>
          <Toggle
            checked={autoScroll}
            onChange={() => setAutoScroll((prev) => !prev)}
            label={t('live.auto_scroll', 'Auto-scroll')}
            labelPosition="right"
          />
        </div>
      </div>

      <div
        ref={liveDataRef}
        className="bg-black/60 border border-gray-700 rounded p-3 h-64 overflow-y-auto space-y-1"
      >
        {logs.length === 0 ? (
          <div className="text-center text-gray-500 text-sm">
            {wsConnected
              ? t('live.waiting_logs', 'Waiting for live data ...')
              : t('live.feed_disabled', 'Live feed disabled. Enable to stream logs.')}
          </div>
        ) : (
          renderedLogs
        )}
      </div>

      <div className="mt-3 text-xs text-gray-400 flex items-center justify-between">
        <span>
          {t('live.lines', 'Lines')}: {logs.length}
        </span>
        <span>{t('live.auto_scroll_hint', 'Toggle auto-scroll to pause the feed')}</span>
      </div>
    </div>
  );
};

export default LiveDataPanel;
