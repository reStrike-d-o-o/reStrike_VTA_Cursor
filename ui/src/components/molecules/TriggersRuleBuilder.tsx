import React, { useEffect, useMemo, useState } from 'react';
import Button from '../atoms/Button';
import Input from '../atoms/Input';
import Toggle from '../atoms/Toggle';
import { useTriggersStore, EventTriggerRow, DelayTriggerRow, TriggerRow } from '../../stores/triggersStore';
import { obsObwsCommands } from '../../utils/tauriCommandsObws';

type ObsConnection = { name: string; host?: string; port?: number };

const TriggersRuleBuilder: React.FC<{ tournamentId?: number; dayId?: number }> = ({ tournamentId, dayId }) => {
  const store = useTriggersStore();
  const {
    eventsCatalog,
    scenes,
    overlays,
    rows,
    loading,
    dirty,
    selectedIndex,
    resumeDelay,
    fetchData,
    fetchScenes,
    addRow,
    deleteSelectedRow,
    selectRow,
    updateRow,
    setResumeDelay,
    saveChanges,
  } = store;

  const [connections, setConnections] = useState<ObsConnection[]>([]);
  const [connLoading, setConnLoading] = useState(false);
  const [logs, setLogs] = useState<any[]>([]);
  const [logsLoading, setLogsLoading] = useState(false);
  const [showRecent, setShowRecent] = useState(true);

  useEffect(() => {
    fetchData(tournamentId, dayId);
    // Load OBS connections for connection targeting
    (async () => {
      try {
        setConnLoading(true);
        const res = await obsObwsCommands.getConnections();
        if (res.success && res.data?.connections) {
          setConnections(res.data.connections.map((c: any) => ({ name: c.name, host: c.host, port: c.port })));
        }
      } catch (e) {
        console.warn('Failed to fetch OBS connections', e);
      } finally {
        setConnLoading(false);
      }
    })();
  }, [fetchData, tournamentId, dayId]);

  const refreshLogs = async () => {
    try {
      setLogsLoading(true);
      const res = await (await import('@tauri-apps/api/core')).invoke<any>('triggers_recent_logs', { max: 50 });
      if (res?.success) setLogs(res.logs || []);
    } catch (e) {
      console.warn('Failed to get trigger logs', e);
    } finally {
      setLogsLoading(false);
    }
  };

  const eventOptions = useMemo(() => eventsCatalog || [], [eventsCatalog]);

  const handleRowChange = (idx: number, partial: Partial<TriggerRow>) => {
    updateRow(idx, partial);
  };

  const renderEventCell = (row: EventTriggerRow, idx: number) => (
    <select
      aria-label="Event"
      title="Event"
      className="w-full bg-gray-800/50 border border-gray-700 rounded text-sm text-gray-100 px-2 py-1"
      value={row.event_type || ''}
      onChange={(e) => handleRowChange(idx, { event_type: e.target.value })}
    >
      <option value="">Select…</option>
      {eventOptions.map((ev) => (
        <option key={ev} value={ev}>{ev}</option>
      ))}
    </select>
  );

  const actionKinds: Array<{ value: NonNullable<EventTriggerRow['action_kind']>, label: string }> = [
    { value: 'scene', label: 'Change Scene' },
    { value: 'overlay', label: 'Show Overlay' },
    { value: 'record_start', label: 'Start Recording' },
    { value: 'record_stop', label: 'Stop Recording' },
    { value: 'replay_save', label: 'Save Replay Buffer' },
    // Future: { value: 'hotkey', label: 'Send Hotkey' },
    // Future: { value: 'stinger', label: 'Play Stinger' },
  ];

  const renderActionCell = (row: EventTriggerRow, idx: number) => (
    <select
      aria-label="Action"
      title="Action"
      className="w-full bg-gray-800/50 border border-gray-700 rounded text-sm text-gray-100 px-2 py-1"
      value={row.action_kind || 'scene'}
      onChange={(e) => handleRowChange(idx, { action_kind: e.target.value as any })}
    >
      {actionKinds.map((k) => (
        <option key={k.value} value={k.value}>{k.label}</option>
      ))}
    </select>
  );

  const testRule = async (idx: number) => {
    try {
      const r = rows[idx] as EventTriggerRow;
      const trigger = {
        id: r.id ?? null,
        tournament_id: null,
        tournament_day_id: null,
        event_type: r.event_type || '',
        action: r.action || 'show',
        target_type: r.target_type || 'scene',
        obs_scene_id: r.obs_scene_id ?? null,
        overlay_template_id: r.overlay_template_id ?? null,
        delay_ms: (r as any).delay_ms ?? 0,
        is_enabled: r.is_enabled ?? true,
        priority: idx,
        action_kind: r.action_kind ?? 'scene',
        obs_connection_name: r.obs_connection_name ?? null,
        condition_round: r.condition_round ?? null,
        condition_once_per: r.condition_once_per ?? null,
        debounce_ms: r.debounce_ms ?? null,
        cooldown_ms: r.cooldown_ms ?? null,
      };
      const { invoke } = await import('@tauri-apps/api/core');
      const res = await invoke<any>('triggers_preview_evaluate', { trigger, considerLimits: true });
      const canFire = !!res?.can_fire;
      setLogs((prev) => [
        {
          ts: new Date().toISOString(),
          results: [{ trigger_id: r.id || -1, event_type: r.event_type || '(test)', success: canFire, ms: 0 }],
        },
        ...prev,
      ].slice(0, 50));
      selectRow(idx);
    } catch (e) {
      console.warn('Test rule failed', e);
    }
  };

  const handleSaveReplay = async () => {
    try {
      const selected = selectedIndex != null ? (rows[selectedIndex] as EventTriggerRow) : null;
      const conn = selected?.obs_connection_name || undefined;
      const res = await obsObwsCommands.saveReplayBuffer(conn);
      setLogs((prev) => [
        {
          ts: new Date().toISOString(),
          results: [{ trigger_id: selected?.id || -1, event_type: 'replay_save', success: !!res?.success, ms: 0 }],
        },
        ...prev,
      ].slice(0, 50));
    } catch (e) {
      console.warn('Save replay failed', e);
    }
  };

  const renderConnectionCell = (row: EventTriggerRow, idx: number) => (
    <select
      aria-label="OBS Connection"
      title="OBS Connection"
      className="w-full bg-gray-800/50 border border-gray-700 rounded text-sm text-gray-100 px-2 py-1"
      value={row.obs_connection_name || ''}
      onChange={async (e) => {
        handleRowChange(idx, { obs_connection_name: e.target.value });
        // Auto-refresh scenes when connection changes so scene dropdown is populated
        try { await fetchScenes(); } catch {}
      }}
    >
      <option value="">default</option>
      {connections.map((c) => (
        <option key={c.name} value={c.name}>{c.name}</option>
      ))}
    </select>
  );

  const renderTargetTypeCell = (row: EventTriggerRow, idx: number) => (
    <select
      aria-label="Target Type"
      title="Target Type"
      className="w-full bg-gray-800/50 border border-gray-700 rounded text-sm text-gray-100 px-2 py-1"
      value={row.target_type || 'scene'}
      onChange={(e) => handleRowChange(idx, { target_type: e.target.value as any })}
      disabled={row.action_kind && row.action_kind !== 'scene' && row.action_kind !== 'overlay'}
    >
      <option value="scene">Scene</option>
      <option value="overlay">Overlay</option>
    </select>
  );

  const renderTargetCell = (row: EventTriggerRow, idx: number) => {
    if ((row.action_kind ?? 'scene') === 'overlay' || row.target_type === 'overlay') {
      return (
        <select
          aria-label="Overlay Target"
          title="Overlay Target"
          className="w-full bg-gray-800/50 border border-gray-700 rounded text-sm text-gray-100 px-2 py-1"
          value={row.overlay_template_id || ''}
          onChange={(e) => handleRowChange(idx, { overlay_template_id: Number(e.target.value) || undefined, target_type: 'overlay' })}
        >
          <option value="">Select overlay…</option>
          {overlays.map((o) => (
            <option key={o.id} value={o.id}>{o.name}{o.theme ? ` – ${o.theme}` : ''}</option>
          ))}
        </select>
      );
    }
    return (
      <select
        aria-label="Scene Target"
        title="Scene Target"
        className="w-full bg-gray-800/50 border border-gray-700 rounded text-sm text-gray-100 px-2 py-1"
        value={row.obs_scene_id || ''}
        onChange={(e) => handleRowChange(idx, { obs_scene_id: Number(e.target.value) || undefined, target_type: 'scene' })}
      >
        <option value="">Select scene…</option>
        {scenes.map((s, i) => (
          <option key={s.id ?? `${s.scene_id}-${i}`} value={s.id ?? i}>
            {s.connection_name ? `${s.connection_name} – ` : ''}{s.scene_name}
          </option>
        ))}
      </select>
    );
  };

  const renderConditionsCell = (row: EventTriggerRow, idx: number) => (
    <div className="flex gap-2 items-center">
      <Input
        type="number"
        className="w-16"
        placeholder="Rnd"
        value={row.condition_round ?? ''}
        onChange={(e) => handleRowChange(idx, { condition_round: e.target.value === '' ? null : Number(e.target.value) })}
      />
      <select
        aria-label="Once-per scope"
        title="Once-per scope"
        className="bg-gray-800/50 border border-gray-700 rounded text-sm text-gray-100 px-2 py-1"
        value={row.condition_once_per ?? ''}
        onChange={(e) => handleRowChange(idx, { condition_once_per: (e.target.value || null) as any })}
      >
        <option value="">— once —</option>
        <option value="round">Per Round</option>
        <option value="match">Per Match</option>
      </select>
      <Input
        type="number"
        className="w-24"
        placeholder="Debounce"
        value={row.debounce_ms ?? ''}
        onChange={(e) => handleRowChange(idx, { debounce_ms: e.target.value === '' ? null : Number(e.target.value) })}
      />
      <Input
        type="number"
        className="w-24"
        placeholder="Cooldown"
        value={row.cooldown_ms ?? ''}
        onChange={(e) => handleRowChange(idx, { cooldown_ms: e.target.value === '' ? null : Number(e.target.value) })}
      />
    </div>
  );

  if (loading) return <div className="p-4">Loading triggers…</div>;

  return (
    <div className="flex h-full flex-col gap-3">
      <div className="flex items-center justify-between">
        <div className="text-sm text-gray-400">OBS connections: {connLoading ? 'loading…' : connections.length}</div>
        <div className="flex gap-2">
          <Button onClick={addRow}>Add Rule</Button>
          <Button variant="danger" onClick={deleteSelectedRow} disabled={selectedIndex == null}>Delete</Button>
          <Button variant="secondary" onClick={fetchScenes} disabled={loading}>Load OBS Scenes</Button>
          <div className="flex items-center gap-2 ml-4">
            <label className="text-xs text-gray-400">Resume delay (ms)</label>
            <Input type="number" className="w-24" value={resumeDelay} onChange={(e) => setResumeDelay(Number(e.target.value) || 0)} />
          </div>
          <Button variant="primary" onClick={saveChanges} disabled={!dirty}>Save</Button>
          <Button variant="secondary" onClick={() => setShowRecent((s) => !s)}>{showRecent ? 'Hide' : 'Show'} Recent</Button>
          <Button variant="secondary" onClick={refreshLogs} disabled={logsLoading}>Refresh Logs</Button>
          <Button variant="secondary" onClick={handleSaveReplay}>Save Replay</Button>
        </div>
      </div>

      <div className="flex-1 overflow-hidden border border-gray-700 bg-[#0D131A] flex">
        <div className="flex-1 overflow-auto">
        <table className="min-w-full text-left text-sm text-gray-200 border-collapse">
          <thead className="sticky top-0 bg-[#101820] z-10">
            <tr>
              <th className="px-3 py-2 w-[120px]">Event</th>
              <th className="px-3 py-2 w-[160px]">Action</th>
              <th className="px-3 py-2 w-[140px]">Connection</th>
              <th className="px-3 py-2 w-[120px]">Target Type</th>
              <th className="px-3 py-2 w-[260px]">Target</th>
              <th className="px-3 py-2">
                <div className="flex items-center gap-3">
                  <span>Conditions</span>
                  <div className="hidden md:flex text-[10px] text-gray-400 gap-6">
                    <span>Round</span>
                    <span>Once-per</span>
                    <span>Debounce</span>
                    <span>Cooldown</span>
                  </div>
                </div>
              </th>
              <th className="px-3 py-2 w-[90px]">Enabled</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((r, idx) => {
              if ((r as any).kind === 'delay') {
                const d = r as DelayTriggerRow;
                return (
                  <tr key={idx} className={`border-b border-gray-700 ${selectedIndex === idx ? 'bg-blue-900/30' : ''}`} onClick={() => selectRow(idx)}>
                    <td className="px-3 py-2 text-gray-400">Delay</td>
                    <td className="px-3 py-2" colSpan={4}>
                      <Input type="number" className="w-28" value={d.delay_ms} onChange={(e) => handleRowChange(idx, { delay_ms: Number(e.target.value) || 0 })} />
                    </td>
                    <td className="px-3 py-2"></td>
                    <td className="px-3 py-2"><span className="text-gray-500">—</span></td>
                  </tr>
                );
              }
              const row = r as EventTriggerRow;
              return (
                <tr key={idx} className={`border-b border-gray-700 ${selectedIndex === idx ? 'bg-blue-900/30' : ''}`} onClick={() => selectRow(idx)}>
                  <td className="px-3 py-2">{renderEventCell(row, idx)}</td>
                  <td className="px-3 py-2">{renderActionCell(row, idx)}</td>
                  <td className="px-3 py-2">{renderConnectionCell(row, idx)}</td>
                  <td className="px-3 py-2">{renderTargetTypeCell(row, idx)}</td>
                  <td className="px-3 py-2">{renderTargetCell(row, idx)}</td>
                  <td className="px-3 py-2">{renderConditionsCell(row, idx)}</td>
                  <td className="px-3 py-2 flex items-center gap-2">
                    <Toggle id={`en-${idx}`} checked={row.is_enabled} onChange={(e) => handleRowChange(idx, { is_enabled: e.target.checked })} label="" />
                    <Button size="sm" variant="secondary" onClick={() => testRule(idx)}>Test</Button>
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
        </div>
        {showRecent && (
        <div className="w-[360px] border-l border-gray-700 overflow-auto">
          <div className="p-3 sticky top-0 bg-[#101820] flex items-center justify-between">
            <div className="text-gray-300 font-medium">Recent Executions</div>
            <div className="flex gap-2">
              <Button size="sm" variant="secondary" onClick={refreshLogs} disabled={logsLoading}>Refresh</Button>
              <Button size="sm" variant="secondary" onClick={() => setShowRecent(false)}>Collapse</Button>
            </div>
          </div>
          <div className="p-2 space-y-2">
            {logsLoading && <div className="text-sm text-gray-400 p-2">Loading…</div>}
            {!logsLoading && logs.length === 0 && <div className="text-sm text-gray-500 p-2">No recent executions</div>}
            {logs.map((entry, i) => (
              <div key={i} className="p-2 bg-gray-800/50 border border-gray-700">
                <div className="text-xs text-gray-400">{entry.ts || ''}</div>
                {(entry.results || []).map((r: any, idx: number) => (
                  <div key={idx} className="text-xs text-gray-300 flex justify-between">
                    <span>#{r.trigger_id} {r.event_type}</span>
                    <span className={r.success ? 'text-green-400' : 'text-red-400'}>{r.success ? 'OK' : 'ERR'}</span>
                  </div>
                ))}
              </div>
            ))}
          </div>
        </div>
        )}
      </div>
    </div>
  );
};

export default TriggersRuleBuilder;


