import React, { useEffect } from 'react';
import {
  useTriggersStore,
  EventTriggerRow,
  DelayTriggerRow,
  TriggerRow,
} from '../../stores/triggersStore';
import { Select, SelectTrigger, SelectContent, SelectItem, SelectValue } from '../atoms/Select';
import Button from '../atoms/Button';
import { useConfirmStore } from '../../stores/confirmStore';
import Input from '../atoms/Input';

interface Props {
  tournamentId?: number;
  dayId?: number;
}

const humanReadableEvent = (ev: string) => {
  const map: Record<string, string> = {
    pre: 'Match Ready',
    rdy: 'Match Loaded',
    rnd: 'Round Start',
    sup: 'Break',
    wrd: 'Round End',
    wmh: 'Match Winner',
  };
  return map[ev] ?? ev;
};

interface RowProps {
  row: TriggerRow;
  index: number;
  eventsCatalog: string[];
}

const RowComponent: React.FC<RowProps> = ({ row, index, eventsCatalog }) => {
  const { scenes, overlays, updateRow, selectRow, selectedIndex } = useTriggersStore();
  const isSelected = selectedIndex === index;

  // Handlers
  const handleEventChange = (value: string) => {
    if (value === 'delay') {
      const delayRow: DelayTriggerRow = {
        kind: 'delay',
        delay_ms: 300,
        priority: row.priority,
      };
      updateRow(index, delayRow);
    } else {
      const eventRow: Partial<EventTriggerRow> = {
        kind: 'event',
        event_type: value,
        action: 'show',
        target_type: 'scene',
        obs_scene_id: undefined,
        overlay_template_id: undefined,
      };
      updateRow(index, eventRow);
    }
  };

  const rowClass = `border-b border-gray-600 text-sm hover:bg-blue-900 ${isSelected ? 'bg-blue-900' : ''}`;

  if (row.kind === 'delay') {
    const delayRow = row as DelayTriggerRow;
    return (
      <tr className={rowClass} onClick={() => selectRow(index)}>
        {/* Event column */}
        <td className="px-3 py-2 w-[100px]">
          <Select value="delay" onValueChange={handleEventChange}>
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {eventsCatalog.map((e) => (
                <SelectItem key={e} value={e}>
                  {humanReadableEvent(e)}
                </SelectItem>
              ))}
              <SelectItem value="delay">Delay</SelectItem>
            </SelectContent>
          </Select>
        </td>
        {/* Delay ms column (Action column slot) */}
        <td className="px-3 py-2 w-[100px]">
          <Input
            type="number"
            min={50}
            value={delayRow.delay_ms}
            onChange={(e) => updateRow(index, { delay_ms: Number(e.target.value) })}
          />
        </td>
        {/* Disabled columns */}
        <td className="px-3 py-2 w-[100px] text-gray-500">—</td>
        <td className="px-3 py-2 w-[100px] text-gray-500">—</td>
      </tr>
    );
  }

  // Event row rendering
  const evRow = row as EventTriggerRow;
  const targetType = evRow.target_type;

  return (
    <tr className={rowClass} onClick={() => selectRow(index)}>
      {/* Event column */}
      <td className="px-3 py-2 w-[100px] capitalize">
        <Select value={evRow.event_type} onValueChange={handleEventChange}>
          <SelectTrigger>
            <SelectValue placeholder="Event" />
          </SelectTrigger>
          <SelectContent>
            {eventsCatalog.map((e) => (
              <SelectItem key={e} value={e}>
                {humanReadableEvent(e)}
              </SelectItem>
            ))}
            <SelectItem value="delay">Delay</SelectItem>
          </SelectContent>
        </Select>
      </td>

      {/* Action column */}
      <td className="px-3 py-2 w-[100px]">
        <Select
          value={evRow.action}
          onValueChange={(v) => updateRow(index, { action: v as 'show' | 'hide' })}
        >
          <SelectTrigger>
            <SelectValue placeholder="Action" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="show">Show</SelectItem>
            <SelectItem value="hide">Hide</SelectItem>
          </SelectContent>
        </Select>
      </td>

      {/* Target Type */}
      <td className="px-3 py-2 w-[100px]">
        <Select
          value={targetType}
          onValueChange={(v) => updateRow(index, { target_type: v as 'scene' | 'overlay' })}
        >
          <SelectTrigger>
            <SelectValue placeholder="Type" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="scene">OBS scene</SelectItem>
            <SelectItem value="overlay">Overlay</SelectItem>
          </SelectContent>
        </Select>
      </td>

      {/* Target */}
      <td className="px-3 py-2 w-[100px]">
        {targetType === 'scene' ? (
          <Select
            value={evRow.obs_scene_id?.toString()}
            onValueChange={(v) => updateRow(index, { obs_scene_id: Number(v) })}
            disabled={!scenes.length}
          >
            <SelectTrigger>
              <SelectValue placeholder={scenes.length ? 'Scene' : 'No scenes'} />
            </SelectTrigger>
            <SelectContent>
              {scenes.map((s) => (
                <SelectItem key={s.id} value={s.id.toString()}>
                  {s.connection_name ? `${s.connection_name} – ${s.scene_name}` : s.scene_name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        ) : (
          <Select
            value={evRow.overlay_template_id?.toString()}
            onValueChange={(v) => updateRow(index, { overlay_template_id: Number(v) })}
            disabled={!overlays.length}
          >
            <SelectTrigger>
              <SelectValue placeholder={overlays.length ? 'Overlay' : 'No overlays'} />
            </SelectTrigger>
            <SelectContent>
              {overlays.map((o) => (
                <SelectItem key={o.id} value={o.id.toString()}>
                  {o.theme ? `${o.name} – ${o.theme}` : o.name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        )}
      </td>
    </tr>
  );
};

const Row = React.memo(RowComponent);

export const TriggersTable: React.FC<Props> = ({ tournamentId, dayId }) => {
  const {
    rows,
    eventsCatalog,
    loading,
    dirty,
    resumeDelay,
    setResumeDelay,
    fetchData,
    saveChanges,
    addRow,
    deleteSelectedRow,
  } = useTriggersStore();

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
              {rows.map((row, idx) => (
                <Row key={idx} row={row} index={idx} eventsCatalog={eventsCatalog} />
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
      <div className="w-40 ml-4 flex flex-col gap-2">
        <Button onClick={addRow}>Add</Button>
        <Button
          variant="danger"
          onClick={() => {
            useConfirmStore.getState().open({
              title: 'Delete Row',
              message: 'This will remove the selected trigger row.',
              delayMs: 3000,
              action: deleteSelectedRow,
            });
          }}>
          Delete
        </Button>
        <Button variant="secondary">Load</Button>
        <Button variant="secondary">Test</Button>
        <div className="mt-4">
          <label className="text-xs block mb-1">Resume delay (ms)</label>
          <Input
            type="number"
            min={0}
            value={resumeDelay}
            onChange={(e) => setResumeDelay(Number(e.target.value))}
          />
        </div>
      </div>
    </div>
  );
};
