import React, { useEffect, useMemo, useState } from 'react';
import Button from '../atoms/Button';
import StatusDot from '../atoms/StatusDot';
import StatusRow from '../atoms/StatusRow';
import { invoke } from '@tauri-apps/api/core';
import { obsObwsCommands } from '../../utils/tauriCommandsObws';

type CheckStatus = 'unknown' | 'ok' | 'warn' | 'error';

interface LiveOrchestratorModalProps {
  isOpen: boolean;
  onClose: () => void;
  onStarted: () => void;
}

const LiveOrchestratorModal: React.FC<LiveOrchestratorModalProps> = ({ isOpen, onClose, onStarted }) => {
  const [runningChecks, setRunningChecks] = useState(false);
  const [udpStatus, setUdpStatus] = useState<CheckStatus>('unknown');
  const [tournamentStatus, setTournamentStatus] = useState<CheckStatus>('unknown');
  const [recordingPathStatus, setRecordingPathStatus] = useState<CheckStatus>('unknown');
  const [obsStatus, setObsStatus] = useState<CheckStatus>('unknown');
  const [driveStatus, setDriveStatus] = useState<CheckStatus>('unknown');
  const [message, setMessage] = useState<string>('');

  const colorFor = (s: CheckStatus) => {
    switch (s) {
      case 'ok': return 'green';
      case 'warn': return 'yellow';
      case 'error': return 'red';
      default: return 'gray';
    }
  };

  const checkDiskSpace = async (dir: string) => {
    try {
      const res = await invoke<any>('fs_get_disk_free_space', { directory: dir });
      // Expect res = { success, freeBytes, totalBytes }
      if (res?.success && typeof res.freeBytes === 'number' && typeof res.totalBytes === 'number') {
        const pct = (res.freeBytes / res.totalBytes) * 100;
        return pct > 10 ? 'ok' : 'warn';
      }
    } catch {}
    return 'warn';
  };

  const runChecks = async () => {
    setRunningChecks(true);
    setMessage('Running system checks...');
    try {
      // UDP server status
      try {
        const status = await invoke<string>('get_udp_status');
        setUdpStatus(status && status.includes('Running') ? 'ok' : 'warn');
      } catch {
        setUdpStatus('error');
      }

      // Tournament and day status
      try {
        const active = await invoke<any>('tournament_get_active');
        if (active && active.success && active.tournament) {
          const tournamentId = active.tournament.id as number;
          const daysResult = await invoke<any>('tournament_get_days', { tournamentId });
          if (daysResult && daysResult.success) {
            const today = new Date();
            const matchDay = (daysResult.days as any[]).find(d => {
              const dDate = new Date(d.date);
              return dDate.getFullYear() === today.getFullYear() && dDate.getMonth() === today.getMonth() && dDate.getDate() === today.getDate();
            });
            if (matchDay) {
              setTournamentStatus(matchDay.status === 'active' ? 'ok' : 'warn');
            } else {
              setTournamentStatus('warn');
            }
          } else {
            setTournamentStatus('warn');
          }
        } else {
          setTournamentStatus('warn');
        }
      } catch {
        setTournamentStatus('error');
      }

      // Recording path configured (basic check via settings)
      try {
        const full = await obsObwsCommands.getFullConfig();
        const path = (full as any)?.data?.recording_config?.recording_root_path;
        setRecordingPathStatus(path ? 'ok' : 'warn');
      } catch {
        setRecordingPathStatus('warn');
      }

      // OBS websocket connection
      try {
        const res = await obsObwsCommands.getStatus();
        setObsStatus(res.success ? 'ok' : 'warn');
      } catch {
        setObsStatus('warn');
      }

      // Disk space (best effort) on integration recording root
      try {
        const full = await obsObwsCommands.getFullConfig();
        const path = (full as any)?.data?.recording_config?.recording_root_path as string | undefined;
        if (path) {
          const disk = await checkDiskSpace(path);
          if (disk === 'warn') setRecordingPathStatus('warn');
        }
      } catch {}

      // Google Drive connectivity
      try {
        const drive = await invoke<any>('drive_get_connection_status');
        setDriveStatus(drive?.success && drive.data?.connected ? 'ok' : 'warn');
      } catch {
        setDriveStatus('warn');
      }

      setMessage('Checks completed.');
    } finally {
      setRunningChecks(false);
    }
  };

  useEffect(() => {
    if (isOpen) {
      // Reset and run
      setUdpStatus('unknown');
      setTournamentStatus('unknown');
      setRecordingPathStatus('unknown');
      setObsStatus('unknown');
      void runChecks();
    }
  }, [isOpen]);

  const activateAll = async () => {
    setMessage('Activating subsystems...');
    try {
      // Start UDP
      try { await invoke('start_udp_server'); setUdpStatus('ok'); } catch {}

      // Ensure today tournament day is active
      try {
        const active = await invoke<any>('tournament_get_active');
        if (active && active.success && active.tournament) {
          const tournamentId = active.tournament.id as number;
          const daysResult = await invoke<any>('tournament_get_days', { tournamentId });
          if (daysResult && daysResult.success) {
            const today = new Date();
            const matchDay = (daysResult.days as any[]).find(d => {
              const dDate = new Date(d.date);
              return dDate.getFullYear() === today.getFullYear() && dDate.getMonth() === today.getMonth() && dDate.getDate() === today.getDate();
            });
            if (matchDay && matchDay.status === 'pending') {
              await invoke('tournament_start_day', { tournamentDayId: matchDay.id });
              setTournamentStatus('ok');
            }
          }
        }
      } catch {}

      // Re-check after activation
      await runChecks();
    } finally {
      setMessage('');
    }
  };

  const startAll = async () => {
    setMessage('Starting systems...');
    await activateAll();

    try {
      // Read integration settings (recording config) - Integration tab is authoritative
      const full = await obsObwsCommands.getFullConfig();
      const connectionName = (full as any)?.data?.connection_name || 'OBS_REC';
      const root = (full as any)?.data?.recording_config?.recording_root_path || 'C:/Users/Public/Videos';
      const folderPattern = (full as any)?.data?.recording_config?.folder_pattern || '{tournament}/{tournamentDay}';
      const filenameTemplate = (full as any)?.data?.recording_config?.filename_template || '{matchNumber} - {player1} {player1Flag} vs {player2} {player2Flag}';

      // Resolve tournament/day for today
      let tournamentName = 'Tournament';
      let tournamentDay = 'Day 1';
      try {
        const active = await invoke<any>('tournament_get_active');
        if (active?.success && active.tournament) {
          tournamentName = active.tournament.name || tournamentName;
          const daysRes = await invoke<any>('tournament_get_days', { tournamentId: active.tournament.id });
          if (daysRes?.success) {
            const today = new Date();
            const matchDay = (daysRes.days as any[]).find(d => {
              const dDate = new Date(d.date);
              return dDate.getFullYear() === today.getFullYear() && dDate.getMonth() === today.getMonth() && dDate.getDate() === today.getDate();
            });
            if (matchDay) tournamentDay = `Day ${matchDay.day_number}`;
          }
        }
      } catch {}

      // Expand normalized placeholders (for folder only tournament/tournamentDay here)
      const expandedDir = `${root.replace(/\\/g,'/')}/${folderPattern
        .replace('{tournament}', tournamentName)
        .replace('{tournamentDay}', tournamentDay)}`.replace(/\/+/g,'/');

      // Ensure directory exists on disk
      await invoke('obs_obws_create_recording_folders', {
        recordingPath: expandedDir,
        filenameTemplate,
        tournamentName,
        tournamentDay,
        matchNumber: '',
      });

      // Push directory and filename template to OBS (overwrite OBS profile)
      await obsObwsCommands.sendConfigToObs(connectionName, expandedDir, filenameTemplate);
    } catch (e) {
      console.warn('OBS path/template push failed:', e);
    }

    setMessage('All systems ready.');
    onStarted();
  };

  // End (green -> click) flow
  const endDayAndShutdown = async () => {
    if (!confirm('Do you really want to end the active day and shutdown subsystems?')) return;
    setMessage('Ending day and shutting down subsystems...');
    try {
      // End active day (if present)
      try {
        const active = await invoke<any>('tournament_get_active');
        if (active?.success && active.tournament) {
          const daysRes = await invoke<any>('tournament_get_days', { tournamentId: active.tournament.id });
          if (daysRes?.success) {
            const running = (daysRes.days as any[]).find((d:any) => d.status === 'active');
            if (running) await invoke('tournament_end_day', { tournamentDayId: running.id });
          }
        }
      } catch (e) { console.warn('End day skipped:', e); }

      // Stop UDP server
      try { await invoke('stop_udp_server'); } catch {}

      // Stop OBS recording and streaming (if streaming connection later provided)
      try { await obsObwsCommands.stopRecording(); } catch {}
      try { await obsObwsCommands.stopStreaming(); } catch {}

      setMessage('Day ended. Systems stopped. Ready for next day setup.');
    } finally {
      // Keep modal open to allow next steps; external toggle handler will change UI state
    }
  };

  // End day flow when LIVE is green: called externally by toggle handler if needed
  // We keep implementation hooks ready for future wiring (stop UDP/OBS, end day)

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50">
      <div className="bg-gradient-to-br from-gray-800/90 to-gray-900/95 rounded-lg border border-gray-600/30 shadow-xl p-6 w-full max-w-2xl">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold text-white">Live Orchestrator</h3>
          <Button size="sm" className="bg-gray-600 hover:bg-gray-700" onClick={onClose}>Close</Button>
        </div>

        <div className="space-y-3">
          <StatusRow label="UDP Server" status={udpStatus} />
          <StatusRow label="Tournament / Day Active" status={tournamentStatus} />
          <StatusRow label="Recording Path Configured" status={recordingPathStatus} />
          <StatusRow label="OBS WebSocket" status={obsStatus} />
          <StatusRow label="Google Drive Connectivity" status={driveStatus} />
          <DriveQuotaRow />
        </div>

        {message && <div className="mt-4 text-sm text-gray-300">{message}</div>}

        <div className="flex items-center justify-between gap-2 mt-6">
          <div className="text-sm text-gray-400">Use Integration tab patterns for path and filename. Folders will be created automatically.</div>
          <div className="flex items-center gap-2">
            <Button size="sm" className="bg-gray-600 hover:bg-gray-700" onClick={runChecks} disabled={runningChecks}>Re-check</Button>
            <Button size="sm" className="bg-blue-600 hover:bg-blue-700" onClick={activateAll} disabled={runningChecks}>Activate All</Button>
            <Button size="sm" className="bg-green-600 hover:bg-green-700" onClick={startAll} disabled={runningChecks}>Start</Button>
            <Button size="sm" className="bg-red-600 hover:bg-red-700" onClick={endDayAndShutdown} disabled={runningChecks}>End Day</Button>
          </div>
        </div>
      </div>
    </div>
  );
};

export default LiveOrchestratorModal;

// Sub-component to show Drive quota if available
const DriveQuotaRow: React.FC = () => {
  const [text, setText] = React.useState<string>('');
  React.useEffect(() => {
    (async () => {
      try {
        const res = await invoke<any>('drive_get_quota');
        if (res?.success && res.quota) {
          const limit = res.quota.limit as number;
          const usage = res.quota.usage as number;
          if (limit > 0) {
            const pct = ((usage / limit) * 100).toFixed(1);
            const gb = (n: number) => (n / (1024 * 1024 * 1024)).toFixed(2);
            setText(`${gb(usage)} GB / ${gb(limit)} GB (${pct}%)`);
          }
        }
      } catch {}
    })();
  }, []);

  if (!text) return null;
  return (
    <StatusRow label="Drive Quota" right={text} />
  );
};


