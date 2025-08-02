import React from 'react';
import { useDraggable } from '@dnd-kit/core';

interface BadgeProps {
  id: string;
  label: string;
  colorClass: string;
}

const DraggableBadge: React.FC<BadgeProps> = ({ id, label, colorClass }) => {
  const { attributes, listeners, setNodeRef, transform } = useDraggable({ id });
  const style: React.CSSProperties = {
    transform: transform ? `translate3d(${transform.x}px, ${transform.y}px, 0)` : undefined,
    cursor: 'grab',
    zIndex: 1000,
    position: 'relative',
  };

  return (
    <div
      ref={setNodeRef}
      {...listeners}
      {...attributes}
      style={style}
      className={`px-3 py-1 rounded-full text-xs font-semibold text-white ${colorClass} select-none mb-2`}
    >
      {label}
    </div>
  );
};

interface DragPaletteProps {
  scenes: { id: number; scene_name: string }[];
  overlays: { id: number; name: string; theme: string | null }[];
}

const DragPalette: React.FC<DragPaletteProps> = ({ scenes, overlays }) => {
  return (
    <div className="w-40 p-2 border-r border-gray-700 overflow-y-auto" style={{ maxHeight: '600px' }}>
      <h3 className="text-xs font-bold text-gray-300 mb-1">Events</h3>
      {['pre', 'rdy', 'rnd', 'sup', 'wrd', 'wmh'].map(ev => (
        <DraggableBadge key={ev} id={`ev-${ev}`} label={ev} colorClass="bg-blue-600" />
      ))}

      <h3 className="text-xs font-bold text-gray-300 mt-4 mb-1">Actions</h3>
      {['show', 'hide', 'delay'].map(a => (
        <DraggableBadge key={a} id={`act-${a}`} label={a} colorClass="bg-green-600" />
      ))}

      <h3 className="text-xs font-bold text-gray-300 mt-4 mb-1">Scenes</h3>
      {scenes.map(s => (
        <DraggableBadge key={s.id} id={`scene-${s.id}`} label={s.scene_name} colorClass="bg-orange-500" />
      ))}

      <h3 className="text-xs font-bold text-gray-300 mt-4 mb-1">Overlays</h3>
      {overlays.map(o => (
        <DraggableBadge key={o.id} id={`ov-${o.id}`} label={o.name} colorClass="bg-amber-500" />
      ))}
    </div>
  );
};

export default DragPalette;
