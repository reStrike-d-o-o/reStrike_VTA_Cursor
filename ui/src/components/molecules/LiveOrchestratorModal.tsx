import React, { useEffect, useMemo, useState } from 'react';
import Button from '../atoms/Button';
import StatusDot from '../atoms/StatusDot';
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
  const [message, setMessage] = useState<string>('');

  const colorFor = (s: CheckStatus) => {
    switch (s) {
      case 'ok': return 'green';
      case 'warn': return 'yellow';
      case 'error': return 'red';
      default: return 'gray';
    }
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
        const settings = await invoke<any>('get_settings');
        const path = settings?.video?.recording?.folder || settings?.recording?.path;
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
    setMessage('All systems ready.');
    onStarted();
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50">
      <div className="bg-gradient-to-br from-gray-800/90 to-gray-900/95 rounded-lg border border-gray-600/30 shadow-xl p-6 w-full max-w-2xl">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold text-white">Live Orchestrator</h3>
          <Button size="sm" className="bg-gray-600 hover:bg-gray-700" onClick={onClose}>Close</Button>
        </div>

        <div className="space-y-3">
          <div className="flex items-center justify-between p-3 rounded bg-gray-800/50 border border-gray-700">
            <div className="text-gray-200">UDP Server</div>
            <StatusDot color={colorFor(udpStatus)} />
          </div>
          <div className="flex items-center justify-between p-3 rounded bg-gray-800/50 border border-gray-700">
            <div className="text-gray-200">Tournament / Day Active</div>
            <StatusDot color={colorFor(tournamentStatus)} />
          </div>
          <div className="flex items-center justify-between p-3 rounded bg-gray-800/50 border border-gray-700">
            <div className="text-gray-200">Recording Path Configured</div>
            <StatusDot color={colorFor(recordingPathStatus)} />
          </div>
          <div className="flex items-center justify-between p-3 rounded bg-gray-800/50 border border-gray-700">
            <div className="text-gray-200">OBS WebSocket</div>
            <StatusDot color={colorFor(obsStatus)} />
          </div>
        </div>

        {message && <div className="mt-4 text-sm text-gray-300">{message}</div>}

        <div className="flex items-center justify-end gap-2 mt-6">
          <Button size="sm" className="bg-gray-600 hover:bg-gray-700" onClick={runChecks} disabled={runningChecks}>Re-check</Button>
          <Button size="sm" className="bg-blue-600 hover:bg-blue-700" onClick={activateAll} disabled={runningChecks}>Activate All</Button>
          <Button size="sm" className="bg-green-600 hover:bg-green-700" onClick={startAll} disabled={runningChecks}>Start</Button>
        </div>
      </div>
    </div>
  );
};

export default LiveOrchestratorModal;


