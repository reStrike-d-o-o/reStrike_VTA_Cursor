import React from 'react';
import { StatusDot } from '../atoms/StatusDot';
import { usePssMatchStore } from '../../stores';
import { useAppStore } from '../../stores';

const StatusbarDock: React.FC = () => {
  const isPssDataLoaded = usePssMatchStore((state) => state.matchData.isLoaded);
  const obsConnections = useAppStore((state) => state.obsConnections);
  const obsHealth = useAppStore((state) => state.obsHealth);
  const pssStats = useAppStore((state) => state.pssStats);

  const recConnection = obsConnections.find((c) => c.name === 'OBS_REC');
  const strConnection = obsConnections.find((c) => c.name === 'OBS_STR');

  const recHealth = obsHealth['OBS_REC'];
  const strHealth = obsHealth['OBS_STR'];

  const recFps = recHealth?.active_fps ?? null;
  const recCpu = recHealth?.cpu_usage ?? null;
  const strCpu = strHealth?.cpu_usage ?? null;
  const strDropPct =
    strHealth && strHealth.total_frames > 0
      ? (strHealth.skipped_frames / strHealth.total_frames) * 100
      : null;

  const totalBytes = pssStats?.total_bytes_received ?? null;

  const isConnectionActive = (status?: string) =>
    status === 'Connected' || status === 'Authenticated';
  const isConnectionConnecting = (status?: string) =>
    status === 'Connecting' || status === 'Authenticating';

  const clampToFour = (value: string) => (value.length <= 4 ? value : '999+');

  const formatCompactPercent = (value: number | null) => {
    if (value === null || Number.isNaN(value)) return '--';
    const abs = Math.abs(value);

    const withDecimal = (() => {
      const raw = value.toFixed(1).replace(/\.0$/, '');
      return `${raw}%`;
    })();

    if (abs < 10 && withDecimal.length <= 4) {
      return withDecimal;
    }

    const rounded = `${Math.round(value)}%`;
    if (rounded.length <= 4) {
      return rounded;
    }

    return '999+';
  };

  const formatCompactFps = (value: number | null) => {
    if (value === null || Number.isNaN(value)) return '--';
    const rounded = Math.round(value);
    return clampToFour(rounded.toString());
  };

  const formatCompactBytes = (bytes: number | null) => {
    if (bytes === null || Number.isNaN(bytes)) return '--';
    const KB = 1024;
    const MB = KB * 1024;
    const GB = MB * 1024;

    const formatWithUnit = (unitValue: number, unit: string) => {
      const whole = Math.round(unitValue);
      let candidate = `${whole}${unit}`;
      if (candidate.length <= 4) return candidate;

      if (unitValue < 10) {
        const decimal = (Math.round(unitValue * 10) / 10).toFixed(1).replace(/\.0$/, '');
        candidate = `${decimal}${unit}`;
        if (candidate.length <= 4) return candidate;
      }

      candidate = `${whole}${unit[0]}`;
      if (candidate.length <= 4) return candidate;

      return '999+';
    };

    if (bytes >= GB) {
      return formatWithUnit(bytes / GB, 'GB');
    }
    if (bytes >= MB) {
      return formatWithUnit(bytes / MB, 'MB');
    }
    if (bytes > 0) {
      return formatWithUnit(bytes / KB, 'KB');
    }
    return '0KB';
  };

  const getCpuColor = (value: number | null) => {
    if (value === null) return 'bg-gray-500';
    if (value <= 30) return 'bg-green-500';
    if (value <= 70) return 'bg-yellow-500';
    return 'bg-red-500';
  };

  const cpuValues = [recCpu, strCpu].filter(
    (value): value is number => value !== null && !Number.isNaN(value),
  );
  const cpuPanelColor = getCpuColor(cpuValues.length ? Math.max(...cpuValues) : null);

  const buildConnectionStyles = (
    connection: typeof recConnection,
    palette: {
      active: { container: string; dot: string; shadow: string };
      transient: { container: string; dot: string; shadow: string };
      error: { container: string; dot: string; shadow: string };
      idle: { container: string; dot: string; shadow: string };
    },
  ) => {
    if (!connection) {
      return palette.idle;
    }

    if (isConnectionActive(connection.status)) {
      return palette.active;
    }

    if (isConnectionConnecting(connection.status)) {
      return palette.transient;
    }

    if (connection.status === 'Error') {
      return palette.error;
    }

    return palette.idle;
  };

  const recStyles = buildConnectionStyles(recConnection, {
    active: {
      container: 'bg-red-500/10 border-red-500/20',
      dot: 'bg-red-500',
      shadow: 'shadow-red-500/40',
    },
    transient: {
      container: 'bg-yellow-500/10 border-yellow-500/20',
      dot: 'bg-yellow-500',
      shadow: 'shadow-yellow-500/40 animate-pulse',
    },
    error: {
      container: 'bg-red-500/10 border-red-500/20',
      dot: 'bg-red-500',
      shadow: 'shadow-red-500/40',
    },
    idle: {
      container: 'bg-gray-500/10 border-gray-500/20',
      dot: 'bg-gray-500',
      shadow: 'shadow-gray-500/30',
    },
  });

  const strStyles = buildConnectionStyles(strConnection, {
    active: {
      container: 'bg-orange-500/10 border-orange-500/20',
      dot: 'bg-orange-500',
      shadow: 'shadow-orange-500/40',
    },
    transient: {
      container: 'bg-yellow-500/10 border-yellow-500/20',
      dot: 'bg-yellow-500',
      shadow: 'shadow-yellow-500/40 animate-pulse',
    },
    error: {
      container: 'bg-red-500/10 border-red-500/20',
      dot: 'bg-red-500',
      shadow: 'shadow-red-500/40',
    },
    idle: {
      container: 'bg-gray-500/10 border-gray-500/20',
      dot: 'bg-gray-500',
      shadow: 'shadow-gray-500/30',
    },
  });

  const pssStyles = isPssDataLoaded
    ? {
        container: 'bg-green-500/10 border-green-500/20',
        dot: 'bg-green-500',
        shadow: 'shadow-green-500/40',
      }
    : {
        container: 'bg-gray-500/10 border-gray-500/20',
        dot: 'bg-gray-500',
        shadow: 'shadow-gray-500/30',
      };

  const renderSingleValueCard = (
    label: string,
    value: string,
    styles: { container: string; dot: string; shadow: string },
  ) => (
    <div
      className={`relative flex flex-col justify-between min-w-[70px] px-3 py-2 rounded-lg border backdrop-blur-sm text-left pl-7 ${
        styles.container
      }`}
    >
      <StatusDot
        color={styles.dot}
        size="w-2.5 h-2.5"
        className={`absolute left-2 top-1/2 -translate-y-1/2 shadow-md ${styles.shadow}`}
      />
      <div className="flex flex-col gap-[2px] leading-tight">
        <span className="text-[0.65rem] font-semibold uppercase tracking-wide text-gray-100">
          {label}
        </span>
        <span className="text-[0.7rem] font-medium text-gray-100">{value}</span>
      </div>
    </div>
  );

  return (
    <div className="w-full h-[4.5rem] bg-gradient-to-r from-gray-800/80 to-gray-900/90 backdrop-blur-sm flex items-center justify-center text-xs text-gray-300 px-8 border-t border-gray-600/30">
      <div className="flex items-center space-x-4">
        {renderSingleValueCard('REC', formatCompactFps(recFps), recStyles)}
        {renderSingleValueCard('STR', formatCompactPercent(strDropPct), strStyles)}
        {renderSingleValueCard('PSS', formatCompactBytes(totalBytes), pssStyles)}

        {/* CPU panel */}
        <div
          className={`flex flex-col gap-1 px-3 py-2 rounded-lg border backdrop-blur-sm min-w-[70px] ${
            cpuPanelColor === 'bg-green-500'
              ? 'bg-green-500/10 border-green-500/20'
              : cpuPanelColor === 'bg-yellow-500'
              ? 'bg-yellow-500/10 border-yellow-500/20'
              : cpuPanelColor === 'bg-red-500'
              ? 'bg-red-500/10 border-red-500/20'
              : 'bg-gray-500/10 border-gray-500/20'
          }`}
        >
          <div className="flex items-center gap-2">
            <StatusDot
              color={getCpuColor(recCpu)}
              size="w-2.5 h-2.5"
              className="shadow-md"
            />
            <span className="text-[0.7rem] font-medium text-gray-100">
              R{formatCompactPercent(recCpu)}
            </span>
          </div>
          <div className="flex items-center gap-2">
            <StatusDot
              color={getCpuColor(strCpu)}
              size="w-2.5 h-2.5"
              className="shadow-md"
            />
            <span className="text-[0.7rem] font-medium text-gray-100">
              S{formatCompactPercent(strCpu)}
            </span>
          </div>
        </div>
      </div>
    </div>
  );
};

export default StatusbarDock;
