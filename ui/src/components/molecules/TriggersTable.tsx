import React, { useEffect, useMemo } from 'react';
import { DndContext, DragEndEvent, useDroppable } from '@dnd-kit/core';
import DragPalette from './DragPalette';
import {
  useTriggersStore,
  EventTriggerRow,
  DelayTriggerRow,
  TriggerRow,
} from '../../stores/triggersStore';
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
}

// Allowed prefixes per column for validation / highlight
const allowedPrefixes = {
  event: ['ev-'],
  action: ['act-'],
  target: ['scene-', 'ov-'],
};

// Generic droppable TD
const DroppableCell: React.FC<
  React.PropsWithChildren<{ id: string; column: 'event' | 'action' | 'target'; className?: string }>
> = ({ id, column, className = '', children }) => {
  const { isOver, setNodeRef, active } = useDroppable({ id });
  const hoveringAllowed =
    isOver && active?.id && allowedPrefixes[column]?.some((p: string) => active.id.toString().startsWith(p));
  return (
    <td
      ref={setNodeRef}
      className={`${className} ${hoveringAllowed ? 'ring-2 ring-blue-400' : ''}`}
    >
      {children}
    </td>
  );
};

const RowComponent: React.FC<RowProps> = ({ row, index }) => {
  const { scenes, overlays, updateRow, selectRow, selectedIndex } =
    useTriggersStore();
  const isSelected = selectedIndex === index;

  const rowClass = `border-b border-gray-600 text-sm hover:bg-blue-900 ${
    isSelected ? 'bg-blue-900' : ''
  }`;

  if (row.kind === 'delay') {
    const dRow = row as DelayTriggerRow;
    return (
      <tr className={rowClass} onClick={() => selectRow(index)}>
        <DroppableCell
          id={`cell-${index}-event`}
          column="event"
          className="px-3 py-2 w-[100px] capitalize"
        >
          Delay
        </DroppableCell>
        <DroppableCell id={`cell-${index}-action`} column="action" className="px-3 py-2 w-[100px]">
          <Input
            type="number"
            min={50}
            value={dRow.delay_ms}
            onChange={(e) => updateRow(index, { delay_ms: Number(e.target.value) })}
          />
        </DroppableCell>
        <td className="px-3 py-2 w-[100px] text-gray-500">—</td>
        <td className="px-3 py-2 w-[100px] text-gray-500">—</td>
      </tr>
    );
  }

  const evRow = row as EventTriggerRow;
  const sceneName = useMemo(() => {
    const s = scenes.find((sc) => sc.id === evRow.obs_scene_id);
    if (!s) return '';
    return s.connection_name ? `${s.connection_name} – ${s.scene_name}` : s.scene_name;
  }, [scenes, evRow.obs_scene_id]);

  const overlayName = useMemo(() => {
    const o = overlays.find((ov) => ov.id === evRow.overlay_template_id);
    if (!o) return '';
    return o.theme ? `${o.name} – ${o.theme}` : o.name;
  }, [overlays, evRow.overlay_template_id]);

  return (
    <tr className={rowClass} onClick={() => selectRow(index)}>
      <DroppableCell
        id={`cell-${index}-event`}
        column="event"
        className="px-3 py-2 w-[100px] capitalize"
      >
        {humanReadableEvent(evRow.event_type)}
      </DroppableCell>

      <DroppableCell id={`cell-${index}-action`} column="action" className="px-3 py-2 w-[100px]">
        {evRow.action}
      </DroppableCell>

      <td className="px-3 py-2 w-[100px]">{evRow.target_type}</td>

      <DroppableCell id={`cell-${index}-target`} column="target" className="px-3 py-2 w-[100px]">
        {evRow.target_type === 'scene' ? sceneName : overlayName}
      </DroppableCell>
    </tr>
  );
};

const Row = React.memo(RowComponent);

export const TriggersTable: React.FC<Props> = ({ tournamentId, dayId }) => {
  const handleDragEnd = (e: DragEndEvent) => {
    const { over, active } = e;
    if (!over) return;
    const parts = over.id.toString().split('-');
    if (parts[0] !== 'cell') return;
    const rowIdx = Number(parts[1]);
    const column = parts[2] as 'event' | 'action' | 'target';
    const store = useTriggersStore.getState();

    const aid = active.id.toString();

    if (column === 'event' && aid.startsWith('ev-')) {
      store.updateRow(rowIdx, { kind: 'event', event_type: aid.substring(3) });
    }
    if (column === 'action' && aid.startsWith('act-')) {
      const act = aid.substring(4);
      if (act === 'delay') {
        store.updateRow(rowIdx, { kind: 'delay', delay_ms: 300 });
      } else {
        store.updateRow(rowIdx, { kind: 'event', action: act as 'show' | 'hide' });
      }
    }
    if (column === 'target') {
      if (aid.startsWith('scene-')) {
        const id = Number(aid.substring(6));
        store.updateRow(rowIdx, { overlay_template_id: undefined, obs_scene_id: id, target_type: 'scene' });
      }
      if (aid.startsWith('ov-')) {
        const id = Number(aid.substring(3));
        store.updateRow(rowIdx, { obs_scene_id: undefined, overlay_template_id: id, target_type: 'overlay' });
      }
    }
  };

  const {
    rows,
    loading,
    dirty,
    resumeDelay,
    setResumeDelay,
    fetchData,
    saveChanges,
    addRow,
    deleteSelectedRow,
    scenes,
    overlays,
  } = useTriggersStore();

  useEffect(() => {
    fetchData(tournamentId, dayId);
  }, [tournamentId, dayId]);

  if (loading) return <div className="p-4">Loading triggers…</div>;

  return (
    <DndContext onDragEnd={handleDragEnd}>
      <div className="flex h-full">
        {/* Palette */}
        

        {/* Table */}
        <div className="flex-1 flex flex-col border-r border-gray-700">
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
                  <Row key={idx} row={row} index={idx} />
                ))}
              </tbody>
            </table>
          </div>
          {/* Badge palette row */}
          <div className="border-t border-gray-700 bg-gray-800/70 backdrop-blur-sm p-2 flex flex-row space-x-4 overflow-x-auto">
            <DragPalette scenes={scenes} overlays={overlays} />
          </div>
          <div className="border-t border-gray-700 bg-gray-800 p-2 text-right">
            <Button variant="primary" onClick={saveChanges} disabled={!dirty}>
              Save
            </Button>
          </div>
        </div>

        {/* Buttons */}
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
            }}
          >
            Delete
          </Button>
          <Button 
            variant="secondary" 
            onClick={() => useTriggersStore.getState().fetchScenes()}
            disabled={loading}
          >
            {loading ? 'Loading...' : 'Load OBS Scenes'}
          </Button>
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
    </DndContext>
  );
};

export default TriggersTable;
