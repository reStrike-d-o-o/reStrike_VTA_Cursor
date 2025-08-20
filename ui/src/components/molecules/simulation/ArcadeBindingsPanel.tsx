import React, { useEffect, useMemo, useState } from 'react';
import { useI18n } from '../../../i18n/index';
import { defaultMapping, loadMapping, saveMapping, listConnectedGamepads } from './gamepad';

const ArcadeBindingsPanel: React.FC = () => {
  const [cfg, setCfg] = useState(loadMapping());
  const [pads, setPads] = useState<ReturnType<typeof listConnectedGamepads>>([]);

  useEffect(() => {
    const refresh = () => setPads(listConnectedGamepads());
    refresh();
    const t = setInterval(refresh, 1000);
    window.addEventListener('gamepadconnected', refresh);
    window.addEventListener('gamepaddisconnected', refresh);
    return () => { clearInterval(t); window.removeEventListener('gamepadconnected', refresh); window.removeEventListener('gamepaddisconnected', refresh); };
  }, []);

  const update = (updater: (c: typeof cfg) => typeof cfg) => {
    setCfg(prev => {
      const n = updater(prev);
      saveMapping(n);
      return n;
    });
  };

  const { t } = useI18n();
  const mappingRows = useMemo(() => ([
    { label: 'Move X Axis', key: 'moveX.index', player: 1 },
    { label: 'Punch (1pt)', key: 'punch.index', player: 1 },
    { label: 'Body (2pt)', key: 'body.index', player: 1 },
    { label: 'Head (3pt)', key: 'head.index', player: 1 },
    { label: 'Tech Body (4pt)', key: 'tech_body.index', player: 1 },
    { label: 'Tech Head (5pt)', key: 'tech_head.index', player: 1 },
    { label: 'Warning', key: 'warning.index', player: 1 },
    { label: 'Hit Level (btn)', key: 'hit_level.index', player: 1 },
    { label: 'Move X Axis', key: 'moveX.index', player: 2 },
    { label: 'Punch (1pt)', key: 'punch.index', player: 2 },
    { label: 'Body (2pt)', key: 'body.index', player: 2 },
    { label: 'Head (3pt)', key: 'head.index', player: 2 },
    { label: 'Tech Body (4pt)', key: 'tech_body.index', player: 2 },
    { label: 'Tech Head (5pt)', key: 'tech_head.index', player: 2 },
    { label: 'Warning', key: 'warning.index', player: 2 },
    { label: 'Hit Level (btn)', key: 'hit_level.index', player: 2 },
  ]), []);

  const setIndex = (player: 1 | 2, key: string, value: number) => {
    update(prev => {
      const next = { ...prev, [player === 1 ? 'player1' : 'player2']: { ...(player === 1 ? prev.player1 : prev.player2) } } as any;
      const target = player === 1 ? next.player1 : next.player2;
      const [field, sub] = key.split('.');
      if (sub === 'index') target[field].index = value;
      return next;
    });
  };

  return (
    <div className="space-y-3">
      <div className="text-sm text-gray-300">Gamepad mapping. Connect two controllers; assign indices and button/axis IDs as needed.</div>
      <div className="text-xs text-gray-400">Connected gamepads:</div>
      <div className="grid grid-cols-1 md:grid-cols-2 gap-2">
        {pads.map((p, i) => (
          <div key={i} className="p-2 border border-gray-700">
            <div className="text-sm text-gray-200">{p?.id || 'Unknown'}</div>
            <div className="text-xs text-gray-400">Index: {i}</div>
          </div>
        ))}
        {pads.length === 0 && <div className="text-xs text-gray-500">No gamepads detected</div>}
      </div>
      <div className="grid grid-cols-1 md:grid-cols-2 gap-2">
        {[1,2].map((player) => (
          <div key={player} className="space-y-2 p-2 border border-gray-700">
            <div className="flex items-center justify-between">
              <div className="text-sm text-gray-200">Player {player}</div>
              <div className="flex items-center space-x-2 text-xs">
                <span className="text-gray-400" id={`lbl-pad-${player}`}>Pad Index</span>
                <input
                  className="w-16 bg-transparent border border-gray-700 text-gray-200 px-1 py-0.5"
                  type="number"
                  value={player === 1 ? cfg.player1.gamepadIndex : cfg.player2.gamepadIndex}
                  aria-labelledby={`lbl-pad-${player}`}
                  title={`Player ${player} pad index`}
                  onChange={(e) => update(prev => ({
                    ...prev,
                    [player === 1 ? 'player1' : 'player2']: {
                      ...(player === 1 ? prev.player1 : prev.player2),
                      gamepadIndex: Number(e.target.value || 0)
                    }
                  }))}
                />
              </div>
            </div>
            {mappingRows.filter(r => r.player === player).map((row) => {
              const v = (player === 1 ? (cfg as any).player1 : (cfg as any).player2);
              const idx = row.key.split('.')[0] === 'moveX' ? v.moveX.index : (v as any)[row.key.split('.')[0]].index;
              return (
                <div key={row.label} className="flex items-center justify-between text-xs">
                  <span className="text-gray-300" id={`lbl-${player}-${row.key}`}>{row.label}</span>
                  <input
                    className="w-16 bg-transparent border border-gray-700 text-gray-200 px-1 py-0.5"
                    type="number"
                    value={idx}
                    aria-labelledby={`lbl-${player}-${row.key}`}
                    title={`${row.label} mapping for player ${player}`}
                    onChange={(e) => setIndex(player as 1|2, row.key, Number(e.target.value || 0))}
                  />
                </div>
              );
            })}
          </div>
        ))}
      </div>
      <div className="flex items-center space-x-2 text-xs">
        <span className="text-gray-400" id="lbl-hit-level">Hit Level Value</span>
        <input
          className="w-20 bg-transparent border border-gray-700 text-gray-200 px-1 py-0.5"
          type="number"
          value={cfg.hitLevelValue}
          aria-labelledby="lbl-hit-level"
          title={t('simulation.arcade.hit_level_tooltip', 'Intezitet udarca')}
          onChange={(e) => update(prev => ({ ...prev, hitLevelValue: Number(e.target.value || 0) }))}
        />
        <button
          className="px-2 py-1 border border-gray-700 text-gray-300"
          onClick={() => { const d = defaultMapping(); setCfg(d); saveMapping(d); }}
        >Reset Defaults</button>
      </div>
    </div>
  );
};

export default ArcadeBindingsPanel;


