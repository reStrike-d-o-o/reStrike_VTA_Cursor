import React, { useEffect, useMemo } from 'react';
import DragPalette from './DragPalette';
import {
  useTriggersStore,
  EventTriggerRow,
  DelayTriggerRow,
  TriggerRow,
} from '../../stores/triggersStore';
import Button from '../atoms/Button';
import { useMessageCenter } from '../../stores/messageCenter';
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

const Cell: React.FC<React.PropsWithChildren<{ className?: string }>> = ({ className = '', children }) => (
  <td className={className}>{children}</td>
);

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
        <Cell className="px-3 py-2 w-[100px] capitalize">Delay</Cell>
        <Cell className="px-3 py-2 w-[100px]">
          <Input
            type="number"
            value={dRow.delay_ms}
            onChange={(e) => updateRow(index, { ...dRow, delay_ms: Number(e.target.value) || 0 })}
            className="w-24"
          />
        </Cell>
        <Cell className="px-3 py-2">
          <span className="text-xs text-gray-400">No target</span>
        </Cell>
      </tr>
    );
  }

  const eRow = row as EventTriggerRow;
  return (
    <tr className={rowClass} onClick={() => selectRow(index)}>
      <Cell className="px-3 py-2 w-[100px] capitalize">{humanReadableEvent(eRow.event_type)}</Cell>
      <Cell className="px-3 py-2 w-[100px]">{eRow.action}</Cell>
      <Cell className="px-3 py-2">{(eRow.target_type === 'scene' ? 'Scene' : eRow.target_type === 'overlay' ? 'Overlay' : '') || <span className="text-xs text-gray-400">No target</span>}</Cell>
    </tr>
  );
};

const TriggersTable: React.FC<Props> = ({ tournamentId, dayId }) => {
  const { rows, addRow, scenes, overlays } = useTriggersStore();

  useEffect(() => {
    // table mounted
  }, []);

  return (
    <div className="flex">
      <DragPalette scenes={scenes} overlays={overlays} className="flex-none" />
      <div className="flex-1 p-3">
        <div className="flex items-center justify-between mb-2">
          <div className="text-sm text-gray-300">Triggers</div>
          <div className="space-x-2">
            <Button size="sm" onClick={() => addRow()}>Add Row</Button>
            {/* Clear action can be reintroduced if supported by store */}
          </div>
        </div>
        <table className="w-full text-left">
          <thead>
            <tr className="text-xs text-gray-400 border-b border-gray-700">
              <th className="px-3 py-1 w-[100px]">Event</th>
              <th className="px-3 py-1 w-[100px]">Action</th>
              <th className="px-3 py-1">Target</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((row, idx) => (
              <RowComponent key={idx} row={row} index={idx} />
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
};

export default TriggersTable;
