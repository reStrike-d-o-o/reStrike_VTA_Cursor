import React, { useMemo } from 'react';
import Label from '../atoms/Label';
import { Select, SelectTrigger, SelectValue, SelectContent, SelectItem } from '../atoms/Select';
import { useOverlayRoutingStore } from '../../stores/overlayRoutingStore';
import type { OverlayId, OverlayTriggerType, OverlayRoutingRule } from '../../types';
import { useI18n } from '../../i18n';

const overlayOptions: { value: OverlayId; label: string }[] = [
  { value: 'none', label: 'None' },
  // Olympic theme
  { value: 'olympicScoreboard', label: 'Olympic – Scoreboard' },
  { value: 'olympicPlayers', label: 'Olympic – Players' },
  { value: 'olympicMatchResult', label: 'Olympic – Match Result' },
  { value: 'olympicWinner', label: 'Olympic – Winner' },
  { value: 'olympicVideoReplay', label: 'Olympic – Video Replay' },
  // Modern theme
  { value: 'modernScoreboard', label: 'Modern – Scoreboard' },
  { value: 'modernPlayers', label: 'Modern – Players' },
  { value: 'modernMatchResult', label: 'Modern – Match Result' },
  { value: 'modernWinner', label: 'Modern – Winner' },
  { value: 'modernVideoReplay', label: 'Modern – Video Replay' },
  // Arcade theme
  { value: 'arcadeScoreboard', label: 'Arcade – Scoreboard' },
  { value: 'arcadePlayers', label: 'Arcade – Players' },
  { value: 'arcadeMatchResult', label: 'Arcade – Match Result' },
  { value: 'arcadeWinner', label: 'Arcade – Winner' },
  { value: 'arcadeVideoReplay', label: 'Arcade – Video Replay' },
];

const actionOptions = [
  { value: 'show', label: 'Show' },
  { value: 'hide', label: 'Hide' },
  { value: 'toggle', label: 'Toggle' },
];

const triggerLabels: Record<OverlayTriggerType, string> = {
  fight_loaded: 'Fight Loaded',
  fight_ready: 'Fight Ready',
  round_start: 'Round Start',
  round_end: 'Round End',
  winner: 'Winner',
  injury_show: 'Injury Show',
  injury_hide: 'Injury Hide',
};

interface OverlayRoutingSettingsProps {
  className?: string;
}

const OverlayRoutingSettings: React.FC<OverlayRoutingSettingsProps> = ({ className = '' }) => {
  const { t } = useI18n();
  const rules = useOverlayRoutingStore((state) => state.rules);
  const setRules = useOverlayRoutingStore((state) => state.setRules);
  const updateRule = useOverlayRoutingStore((state) => state.updateRule);
  const resetToDefaults = useOverlayRoutingStore((state) => state.resetToDefaults);
  const saveToBackend = useOverlayRoutingStore((state) => state.saveToBackend);

  const sortedRules = useMemo(
    () =>
      [...rules].sort((a, b) => {
        const order: OverlayTriggerType[] = [
          'fight_loaded',
          'fight_ready',
          'round_start',
          'round_end',
          'winner',
          'injury_show',
          'injury_hide',
        ];
        return order.indexOf(a.trigger) - order.indexOf(b.trigger);
      }),
    [rules],
  );

  const handleChange = (index: number, partial: Partial<OverlayRoutingRule>) => {
    const current = sortedRules[index];
    if (!current) return;
    const next: OverlayRoutingRule = { ...current, ...partial };

    const originalIndex = rules.findIndex((r) => r.trigger === current.trigger);
    if (originalIndex >= 0) {
      updateRule(originalIndex, next);
    } else {
      const updated = [...rules, next];
      setRules(updated);
    }
  };

  return (
    <div className={className}>
      <div className="mt-6 border-t border-gray-700/60 pt-4">
        <div className="flex items-center justify-between mb-3">
          <h4 className="text-sm font-semibold text-gray-100">
            {t('ovr.routing.title', 'Overlay Routing')}
          </h4>
          <button
            type="button"
            className="text-xs text-blue-300 hover:text-blue-200 underline"
            onClick={async () => {
              resetToDefaults();
              await saveToBackend();
            }}
          >
            {t('ovr.routing.reset_defaults', 'Reset to defaults')}
          </button>
        </div>
        <p className="text-xs text-gray-400 mb-3">
          {t(
            'ovr.routing.help',
            'Choose which overlay window should react to key PSS triggers.',
          )}
        </p>

        <div className="overflow-x-auto">
          <table className="min-w-full text-xs">
            <thead>
              <tr className="text-left text-gray-400 border-b border-gray-700/60">
                <th className="py-2 pr-4 font-normal">
                  {t('ovr.routing.trigger', 'Trigger')}
                </th>
                <th className="py-2 pr-4 font-normal">
                  {t('ovr.routing.overlay', 'Overlay')}
                </th>
                <th className="py-2 pr-4 font-normal">
                  {t('ovr.routing.action', 'Action')}
                </th>
              </tr>
            </thead>
            <tbody>
              {sortedRules.map((rule, index) => (
                <tr key={rule.trigger} className="border-b border-gray-800/60">
                  <td className="py-2 pr-4 align-middle">
                    <Label className="text-[11px] text-gray-200">
                      {t(`ovr.routing.trigger.${rule.trigger}`, triggerLabels[rule.trigger])}
                    </Label>
                  </td>
                  <td className="py-2 pr-4 align-middle">
                    <Select
                      value={rule.overlay}
                      onValueChange={async (value) => {
                        handleChange(index, {
                          overlay: value as OverlayId,
                        });
                        await saveToBackend();
                      }}
                      className="w-40"
                    >
                      <SelectTrigger className="bg-gray-800/80 border border-gray-600/60 text-xs text-gray-100" isOpen={false}>
                        <SelectValue
                          value={t(
                            `ovr.routing.overlay.${rule.overlay}`,
                            overlayOptions.find((o) => o.value === rule.overlay)?.label ?? '',
                          )}
                        />
                      </SelectTrigger>
                      <SelectContent>
                        {overlayOptions.map((opt) => (
                          <SelectItem key={opt.value} value={opt.value}>
                            {t(`ovr.routing.overlay.${opt.value}`, opt.label)}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </td>
                  <td className="py-2 pr-4 align-middle">
                    <Select
                      value={rule.action}
                      onValueChange={async (value) => {
                        handleChange(index, {
                          action: value as OverlayRoutingRule['action'],
                        });
                        await saveToBackend();
                      }}
                      className="w-32"
                    >
                      <SelectTrigger className="bg-gray-800/80 border border-gray-600/60 text-xs text-gray-100" isOpen={false}>
                        <SelectValue
                          value={t(
                            `ovr.routing.action.${rule.action}`,
                            actionOptions.find((o) => o.value === rule.action)?.label ?? '',
                          )}
                        />
                      </SelectTrigger>
                      <SelectContent>
                        {actionOptions.map((opt) => (
                          <SelectItem key={opt.value} value={opt.value}>
                            {t(`ovr.routing.action.${opt.value}`, opt.label)}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
};

export default OverlayRoutingSettings;
