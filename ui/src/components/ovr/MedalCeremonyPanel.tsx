import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import Button from '../atoms/Button';
import Input from '../atoms/Input';
import Label from '../atoms/Label';
import Toggle from '../atoms/Toggle';
import { shallow } from 'zustand/shallow';
import {
  AnthemAsset,
  FlagAnimationAsset,
  MedalCeremonyAthleteOption,
  MedalCeremonyDetail,
  MedalCeremonyDivision,
  MedalCeremonyDivisionOption,
  MedalCeremonyMedalist,
} from '../../types';
import { useMedalCeremonyStore, createEmptyDivision } from '../../stores/medalCeremonyStore';
import { pickFilePath } from '../../utils/filePicker';

const MAX_DROPDOWN_RESULTS = 50;

type DropdownOption<T> = {
  id: string;
  title: string;
  subtitle?: string;
  data: T;
};

const normalizeIocCode = (value?: string | null): string | null => {
  if (!value) return null;
  const trimmed = value.trim();
  return trimmed ? trimmed.toUpperCase() : null;
};

const buildFlagEmoji = (code?: string | null): string => {
  if (!code) return '';
  const upper = code.trim().toUpperCase();
  if (upper.length !== 2) return '';
  const base = 127397;
  return String.fromCodePoint(...upper.split('').map((char) => base + char.charCodeAt(0)));
};

interface SearchableDropdownProps<T> {
  value: string;
  placeholder?: string;
  onChange: (next: string) => void;
  onSelect: (option: DropdownOption<T>) => void;
  options: DropdownOption<T>[];
  allowCustom?: boolean;
  disabled?: boolean;
  onFocus?: () => void;
}

function SearchableDropdown<T>({
  value,
  placeholder,
  onChange,
  onSelect,
  options,
  allowCustom = true,
  disabled,
  onFocus,
}: SearchableDropdownProps<T>) {
  const [query, setQuery] = useState(value);
  const [isOpen, setIsOpen] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);
  const listRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    setQuery(value);
  }, [value]);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (!containerRef.current) return;
      if (!containerRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    };
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  const filteredOptions = useMemo(() => {
    const normalized = query.trim().toLowerCase();
    const source = normalized
      ? options.filter(
          (option) =>
            option.title.toLowerCase().includes(normalized) ||
            option.subtitle?.toLowerCase().includes(normalized),
        )
      : options;
    return source.slice(0, MAX_DROPDOWN_RESULTS);
  }, [options, query]);

  const handleSelect = (option: DropdownOption<T>) => {
    setQuery(option.title);
    onChange(option.title);
    onSelect(option);
    setIsOpen(false);
  };

  return (
    <div className="relative" ref={containerRef}>
      <input
        className="w-full rounded-md border border-gray-700 bg-gray-900 px-3 py-2 text-sm text-gray-200 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500 disabled:cursor-not-allowed disabled:opacity-50"
        value={query}
        placeholder={placeholder}
        onFocus={() => {
          if (disabled) return;
          setIsOpen(true);
          onFocus?.();
        }}
        onChange={(event) => {
          const nextValue = event.target.value;
          setQuery(nextValue);
          onChange(nextValue);
          if (!isOpen) {
            setIsOpen(true);
          }
        }}
        onKeyDown={(event) => {
          if (event.key === 'Escape') {
            setIsOpen(false);
          }
          if (event.key === 'Enter' && allowCustom) {
            setIsOpen(false);
          }
        }}
        disabled={disabled}
      />
      {isOpen && filteredOptions.length > 0 && (
        <div
          ref={listRef}
          className="absolute z-40 mt-1 max-h-56 w-full overflow-auto rounded-md border border-gray-700 bg-gray-900 shadow-lg"
        >
          {filteredOptions.map((option) => (
            <button
              key={option.id}
              type="button"
              className="flex w-full flex-col items-start gap-0.5 px-3 py-2 text-left text-sm text-gray-200 hover:bg-gray-800 focus:bg-gray-800 focus:outline-none"
              onMouseDown={(event) => {
                event.preventDefault();
                handleSelect(option);
              }}
            >
              <span className="font-medium text-gray-100">{option.title}</span>
              {option.subtitle && <span className="text-xs text-gray-400">{option.subtitle}</span>}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}

const normalizeDivisionOrders = (divisions: MedalCeremonyDivision[]): MedalCeremonyDivision[] =>
  divisions.map((division, index) => ({
    ...division,
    order_index: index + 1,
  }));

const assetLabel = (asset: FlagAnimationAsset | AnthemAsset | null | undefined): string => {
  if (!asset) return 'None';
  return (
    asset.display_name ||
    asset.file_name ||
    asset.file_path ||
    asset.id ||
    'Unnamed asset'
  );
};

const MedalCeremonyPanel: React.FC = () => {
  const {
    ceremonies,
    selectedId,
    detail,
    divisionOptions,
    flagAssets,
    anthemAssets,
    preparedDivisions,
    athleteOptions,
    loading,
    saving,
    error,
    loadCeremonies,
    selectCeremony,
    setDetail,
    saveCeremony,
    deleteCeremony,
    prepareCeremony,
    markDivisionPlayed,
    resetPlayback,
    toggleExternalDisplay,
    loadAssets,
    loadDivisionOptions,
    loadAthletesForDivision,
    refreshDetail,
    setError,
    syncExternalState,
    emitPlaybackEvent,
  } = useMedalCeremonyStore(
    (state) => ({
      ceremonies: state.ceremonies,
      selectedId: state.selectedId,
      detail: state.detail,
      divisionOptions: state.divisionOptions,
      flagAssets: state.flagAssets,
      anthemAssets: state.anthemAssets,
      preparedDivisions: state.preparedDivisions,
      athleteOptions: state.athleteOptions,
      loading: state.loading,
      saving: state.saving,
      error: state.error,
      loadCeremonies: state.loadCeremonies,
      selectCeremony: state.selectCeremony,
      setDetail: state.setDetail,
      saveCeremony: state.saveCeremony,
    deleteCeremony: state.deleteCeremony,
    prepareCeremony: state.prepareCeremony,
    markDivisionPlayed: state.markDivisionPlayed,
    resetPlayback: state.resetPlayback,
    toggleExternalDisplay: state.toggleExternalDisplay,
    loadAssets: state.loadAssets,
    loadDivisionOptions: state.loadDivisionOptions,
    loadAthletesForDivision: state.loadAthletesForDivision,
    refreshDetail: state.refreshDetail,
    setError: state.setError,
    syncExternalState: state.syncExternalState,
    emitPlaybackEvent: state.emitPlaybackEvent,
  }),
  shallow,
);

  const [localMessage, setLocalMessage] = useState<string | null>(null);
  const initializedRef = useRef(false);
  const currentHash = useMemo(() => (detail ? JSON.stringify(detail) : ''), [detail]);
  const [savedHash, setSavedHash] = useState(currentHash);
  const invokeExternalWindow = useCallback(async (action: 'open' | 'close') => {
    if (typeof window === 'undefined') return;
    const core = (window as any).__TAURI__?.core;
    if (!core?.invoke) return;
    try {
      await core.invoke(
        action === 'open'
          ? 'medal_ceremony_open_external_window'
          : 'medal_ceremony_close_external_window',
      );
    } catch (error) {
      console.warn('External window command failed', error);
    }
  }, []);

  useEffect(() => {
    void loadCeremonies();
    void loadAssets();
    void loadDivisionOptions();
  }, [loadCeremonies, loadAssets, loadDivisionOptions]);

  useEffect(() => {
    syncExternalState();
  }, [syncExternalState]);

  useEffect(() => {
    if (!initializedRef.current && ceremonies.length > 0) {
      initializedRef.current = true;
      void selectCeremony(ceremonies[0].id);
    }
  }, [ceremonies, selectCeremony]);

  useEffect(() => {
    if (!detail) {
      setSavedHash('');
      return;
    }
    if (detail.ceremony.id) {
      setSavedHash(JSON.stringify(detail));
    }
  }, [detail?.ceremony.id]);

  const isDirty = detail ? (detail.ceremony.id ? currentHash !== savedHash : true) : false;

  const divisionOptionItems = useMemo<DropdownOption<MedalCeremonyDivisionOption>[]>(() => {
    return divisionOptions.map((option, index) => {
      const summaryParts = [
        option.category?.trim(),
        option.gender?.trim(),
        option.weight_class?.trim(),
      ].filter(Boolean);
      return {
        id: `${option.name}-${index}`,
        title: option.name,
        subtitle: summaryParts.join(' • ') || undefined,
        data: option,
      };
    });
  }, [divisionOptions]);

  const findFlagAsset = useCallback(
    (code?: string | null): FlagAnimationAsset | undefined => {
      const normalized = normalizeIocCode(code);
      if (!normalized) return undefined;
      const matches = flagAssets.filter(
        (asset) => normalizeIocCode(asset.ioc_code) === normalized,
      );
      if (!matches.length) return undefined;
      const preferred = matches.find((asset) => asset.is_default);
      return preferred || matches[0];
    },
    [flagAssets],
  );

  const findAnthemAsset = useCallback(
    (code?: string | null): AnthemAsset | undefined => {
      const normalized = normalizeIocCode(code);
      if (!normalized) return undefined;
      const matches = anthemAssets.filter(
        (asset) => normalizeIocCode(asset.ioc_code) === normalized,
      );
      if (!matches.length) return undefined;
      const preferred = matches.find((asset) => asset.is_default);
      return preferred || matches[0];
    },
    [anthemAssets],
  );

  const updateCeremonyField = useCallback(
    <K extends keyof MedalCeremonyDetail['ceremony']>(
      key: K,
      value: MedalCeremonyDetail['ceremony'][K],
    ) => {
      setDetail((current) => ({
        ...current,
        ceremony: {
          ...current.ceremony,
          [key]: value,
        },
      }));
    },
    [setDetail],
  );

  const updateDivision = useCallback(
    (
      index: number,
      updater: (division: MedalCeremonyDivision) => MedalCeremonyDivision,
    ) => {
      setDetail((current) => {
        if (!current.divisions[index]) {
          return current;
        }
        const nextDivisions = [...current.divisions];
        nextDivisions[index] = updater({ ...nextDivisions[index] });
        return {
          ...current,
          divisions: nextDivisions,
        };
      });
    },
    [setDetail],
  );

  const updateMedalist = useCallback(
    (
      divisionIndex: number,
      medalIndex: number,
      updater: (medalist: MedalCeremonyMedalist) => MedalCeremonyMedalist,
    ) => {
      updateDivision(divisionIndex, (division) => {
        if (!division.medalists[medalIndex]) {
          return division;
        }
        const nextMedalists = [...division.medalists];
        nextMedalists[medalIndex] = updater({ ...nextMedalists[medalIndex] });
        return {
          ...division,
          medalists: nextMedalists,
        };
      });
    },
    [updateDivision],
  );

  const handleCreateNew = useCallback(async () => {
    setLocalMessage(null);
    setError(null);
    await selectCeremony(null);
    setSavedHash('');
  }, [selectCeremony, setError]);

  const handleSelectCeremony = useCallback(
    async (id: string) => {
      setLocalMessage(null);
      setError(null);
      await selectCeremony(id);
    },
    [selectCeremony, setError],
  );

  const handleSave = useCallback(async () => {
    if (!detail) return;
    const id = await saveCeremony();
    if (id) {
      const updated = useMedalCeremonyStore.getState().detail;
      if (updated) {
        setSavedHash(JSON.stringify(updated));
        setLocalMessage('Medal ceremony saved successfully.');
      }
    }
  }, [detail, saveCeremony]);

  const handleDelete = useCallback(async () => {
    if (!detail?.ceremony.id) return;
    if (!window.confirm('Remove this medal ceremony? This action cannot be undone.')) {
      return;
    }
    await deleteCeremony(detail.ceremony.id);
    setSavedHash('');
    setLocalMessage('Medal ceremony deleted.');
  }, [deleteCeremony, detail?.ceremony.id]);

  const handlePrepare = useCallback(async () => {
    if (!detail?.ceremony.id) {
      setLocalMessage('Please save the ceremony before preparing a playlist.');
      return;
    }
    if (isDirty) {
      setLocalMessage('There are unsaved changes. Save before preparing.');
      return;
    }
    try {
      await prepareCeremony(detail.ceremony.id);
      await refreshDetail();
      setLocalMessage('Ceremony playlist prepared.');
    } catch (error) {
      console.error('Failed to prepare ceremony', error);
    }
  }, [detail?.ceremony.id, isDirty, prepareCeremony, refreshDetail]);

  const nextDivision = useMemo(() => {
    return preparedDivisions.find((division) => !division.played_at);
  }, [preparedDivisions]);

  const handlePlayNext = useCallback(async () => {
    if (!nextDivision?.id) return;
    await invokeExternalWindow('open');
    if (detail && !detail.ceremony.show_external) {
      await toggleExternalDisplay(true);
    }
    syncExternalState();
    await emitPlaybackEvent(nextDivision);
    await markDivisionPlayed(nextDivision.id);
    setLocalMessage(`Marked ${nextDivision.division_name} as played.`);
  }, [detail, emitPlaybackEvent, invokeExternalWindow, markDivisionPlayed, nextDivision, syncExternalState, toggleExternalDisplay]);

  const handleResetPlayback = useCallback(async () => {
    if (!detail?.ceremony.id) return;
    await resetPlayback(detail.ceremony.id);
    setLocalMessage('Playback state reset.');
  }, [detail?.ceremony.id, resetPlayback]);

  const getFlagOptions = useCallback(
    (code?: string | null): { preferred: FlagAnimationAsset[]; others: FlagAnimationAsset[] } => {
      const normalized = normalizeIocCode(code);
      if (!normalized) {
        return { preferred: [], others: flagAssets };
      }
      const preferred = flagAssets.filter(
        (asset) => normalizeIocCode(asset.ioc_code) === normalized,
      );
      const preferredIds = new Set(preferred.map((asset) => asset.id));
      const others = flagAssets.filter((asset) => !preferredIds.has(asset.id));
      return { preferred, others };
    },
    [flagAssets],
  );

  const getAnthemOptions = useCallback(
    (code?: string | null): { preferred: AnthemAsset[]; others: AnthemAsset[] } => {
      const normalized = normalizeIocCode(code);
      if (!normalized) {
        return { preferred: [], others: anthemAssets };
      }
      const preferred = anthemAssets.filter(
        (asset) => normalizeIocCode(asset.ioc_code) === normalized,
      );
      const preferredIds = new Set(preferred.map((asset) => asset.id));
      const others = anthemAssets.filter((asset) => !preferredIds.has(asset.id));
      return { preferred, others };
    },
    [anthemAssets],
  );

  const preparedPlaylistLabel = useMemo(() => {
    if (!detail?.ceremony.prepared_at) return 'Not prepared';
    return `Prepared at ${detail.ceremony.prepared_at}`;
  }, [detail?.ceremony.prepared_at]);

  return (
    <div className="space-y-6">
      {(error || localMessage) && (
        <div className="space-y-2">
          {error && (
            <div className="rounded-md border border-red-500/40 bg-red-500/10 px-3 py-2 text-sm text-red-200">
              {error}
            </div>
          )}
          {localMessage && (
            <div className="rounded-md border border-blue-500/40 bg-blue-500/10 px-3 py-2 text-sm text-blue-200">
              {localMessage}
            </div>
          )}
        </div>
      )}

      <div className="grid grid-cols-1 gap-6 lg:grid-cols-[320px,1fr]">
        <div className="theme-card space-y-4 p-4 shadow-lg">
          <div className="flex items-center justify-between">
            <h3 className="text-lg font-semibold text-gray-100">Medal Ceremonies</h3>
            <Button variant="primary" size="sm" onClick={handleCreateNew}>
              New
            </Button>
          </div>
          <div className="max-h-[520px] space-y-2 overflow-y-auto pr-1">
            {ceremonies.map((ceremony) => (
              <button
                key={ceremony.id}
                type="button"
                onClick={() => handleSelectCeremony(ceremony.id)}
                className={`w-full rounded-md border px-3 py-2 text-left text-sm transition ${
                  selectedId === ceremony.id
                    ? 'border-blue-500 bg-blue-500/20 text-blue-100'
                    : 'border-gray-700 bg-gray-900/70 text-gray-200 hover:border-blue-500/80 hover:bg-gray-800'
                }`}
              >
                <div className="font-medium">{ceremony.name}</div>
                <div className="text-xs text-gray-400">
                  {ceremony.prepared_at ? `Prepared ${ceremony.prepared_at}` : 'Not prepared'}
                </div>
              </button>
            ))}
            {ceremonies.length === 0 && (
              <div className="rounded-md border border-dashed border-gray-700 bg-gray-900/80 px-3 py-8 text-center text-sm text-gray-400">
                No medal ceremonies yet. Create a new ceremony to get started.
              </div>
            )}
          </div>
        </div>

        <div className="space-y-6">
          <div className="theme-card space-y-6 p-6 shadow-lg">
            {detail ? (
              <>
                <div className="flex flex-wrap items-start justify-between gap-4">
                  <div>
                    <h3 className="text-xl font-semibold text-gray-100">
                      Medal Ceremony Configuration
                    </h3>
                    <p className="text-xs text-gray-400">{preparedPlaylistLabel}</p>
                  </div>
                  <div className="flex flex-wrap gap-2">
                    <Button
                      variant="secondary"
                      size="sm"
                      onClick={() => {
                        setLocalMessage(null);
                        setError(null);
                        void refreshDetail();
                      }}
                    >
                      Refresh
                    </Button>
                    <Button
                      variant="primary"
                      size="sm"
                      disabled={saving || !isDirty}
                      onClick={handleSave}
                    >
                      {saving ? 'Saving…' : 'Save'}
                    </Button>
                    {detail.ceremony.id && (
                      <Button variant="danger" size="sm" onClick={handleDelete}>
                        Delete
                      </Button>
                    )}
                  </div>
                </div>

                <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
                  <div className="md:col-span-2">
                    <Label htmlFor="medal-ceremony-name">Tournament name</Label>
                    <Input
                      id="medal-ceremony-name"
                      value={detail.ceremony.name}
                      onChange={(event) => updateCeremonyField('name', event.target.value)}
                    />
                  </div>
                  <div className="flex flex-col gap-2">
                    <Label>Background image (Full HD)</Label>
                    <div className="flex gap-2">
                      <Input
                        value={detail.ceremony.background_path ?? ''}
                        readOnly
                        placeholder="Select background image"
                        className="flex-1"
                      />
                      <Button
                        variant="secondary"
                        size="sm"
                        onClick={async () => {
                          const path = await pickFilePath(['png', 'jpg', 'jpeg', 'webp']);
                          if (path) {
                            updateCeremonyField('background_path', path);
                          }
                        }}
                      >
                        Open
                      </Button>
                      {detail.ceremony.background_path && (
                        <Button
                          variant="secondary"
                          size="sm"
                          onClick={() => updateCeremonyField('background_path', null)}
                        >
                          Clear
                        </Button>
                      )}
                    </div>
                  </div>
                  <div className="flex flex-col gap-2">
                    <Label>Break image (Full HD)</Label>
                    <div className="flex gap-2">
                      <Input
                        value={detail.ceremony.break_path ?? ''}
                        readOnly
                        placeholder="Select break image"
                        className="flex-1"
                      />
                      <Button
                        variant="secondary"
                        size="sm"
                        onClick={async () => {
                          const path = await pickFilePath(['png', 'jpg', 'jpeg', 'webp']);
                          if (path) {
                            updateCeremonyField('break_path', path);
                          }
                        }}
                      >
                        Open
                      </Button>
                      {detail.ceremony.break_path && (
                        <Button
                          variant="secondary"
                          size="sm"
                          onClick={() => updateCeremonyField('break_path', null)}
                        >
                          Clear
                        </Button>
                      )}
                    </div>
                  </div>
                  <div>
                    <Label htmlFor="animation-duration">Animation duration (ms)</Label>
                    <Input
                      id="animation-duration"
                      type="number"
                      min={1000}
                      step={500}
                      value={detail.ceremony.animation_duration}
                      onChange={(event) =>
                        updateCeremonyField(
                          'animation_duration',
                          Math.max(1000, Number(event.target.value) || 0),
                        )
                      }
                    />
                  </div>
                  <div>
                    <Label htmlFor="animation-speed">Animation speed</Label>
                    <Input
                      id="animation-speed"
                      type="number"
                      min={0.1}
                      step={0.1}
                      value={detail.ceremony.animation_speed}
                      onChange={(event) =>
                        updateCeremonyField(
                          'animation_speed',
                          Math.max(0.1, Number(event.target.value) || 0),
                        )
                      }
                    />
                  </div>
                  <div>
                    <Label htmlFor="photo-time">Photo time (seconds)</Label>
                    <Input
                      id="photo-time"
                      type="number"
                      min={0}
                      step={1}
                      value={detail.ceremony.photo_time}
                      onChange={(event) =>
                        updateCeremonyField(
                          'photo_time',
                          Math.max(0, Number(event.target.value) || 0),
                        )
                      }
                    />
                  </div>
                  <div className="flex flex-col justify-end">
                    <Toggle
                    label="Show external screen"
                    checked={detail.ceremony.show_external}
                    onChange={(event) => {
                      const enabled = event.currentTarget.checked;
                      void (enabled
                        ? invokeExternalWindow('open')
                        : invokeExternalWindow('close'));
                      void toggleExternalDisplay(enabled);
                      if (enabled) {
                        syncExternalState();
                      }
                    }}
                  />
                </div>
                </div>

                {isDirty && (
                  <div className="rounded-md border border-amber-500/30 bg-amber-500/10 px-3 py-2 text-xs text-amber-200">
                    You have unsaved changes. Save before preparing a ceremony playlist.
                  </div>
                )}

                <div className="flex flex-wrap gap-3">
                  <Button
                    variant="primary"
                    disabled={loading || isDirty || !detail.ceremony.id}
                    onClick={handlePrepare}
                  >
                    Prepare ceremony
                  </Button>
                  <Button
                    variant="secondary"
                    disabled={!detail.ceremony.id}
                    onClick={handleResetPlayback}
                  >
                    Reset playback
                  </Button>
                  <Button
                    variant="success"
                    disabled={!nextDivision}
                    onClick={handlePlayNext}
                  >
                    Play next
                  </Button>
                </div>

                <div className="space-y-4">
                  {detail.divisions.map((division, index) => {
                    const divisionName = division.division_name || '';
                    const divisionKey = divisionName.trim().toLowerCase();
                    const athletesForDivision =
                      athleteOptions[divisionKey] ?? [];
                    const athleteOptionsList: DropdownOption<MedalCeremonyAthleteOption>[] =
                      athletesForDivision.map((athlete) => ({
                        id: String(athlete.id),
                        title: athlete.full_name,
                        subtitle: [athlete.short_name, athlete.ioc_code]
                          .filter(Boolean)
                          .join(' • '),
                        data: athlete,
                      }));

                    return (
                      <div
                        key={division.id ?? `division-${index}`}
                        className="rounded-lg border border-gray-700 bg-gray-900/70 p-4 shadow-inner"
                      >
                        <div className="flex flex-wrap items-center justify-between gap-3 border-b border-gray-700 pb-3">
                          <div className="w-full space-y-3 md:w-auto md:flex-1 md:space-y-0">
                            <div>
                              <Label>Division</Label>
                              <SearchableDropdown<MedalCeremonyDivisionOption>
                                value={divisionName}
                                options={divisionOptionItems}
                                placeholder="Select or type division"
                                onFocus={() => {
                                  if (divisionName.trim().length > 0) {
                                    void loadAthletesForDivision(divisionName);
                                  }
                                }}
                                onChange={(text) => {
                                  updateDivision(index, (current) => ({
                                    ...current,
                                    division_name: text,
                                  }));
                                }}
                                onSelect={(option) => {
                                  updateDivision(index, (current) => ({
                                    ...current,
                                    division_name: option.data.name,
                                  }));
                                  void loadAthletesForDivision(option.data.name);
                                }}
                              />
                            </div>
                          </div>
                          <div className="flex items-end gap-2">
                            <div className="w-24">
                              <Label>Order #</Label>
                              <Input
                                type="number"
                                value={division.order_index}
                                min={1}
                                onChange={(event) => {
                                  const nextValue = Math.max(
                                    1,
                                    Number(event.target.value) || 1,
                                  );
                                  updateDivision(index, (current) => ({
                                    ...current,
                                    order_index: nextValue,
                                  }));
                                }}
                              />
                            </div>
                            <Button
                              variant="secondary"
                              size="sm"
                              disabled={detail.divisions.length === 1}
                              onClick={() => {
                                if (detail.divisions.length === 1) return;
                                setDetail((current) => {
                                  const next = [...current.divisions];
                                  next.splice(index, 1);
                                  return {
                                    ...current,
                                    divisions: normalizeDivisionOrders(next),
                                  };
                                });
                              }}
                            >
                              Remove
                            </Button>
                          </div>
                        </div>

                        <div className="mt-4 space-y-3">
                          {division.medalists.map((medalist, medalIndex) => {
                            const iocCode = normalizeIocCode(medalist.ioc_code);
                            const { preferred: matchingFlags, others: otherFlags } =
                              getFlagOptions(iocCode);
                            const { preferred: matchingAnthems, others: otherAnthems } =
                              getAnthemOptions(iocCode);
                            const medalLabel =
                              medalist.medal_type.toUpperCase() === 'GOLD'
                                ? 'Gold'
                                : medalist.medal_type.toUpperCase() === 'SILVER'
                                ? 'Silver'
                                : medalist.medal_rank <= 3
                                ? 'Bronze'
                                : 'Bronze';

                            const athleteName = medalist.athlete_name;
                            const flagEmoji =
                              (iocCode && iocCode.length === 2
                                ? buildFlagEmoji(iocCode)
                                : '') || '';

                            return (
                              <div
                                key={medalist.id ?? `medalist-${index}-${medalIndex}`}
                                className="rounded-md border border-gray-800 bg-gray-950/40 p-3"
                              >
                                <div className="mb-2 flex items-center justify-between">
                                  <span className="text-sm font-semibold text-gray-100">
                                    {medalLabel}
                                  </span>
                                  {medalist.athlete_id && (
                                    <span className="text-xs text-gray-500">
                                      #{medalist.athlete_id}
                                    </span>
                                  )}
                                </div>
                                <div className="grid grid-cols-1 gap-3 md:grid-cols-2">
                                  <div>
                                    <Label>Medalist</Label>
                                    <SearchableDropdown<MedalCeremonyAthleteOption>
                                      value={athleteName}
                                      options={athleteOptionsList}
                                      placeholder="Search athlete"
                                      allowCustom
                                      onFocus={() => {
                                        if (divisionName.trim().length > 0) {
                                          void loadAthletesForDivision(divisionName);
                                        }
                                      }}
                                      onChange={(text) => {
                                        updateMedalist(index, medalIndex, (current) => ({
                                          ...current,
                                          athlete_name: text,
                                          athlete_short_name: null,
                                          athlete_id: null,
                                        }));
                                      }}
                                      onSelect={(option) => {
                                        const selected = option.data;
                                        const normalized = normalizeIocCode(
                                          selected.ioc_code || selected.country_code,
                                        );
                                        const flagAsset = findFlagAsset(normalized);
                                        const anthemAsset = findAnthemAsset(normalized);
                                        updateMedalist(index, medalIndex, (current) => ({
                                          ...current,
                                          athlete_id: selected.id,
                                          athlete_name: selected.full_name,
                                          athlete_short_name: selected.short_name ?? null,
                                          ioc_code: normalized,
                                          flag_asset: flagAsset?.id ?? null,
                                          anthem_asset: anthemAsset?.id ?? null,
                                        }));
                                      }}
                                    />
                                  </div>
                                  <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
                                    <div>
                                      <Label>IOC code</Label>
                                      <div className="rounded-md border border-gray-800 bg-gray-900 px-3 py-2 text-sm text-gray-200">
                                        {iocCode || 'N/A'}{' '}
                                        {flagEmoji && <span className="ml-1">{flagEmoji}</span>}
                                      </div>
                                    </div>
                                    <div>
                                      <Label>Flag animation</Label>
                                      <select
                                        className="w-full rounded-md border border-gray-700 bg-gray-900 px-3 py-2 text-sm text-gray-200 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
                                        value={medalist.flag_asset ?? ''}
                                        onChange={(event) => {
                                          const nextValue = event.target.value || null;
                                          updateMedalist(index, medalIndex, (current) => ({
                                            ...current,
                                            flag_asset: nextValue,
                                          }));
                                        }}
                                      >
                                        <option value="">None</option>
                                        {matchingFlags.length > 0 && (
                                          <optgroup label="Matching">
                                            {matchingFlags.map((asset) => (
                                              <option
                                                key={asset.id ?? asset.file_path}
                                                value={asset.id ?? asset.file_path}
                                              >
                                                {assetLabel(asset)}
                                              </option>
                                            ))}
                                          </optgroup>
                                        )}
                                        {otherFlags.length > 0 && (
                                          <optgroup label="Other">
                                            {otherFlags.map((asset) => (
                                              <option
                                                key={asset.id ?? asset.file_path}
                                                value={asset.id ?? asset.file_path}
                                              >
                                                {assetLabel(asset)}
                                              </option>
                                            ))}
                                          </optgroup>
                                        )}
                                      </select>
                                    </div>
                                    <div>
                                      <Label>Anthem</Label>
                                      <select
                                        className="w-full rounded-md border border-gray-700 bg-gray-900 px-3 py-2 text-sm text-gray-200 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
                                        value={medalist.anthem_asset ?? ''}
                                        onChange={(event) => {
                                          const nextValue = event.target.value || null;
                                          updateMedalist(index, medalIndex, (current) => ({
                                            ...current,
                                            anthem_asset: nextValue,
                                          }));
                                        }}
                                      >
                                        <option value="">None</option>
                                        {matchingAnthems.length > 0 && (
                                          <optgroup label="Matching">
                                            {matchingAnthems.map((asset) => (
                                              <option
                                                key={asset.id ?? asset.file_path}
                                                value={asset.id ?? asset.file_path}
                                              >
                                                {assetLabel(asset)}
                                              </option>
                                            ))}
                                          </optgroup>
                                        )}
                                        {otherAnthems.length > 0 && (
                                          <optgroup label="Other">
                                            {otherAnthems.map((asset) => (
                                              <option
                                                key={asset.id ?? asset.file_path}
                                                value={asset.id ?? asset.file_path}
                                              >
                                                {assetLabel(asset)}
                                              </option>
                                            ))}
                                          </optgroup>
                                        )}
                                      </select>
                                    </div>
                                  </div>
                                </div>
                              </div>
                            );
                          })}
                        </div>
                      </div>
                    );
                  })}
                </div>

                <Button
                  variant="secondary"
                  onClick={() => {
                    setDetail((current) => ({
                      ...current,
                      divisions: normalizeDivisionOrders([
                        ...current.divisions,
                        createEmptyDivision(current.divisions.length + 1),
                      ]),
                    }));
                  }}
                >
                  Add division
                </Button>
              </>
            ) : (
              <div className="text-sm text-gray-300">
                Select or create a medal ceremony to begin configuration.
              </div>
            )}
          </div>

          {preparedDivisions.length > 0 && (
            <div className="theme-card p-4 shadow-lg">
              <h4 className="mb-3 text-md font-semibold text-gray-100">Prepared playlist</h4>
              <div className="overflow-hidden rounded-md border border-gray-800">
                <table className="min-w-full divide-y divide-gray-800 text-sm">
                  <thead className="bg-gray-900/70 text-xs uppercase tracking-wide text-gray-400">
                    <tr>
                      <th className="px-3 py-2 text-left">Order</th>
                      <th className="px-3 py-2 text-left">Division</th>
                      <th className="px-3 py-2 text-left">Status</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-gray-800 bg-gray-950/40">
                    {preparedDivisions.map((division) => (
                      <tr key={division.id}>
                        <td className="px-3 py-2 text-gray-200">{division.order_index}</td>
                        <td className="px-3 py-2 text-gray-200">{division.division_name}</td>
                        <td className="px-3 py-2 text-gray-200">
                          {division.played_at ? (
                            <span className="text-green-400">Played</span>
                          ) : (
                            <span className="text-yellow-300">Pending</span>
                          )}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default MedalCeremonyPanel;
