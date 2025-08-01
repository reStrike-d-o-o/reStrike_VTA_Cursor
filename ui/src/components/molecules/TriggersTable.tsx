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
  const existing = triggers.find((t) => t.event_type === event);
  const triggerType: TriggerType = existing?.trigger_type ?? 'scene';
  const sceneId = existing?.obs_scene_id;
  const overlayId = existing?.overlay_template_id;

  // determine current target type
  const targetType = sceneId !== undefined ? 'scene' : 'overlay';

  return (
    <tr className="border-b border-gray-600 text-sm hover:bg-blue-900">
      {/* Event */}
      <td className="px-3 py-2 w-[100px] capitalize">{event}</td>

      {/* Action */}
      <td className="px-3 py-2 w-[100px]">
        <Select
          value={triggerType}
          onValueChange={(v: string) => updateTrigger(event, { trigger_type: v as TriggerType })}
        >
          <SelectTrigger>
            <SelectValue placeholder="Action" />
          </SelectTrigger>
          <SelectContent>
            {['scene', 'overlay', 'both'].map((opt) => (
              <SelectItem key={opt} value={opt}>
                {opt}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </td>

      {/* Target Type */}
      <td className="px-3 py-2 w-[100px]">
        <Select
          value={targetType}
          onValueChange={(v: string) => {
            if (v === 'scene') {
              updateTrigger(event, { overlay_template_id: undefined });
            } else {
              updateTrigger(event, { obs_scene_id: undefined });
            }
          }}
        >
          <SelectTrigger>
            <SelectValue placeholder="Type" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="scene">scene</SelectItem>
            <SelectItem value="overlay">overlay</SelectItem>
          </SelectContent>
        </Select>
      </td>

      {/* Target */}
      <td className="px-3 py-2 w-[100px]">
        {targetType === 'scene' ? (
          <Select
            value={sceneId?.toString()}
            onValueChange={(v) => updateTrigger(event, { obs_scene_id: Number(v) })}
            disabled={!scenes.length}
          >
            <SelectTrigger>
              <SelectValue placeholder={scenes.length ? 'Scene' : 'No scenes'} />
            </SelectTrigger>
            <SelectContent>
              {scenes.map((s) => (
                <SelectItem key={s.id} value={s.id.toString()}>
                  {s.scene_name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        ) : (
          <Select
            value={overlayId?.toString()}
            onValueChange={(v) => updateTrigger(event, { overlay_template_id: Number(v) })}
            disabled={!overlays.length}
          >
            <SelectTrigger>
              <SelectValue placeholder={overlays.length ? 'Overlay' : 'No overlays'} />
            </SelectTrigger>
            <SelectContent>
              {overlays.map((o) => (
                <SelectItem key={o.id} value={o.id.toString()}>
                  {o.name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        )}
      </td>
    </tr>
  );
};

export const TriggersTable: React.FC<Props> = ({ tournamentId, dayId }) => {
  const { events, loading, dirty, fetchData, saveChanges } = useTriggersStore();

  useEffect(() => {
    fetchData(tournamentId, dayId);
  }, [tournamentId, dayId]);

  if (loading) return <div className="p-4">Loading triggers…</div>;

  return (
    <div className="flex h-full">
      {/* Left: table */}
      <div className="flex-1 flex flex-col">
        <div className="flex-1 overflow-auto">
          <table className="min-w-full text-left text-sm text-gray-200 border-collapse">
            <thead className="sticky top-0 bg-[#101820] z-10">
              <tr>
                <th className="px-3 py-2 w-[100px]">Event</th>
                <th className="px-3 py-2 w-[100px]">Action</th>
                <th className="px-3 py-2 w-[100px]">Target Type</th>
                <th className="px-3 py-2 w-[100px]">Target</th>
              </tr>
            </thead>
            <tbody>
              {events.map((ev) => (
                <TriggerRow key={ev} event={ev} />
              ))}
            </tbody>
          </table>
        </div>
        <div className="border-t border-gray-700 bg-gray-800 p-2 text-right">
          <Button variant="primary" onClick={saveChanges} disabled={!dirty}>
            Save
          </Button>
        </div>
      </div>

      {/* Right: buttons */}
      <div className="w-32 ml-4 flex flex-col gap-2">
        <Button>Add</Button>
        <Button variant="danger">Delete</Button>
        <Button variant="secondary">Load</Button>
        <Button variant="secondary">Test</Button>
      </div>
    </div>
  );
};