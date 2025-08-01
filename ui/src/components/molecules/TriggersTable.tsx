import React, { useEffect } from 'react';
import { useTriggersStore, TriggerType } from '../../stores/triggersStore';
import { Select, SelectTrigger, SelectContent, SelectItem, SelectValue } from '../atoms/Select';
import Button from '../atoms/Button';

interface Props {
  tournamentId?: number;
  dayId?: number;
}

const TriggerRow: React.FC<{ event: string }> = ({ event }) => {
  const { triggers, scenes, overlays, updateTrigger } = useTriggersStore();
  const existing = triggers.find((t: import('../../stores/triggersStore').TriggerRow) => t.event_type === event);

  const checked = existing?.is_enabled ?? false;
  const triggerType: TriggerType = existing?.trigger_type ?? 'scene';
  const sceneId = existing?.obs_scene_id;
  const overlayId = existing?.overlay_template_id;

  return (
    <tr className="border-b border-gray-600 text-sm">
      <td className="px-2 py-1 text-left">
        <input
          type="checkbox"
          aria-label={`enable ${event}`}
          checked={checked}
          onChange={e => updateTrigger(event, { is_enabled: e.target.checked })}
        />
      </td>
      <td className="px-2 py-1 capitalize">{event}</td>
      <td className="px-2 py-1">
        <Select
          value={triggerType}
          onValueChange={(v: string) => updateTrigger(event, { trigger_type: v as TriggerType })}
        >
          <SelectTrigger>
            <SelectValue placeholder="Type" />
          </SelectTrigger>
          <SelectContent>
            {['scene','overlay','both'].map(opt => (
              <SelectItem key={opt} value={opt}>{opt}</SelectItem>
            ))}
          </SelectContent>
        </Select>
      </td>
      <td className="px-2 py-1">
        <Select
          value={sceneId?.toString()}
          onValueChange={v => updateTrigger(event, { obs_scene_id: Number(v) })}
          disabled={scenes.length === 0}
        >
          <SelectTrigger>
            <SelectValue placeholder={scenes.length ? 'Scene' : 'Please connect OBS_REC'} />
          </SelectTrigger>
          <SelectContent>
            {scenes.map((s: import('../../stores/triggersStore').ObsScene) => (
              <SelectItem key={s.id} value={s.id.toString()}>{s.scene_name}</SelectItem>
            ))}
          </SelectContent>
        </Select>
      </td>
      <td className="px-2 py-1">
        <Select
          value={overlayId?.toString()}
          onValueChange={v => updateTrigger(event, { overlay_template_id: Number(v) })}
          disabled={overlays.length === 0}
        >
          <SelectTrigger>
            <SelectValue placeholder="Overlay" />
          </SelectTrigger>
          <SelectContent>
            {overlays.map((o: import('../../stores/triggersStore').OverlayTemplate) => (
              <SelectItem key={o.id} value={o.id.toString()}>{o.name}</SelectItem>
            ))}
          </SelectContent>
        </Select>
      </td>
    </tr>
  );
};

export const TriggersTable: React.FC<Props> = ({ tournamentId, dayId }) => {
  const { events, loading, fetchData } = useTriggersStore();

  useEffect(() => {
    fetchData(tournamentId, dayId);
  }, [tournamentId, dayId]);

  if (loading) return <div className="p-4">Loading triggers…</div>;

  return (
    <div className="overflow-auto max-h-[70vh]">
      <table className="w-full text-left border-collapse">
        <thead className="sticky top-0 bg-gray-800">
          <tr className="text-xs uppercase tracking-wider">
            <th className="px-2 py-1"></th>
            <th className="px-2 py-1">Event</th>
            <th className="px-2 py-1">Trigger Type</th>
            <th className="px-2 py-1">Scene</th>
            <th className="px-2 py-1">Overlay</th>
          </tr>
        </thead>
        <tbody>
          {events.map((ev: string) => (
            <TriggerRow key={ev} event={ev} />
          ))}
        </tbody>
      </table>
      <div className="mt-4 text-right">
        <Button variant="primary" onClick={() => useTriggersStore.getState().saveChanges()}>Save</Button>
      </div>
    </div>
  );
};