/**
 * DragPalette
 * - Small utility palette for scenarios (static badges; dragging removed)
 */
import React from 'react';

interface BadgeProps {
  id: string;
  label: string;
  colorClass: string;
}

const humanReadableEvent = (ev: string) => {
  const map: Record<string, string> = {
    pre: 'Match Ready', rdy: 'Match Loaded', rnd: 'Round Start', sup: 'Break', wrd: 'Round End', wmh: 'Match Winner',
  };
  return map[ev] ?? ev;
};

const Badge: React.FC<BadgeProps> = ({ label, colorClass }) => {
  const color = colorClass.replace('bg-', '');
  const containerBg = `bg-${color}/10`;
  const borderColor = `border-${color}/20`;
  const textColor = `text-${color.replace('600','400')}`;
  return (
    <div
      className={`flex items-center px-3 py-1 rounded-lg backdrop-blur-sm text-xs font-medium select-none mb-2 ${containerBg} ${borderColor} border ${textColor}`}
    >
      <span>{label}</span>
    </div>
  );
};

interface DragPaletteProps {
  scenes: { id: number; scene_name: string }[];
  overlays: { id: number; name: string; theme: string | null }[];
}

const DragPalette: React.FC<React.PropsWithChildren<DragPaletteProps & {className?:string}>> = ({ scenes, overlays, className='' }) => {
  return (
    <div className={`w-40 p-2 border-r border-gray-700 overflow-y-auto max-h-[600px] ${className}`}>
      <h3 className="text-xs font-bold text-gray-300 mb-1">Events</h3>
      {['pre', 'rdy', 'rnd', 'sup', 'wrd', 'wmh'].map(ev => (
        <Badge key={ev} id={`ev-${ev}`} label={humanReadableEvent(ev)} colorClass="bg-blue-600" />
      ))}

      <h3 className="text-xs font-bold text-gray-300 mt-4 mb-1">Actions</h3>
      {['show', 'hide', 'delay'].map(a => (
        <Badge key={a} id={`act-${a}`} label={a} colorClass="bg-green-600" />
      ))}

      <h3 className="text-xs font-bold text-gray-300 mt-4 mb-1">Scenes</h3>
      {scenes.map(s => (
        <Badge key={s.id} id={`scene-${s.id}`} label={s.scene_name} colorClass="bg-orange-500" />
      ))}

      <h3 className="text-xs font-bold text-gray-300 mt-4 mb-1">Overlays</h3>
      {overlays.map(o => (
        <Badge key={o.id} id={`ov-${o.id}`} label={o.name} colorClass="bg-amber-500" />
      ))}
    </div>
  );
};

export default DragPalette;
