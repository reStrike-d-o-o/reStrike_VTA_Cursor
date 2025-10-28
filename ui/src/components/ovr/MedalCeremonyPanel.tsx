import React, { useCallback, useEffect, useMemo, useState } from 'react';
import Button from '../atoms/Button';
import Input from '../atoms/Input';
import Label from '../atoms/Label';
import Toggle from '../atoms/Toggle';
import {
  AnthemAsset,
  FlagAnimationAsset,
  MedalCeremonyAthleteOption,
  MedalCeremonyDetail,
  MedalCeremonyDivision,
  MedalCeremonyDivisionOption,
  MedalCeremonyMedalist,
  MedalCeremonySummary,
  MedalType,
} from '../../types';
import { useMedalCeremonyStore, createEmptyDivision } from '../../stores/medalCeremonyStore';
import { pickFilePath } from '../../utils/filePicker';

const normalizeIocCode = (value?: string | null): string | null => {
  if (!value) {
    return null;
  }
  const trimmed = value.trim();
  return trimmed ? trimmed.toUpperCase() : null;
};

const formatDivisionSummary = (option: MedalCeremonyDivisionOption): string => {
  const parts = [option.category, option.gender, option.weight_class]
    .map((part) => (part ? part.trim() : ''))
    .filter(Boolean);
  return parts.join(' / ');
};

type DivisionUpdater = (division: MedalCeremonyDivision) => MedalCeremonyDivision;
type MedalistUpdater = (medalist: MedalCeremonyMedalist) => MedalCeremonyMedalist;

const MEDAL_ORDER: Array<{ type: MedalType; rank: number }> = [
  { type: 'gold', rank: 1 },
  { type: 'silver', rank: 2 },
  { type: 'bronze', rank: 3 },
  { type: 'bronze', rank: 4 },
];

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
  } = useMedalCeremonyStore((state) => ({
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
  }));

  const [localMessage, setLocalMessage] = useState<string | null>(null);
  const [snapshot, setSnapshot] = useState<string>('');

  const findFlagAssetByIoc = useCallback(
    (code?: string | null): FlagAnimationAsset | undefined => {
      const normalized = normalizeIocCode(code);
      if (!normalized) {
        return undefined;
      }
      const matches = flagAssets.filter(
        (asset: FlagAnimationAsset) => normalizeIocCode(asset.ioc_code) === normalized,
      );
      if (!matches.length) {
        return undefined;
      }
      const preferred = matches.find((asset: FlagAnimationAsset) => asset.is_default);
      return preferred ?? matches[0];
    },
    [flagAssets],
  );

  const findAnthemAssetByIoc = useCallback(
    (code?: string | null): AnthemAsset | undefined => {
      const normalized = normalizeIocCode(code);
      if (!normalized) {
        return undefined;
      }
      const matches = anthemAssets.filter(
        (asset: AnthemAsset) => normalizeIocCode(asset.ioc_code) === normalized,
      );
      if (!matches.length) {
        return undefined;
      }
      const preferred = matches.find((asset: AnthemAsset) => asset.is_default);
      return preferred ?? matches[0];
    },
    [anthemAssets],
  );

  useEffect(() => {
    void loadCeremonies();
    void loadAssets();
    void loadDivisionOptions();
  }, [loadAssets, loadCeremonies, loadDivisionOptions]);

  useEffect(() => {
    if (detail) {
      setSnapshot(JSON.stringify(detail));
    }
  }, [detail]);

  useEffect(() => {
    syncExternalState();
  }, [syncExternalState]);

  const isDirty = detail ? snapshot !== JSON.stringify(detail) : false;

  const divisionOptionItems = useMemo<MedalCeremonyDivisionOption[]>(
    () => divisionOptions,
    [divisionOptions],
  );

  const updateCeremony = useCallback(
    <K extends keyof MedalCeremonyDetail['ceremony']>(key: K, value: MedalCeremonyDetail['ceremony'][K]) => {
      if (!detail) return;
      setDetail((current: MedalCeremonyDetail) => ({
        ...current,
        ceremony: {
          ...current.ceremony,
          [key]: value,
        },
      }));
    },
    [detail, setDetail],
  );

  const updateDivision = useCallback(
    (index: number, updater: DivisionUpdater) => {
      if (!detail) return;
      setDetail((current: MedalCeremonyDetail) => {
        const divisions = [...current.divisions];
        if (!divisions[index]) {
          return current;
        }
        divisions[index] = updater({ ...divisions[index] });
        return { ...current, divisions };
      });
    },
    [detail, setDetail],
  );

  const updateMedalist = useCallback(
    (divisionIndex: number, medalIndex: number, updater: MedalistUpdater) => {
      updateDivision(divisionIndex, (division) => {
        const medalists = [...division.medalists];
        if (!medalists[medalIndex]) {
          return division;
        }
        medalists[medalIndex] = updater({ ...medalists[medalIndex] });
        return { ...division, medalists };
      });
    },
    [updateDivision],
  );

  const handleCreateNew = useCallback(() => {
    setError(null);
    setLocalMessage(null);
    void selectCeremony(null);
  }, [selectCeremony, setError]);

  const handleSave = useCallback(async () => {
    if (!detail) return;
    const id = await saveCeremony();
    if (id) {
      setLocalMessage('Medal ceremony saved.');
      await selectCeremony(id);
      syncExternalState();
    }
  }, [detail, saveCeremony, selectCeremony, syncExternalState]);

  const handleDelete = useCallback(async () => {
    if (!detail?.ceremony.id) return;
    if (!window.confirm('Delete this medal ceremony?')) {
      return;
    }
    await deleteCeremony(detail.ceremony.id);
    setLocalMessage('Medal ceremony removed.');
  }, [deleteCeremony, detail]);

  const handlePrepare = useCallback(async () => {
    if (!detail?.ceremony.id) {
      setLocalMessage('Save the ceremony before preparing.');
      return;
    }
    if (isDirty) {
      setLocalMessage('Please save changes before preparing.');
      return;
    }
    await prepareCeremony(detail.ceremony.id);
    await refreshDetail();
    syncExternalState();
    setLocalMessage('Playlist prepared.');
  }, [detail?.ceremony.id, isDirty, prepareCeremony, refreshDetail, syncExternalState]);

  const nextDivision = useMemo(
    () => preparedDivisions.find((division: MedalCeremonyDivision) => !division.played_at),
    [preparedDivisions],
  );

  const handlePlayNext = useCallback(async () => {
    if (!nextDivision) {
      return;
    }
    if (!nextDivision.id) {
      setError('Prepared division is missing an identifier.');
      return;
    }
    await emitPlaybackEvent(nextDivision);
    await markDivisionPlayed(nextDivision.id);
    syncExternalState();
    setLocalMessage(`Marked ${nextDivision.division_name} as played.`);
  }, [emitPlaybackEvent, markDivisionPlayed, nextDivision, setError, syncExternalState]);

  const handleResetPlayback = useCallback(async () => {
    if (!detail?.ceremony.id) return;
    await resetPlayback(detail.ceremony.id);
    syncExternalState();
    setLocalMessage('Playback state reset.');
  }, [detail?.ceremony.id, resetPlayback, syncExternalState]);

  const handleAddDivision = useCallback(() => {
    if (!detail) return;
    setDetail((current: MedalCeremonyDetail) => ({
      ...current,
      divisions: [...current.divisions, createEmptyDivision(current.divisions.length + 1)],
    }));
  }, [detail, setDetail]);

  const handleRemoveDivision = useCallback(
    (index: number) => {
      if (!detail) return;
      setDetail((current: MedalCeremonyDetail) => {
        const divisions = [...current.divisions];
        divisions.splice(index, 1);
        return { ...current, divisions };
      });
    },
    [detail, setDetail],
  );

  const handlePickImage = useCallback(
    async (field: 'background_path' | 'break_path') => {
      const path = await pickFilePath(['png', 'jpg', 'jpeg', 'webp']);
      if (path) {
        updateCeremony(field, path);
      }
    },
    [updateCeremony],
  );

  const renderMedalist = useCallback(
    (divisionIndex: number, medalist: MedalCeremonyMedalist, medalIndex: number) => {
      const division = detail?.divisions[divisionIndex];
      const divisionName = division?.division_name ?? '';
      const athleteList: MedalCeremonyAthleteOption[] = divisionName
        ? athleteOptions[divisionName.trim().toLowerCase()] ?? []
        : [];
      const selectedFlagAssetId: string = medalist.flag_asset ?? '';
      const selectedAnthemAssetId: string = medalist.anthem_asset ?? '';

      const handleAthleteSelect = (value: string) => {
        const selected = athleteList.find(
          (athlete: MedalCeremonyAthleteOption) => String(athlete.id) === value,
        );
        if (!selected) {
          return;
        }
        const inferredIoc = normalizeIocCode(selected.ioc_code || selected.country_code || null);
        const inferredFlag = findFlagAssetByIoc(inferredIoc);
        const inferredAnthem = findAnthemAssetByIoc(inferredIoc);

        updateMedalist(divisionIndex, medalIndex, (current) => {
          const next: MedalCeremonyMedalist = {
            ...current,
            athlete_id: selected.id,
            athlete_name: selected.full_name,
            athlete_short_name: selected.short_name ?? null,
            ioc_code: inferredIoc,
          };
          if (!current.flag_asset && inferredFlag?.id) {
            next.flag_asset = inferredFlag.id;
          }
          if (!current.anthem_asset && inferredAnthem?.id) {
            next.anthem_asset = inferredAnthem.id;
          }
          return next;
        });
      };

      return (
        <div key={medalist.id ?? `${divisionIndex}-${medalIndex}`} className="rounded border border-gray-800 bg-gray-950/40 p-3 space-y-3">
          <div className="flex items-center justify-between">
            <span className="text-sm font-semibold text-gray-100">{MEDAL_ORDER[medalIndex].type.toUpperCase()}</span>
            {medalist.athlete_id && <span className="text-xs text-gray-500">#{medalist.athlete_id}</span>}
          </div>
          <div className="grid grid-cols-1 gap-3 md:grid-cols-2">
            <div>
              <Label>Medalist name</Label>
              <Input
                value={medalist.athlete_name}
                onChange={(event) =>
                  updateMedalist(divisionIndex, medalIndex, (current) => ({
                    ...current,
                    athlete_name: event.target.value,
                  }))
                }
              />
            </div>
            <div>
              <Label>Select athlete</Label>
              <select
                className="w-full rounded-md border border-gray-700 bg-gray-900 px-3 py-2 text-sm text-gray-200"
                value={medalist.athlete_id ? String(medalist.athlete_id) : ''}
                onFocus={() => {
                  if (divisionName) void loadAthletesForDivision(divisionName);
                }}
                onChange={(event) => handleAthleteSelect(event.target.value)}
              >
                <option value="">Manual entry</option>
                {athleteList.map((athlete: MedalCeremonyAthleteOption) => (
                  <option key={athlete.id} value={athlete.id}>
                    {athlete.full_name}
                  </option>
                ))}
              </select>
            </div>
            <div>
              <Label>Flag animation</Label>
              <select
                className="w-full rounded-md border border-gray-700 bg-gray-900 px-3 py-2 text-sm text-gray-200"
                value={selectedFlagAssetId}
                onChange={(event) =>
                  updateMedalist(divisionIndex, medalIndex, (current) => ({
                    ...current,
                    flag_asset: event.target.value || null,
                  }))
                }
              >
                <option value="">None</option>
                {flagAssets
                  .filter(
                    (asset: FlagAnimationAsset): asset is FlagAnimationAsset & { id: string } =>
                      typeof asset.id === 'string' && asset.id.length > 0,
                  )
                  .map((asset) => (
                    <option key={asset.id} value={asset.id}>
                      {asset.display_name || asset.file_name}
                    </option>
                  ))}
              </select>
            </div>
            <div>
              <Label>Anthem</Label>
              <select
                className="w-full rounded-md border border-gray-700 bg-gray-900 px-3 py-2 text-sm text-gray-200"
                value={selectedAnthemAssetId}
                onChange={(event) =>
                  updateMedalist(divisionIndex, medalIndex, (current) => ({
                    ...current,
                    anthem_asset: event.target.value || null,
                  }))
                }
              >
                <option value="">None</option>
                {anthemAssets
                  .filter(
                    (asset: AnthemAsset): asset is AnthemAsset & { id: string } =>
                      typeof asset.id === 'string' && asset.id.length > 0,
                  )
                  .map((asset) => (
                    <option key={asset.id} value={asset.id}>
                      {asset.display_name || asset.file_name}
                    </option>
                  ))}
              </select>
            </div>
          </div>
        </div>
      );
    },
    [athleteOptions, detail, findAnthemAssetByIoc, findFlagAssetByIoc, loadAthletesForDivision, updateMedalist],
  );

  if (!detail) {
    return <div className="theme-card p-6">Loading medal ceremonies...</div>;
  }

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
        <div className="theme-card p-4 space-y-4">
          <div className="flex items-center justify-between">
            <h3 className="text-lg font-semibold text-gray-100">Medal Ceremonies</h3>
            <Button variant="primary" size="sm" onClick={handleCreateNew}>
              New
            </Button>
          </div>
          <div className="space-y-2 max-h-[520px] overflow-y-auto pr-1">
            {ceremonies.map((ceremony: MedalCeremonySummary) => (
              <button
                key={ceremony.id}
                type="button"
                onClick={() => selectCeremony(ceremony.id)}
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
            {!ceremonies.length && (
              <div className="rounded-md border border-dashed border-gray-700 bg-gray-900/80 px-3 py-8 text-center text-sm text-gray-400">
                No medal ceremonies yet. Create a new ceremony to begin.
              </div>
            )}
          </div>
        </div>

        <div className="space-y-6">
          <div className="theme-card p-6 space-y-6">
            <div className="flex flex-wrap items-start justify-between gap-4">
              <div>
                <h3 className="text-xl font-semibold text-gray-100">Ceremony configuration</h3>
                <p className="text-xs text-gray-400">
                  {detail.ceremony.prepared_at
                    ? `Prepared at ${detail.ceremony.prepared_at}`
                    : 'Not prepared'}
                </p>
              </div>
              <div className="flex flex-wrap gap-2">
                <Button variant="secondary" size="sm" onClick={() => refreshDetail()}>
                  Refresh
                </Button>
                <Button variant="primary" size="sm" disabled={saving} onClick={handleSave}>
                  {saving ? 'Saving...' : 'Save'}
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
                <Label htmlFor="ceremony-name">Tournament name</Label>
                <Input
                  id="ceremony-name"
                  value={detail.ceremony.name}
                  onChange={(event) => updateCeremony('name', event.target.value)}
                />
              </div>
              <div className="flex flex-col gap-2">
                <Label>Background (Full HD)</Label>
                <div className="flex gap-2">
                  <Input value={detail.ceremony.background_path ?? ''} readOnly className="flex-1" />
                  <Button variant="secondary" size="sm" onClick={() => handlePickImage('background_path')}>
                    Open
                  </Button>
                  {detail.ceremony.background_path && (
                    <Button
                      variant="secondary"
                      size="sm"
                      onClick={() => updateCeremony('background_path', null)}
                    >
                      Clear
                    </Button>
                  )}
                </div>
              </div>
              <div className="flex flex-col gap-2">
                <Label>Break image (Full HD)</Label>
                <div className="flex gap-2">
                  <Input value={detail.ceremony.break_path ?? ''} readOnly className="flex-1" />
                  <Button variant="secondary" size="sm" onClick={() => handlePickImage('break_path')}>
                    Open
                  </Button>
                  {detail.ceremony.break_path && (
                    <Button
                      variant="secondary"
                      size="sm"
                      onClick={() => updateCeremony('break_path', null)}
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
                  onChange={(event) => updateCeremony('animation_duration', Math.max(1000, Number(event.target.value) || 0))}
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
                  onChange={(event) => updateCeremony('animation_speed', Math.max(0.1, Number(event.target.value) || 0))}
                />
              </div>
              <div>
                <Label htmlFor="photo-time">Photo time (seconds)</Label>
                <Input
                  id="photo-time"
                  type="number"
                  min={0}
                  value={detail.ceremony.photo_time}
                  onChange={(event) => updateCeremony('photo_time', Math.max(0, Number(event.target.value) || 0))}
                />
              </div>
              <div className="flex items-end">
                <Toggle
                  label="Show external screen"
                  checked={detail.ceremony.show_external}
                  onChange={(event) => {
                    const enabled = event.currentTarget.checked;
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
                Unsaved changes detected. Save before preparing the playlist.
              </div>
            )}

            <div className="flex flex-wrap gap-3">
              <Button variant="primary" disabled={loading || isDirty || !detail.ceremony.id} onClick={handlePrepare}>
                Prepare ceremony
              </Button>
              <Button variant="secondary" disabled={!detail.ceremony.id} onClick={handleResetPlayback}>
    Reset playback
              </Button>
              <Button variant="success" disabled={!nextDivision} onClick={handlePlayNext}>
                Play next
              </Button>
            </div>

            <div className="space-y-4">
              {detail.divisions.map((division: MedalCeremonyDivision, index: number) => {
                const athletePool = division.division_name
                  ? athleteOptions[division.division_name.trim().toLowerCase()] ?? []
                  : [];
                return (
                  <div key={division.id ?? `division-${index}`} className="rounded-lg border border-gray-700 bg-gray-900/70 p-4 space-y-4">
                    <div className="flex flex-wrap items-center justify-between gap-3 border-b border-gray-700 pb-3">
                      <div className="flex-1 space-y-2">
                        <Label>Division name</Label>
                        <Input
                          value={division.division_name}
                          onChange={(event) =>
                            updateDivision(index, (current) => ({
                              ...current,
                              division_name: event.target.value,
                            }))
                          }
                          onBlur={() => {
                            if (division.division_name.trim()) {
                              void loadAthletesForDivision(division.division_name);
                            }
                          }}
                          list={`division-options-${index}`}
                        />
                        <datalist id={`division-options-${index}`}>
                          {divisionOptionItems.map((option: MedalCeremonyDivisionOption) => {
                            const summary = formatDivisionSummary(option);
                            const label = summary ? `${option.name} (${summary})` : option.name;
                            return (
                              <option
                                key={`${option.name}-${option.weight_class ?? 'none'}`}
                                value={option.name}
                              >
                                {label}
                              </option>
                            );
                          })}
                        </datalist>
                      </div>
                      <div className="flex items-end gap-2">
                        <div>
                          <Label>Order #</Label>
                          <Input
                            type="number"
                            min={1}
                            value={division.order_index}
                            onChange={(event) =>
                              updateDivision(index, (current) => ({
                                ...current,
                                order_index: Math.max(1, Number(event.target.value) || 1),
                              }))
                            }
                          />
                        </div>
                        <Button
                          variant="secondary"
                          size="sm"
                          disabled={detail.divisions.length === 1}
                          onClick={() => handleRemoveDivision(index)}
                        >
                          Remove
                        </Button>
                      </div>
                    </div>

                    <div className="space-y-3">
                      {division.medalists.map((medalist: MedalCeremonyMedalist, medalIndex: number) =>
                        renderMedalist(index, medalist, medalIndex),
                      )}
                    </div>
                  </div>
                );
              })}
            </div>

            <Button variant="secondary" onClick={handleAddDivision}>
              Add division
            </Button>
          </div>

          {preparedDivisions.length > 0 && (
            <div className="theme-card p-4">
              <h4 className="text-md font-semibold text-gray-100 mb-3">Prepared playlist</h4>
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
                    {preparedDivisions.map((division: MedalCeremonyDivision) => (
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








