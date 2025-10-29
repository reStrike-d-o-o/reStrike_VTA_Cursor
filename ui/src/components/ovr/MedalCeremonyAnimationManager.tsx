import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import Lottie from 'lottie-react';
import Button from '../atoms/Button';
import Input from '../atoms/Input';
import Label from '../atoms/Label';
import Toggle from '../atoms/Toggle';
import { useMedalCeremonyStore } from '../../stores/medalCeremonyStore';
import { FlagAnimationAsset } from '../../types';
import { pickFilePath } from '../../utils/filePicker';
import { FlagImage } from '../../utils/flagUtils';
import { canInvokeTauri, invokeTauri } from '../../utils/tauriBridge';

interface FlagAssetForm {
  id?: string | null;
  ioc_code: string;
  file_name: string;
  file_path: string;
  display_name: string;
  duration_ms?: number | null;
  is_default: boolean;
}

const defaultForm: FlagAssetForm = {
  id: null,
  ioc_code: '',
  file_name: '',
  file_path: '',
  display_name: '',
  duration_ms: undefined,
  is_default: false,
};

const readAnimationFile = async (path: string): Promise<string> => {
  if (!canInvokeTauri()) {
    throw new Error('Animation preview requires the desktop runtime.');
  }
  return invokeTauri<string>('read_animation_file', { path });
};

const MedalCeremonyAnimationManager: React.FC = () => {
  const flagAssets = useMedalCeremonyStore((state) => state.flagAssets);
  const loadAssets = useMedalCeremonyStore((state) => state.loadAssets);
  const saveFlagAsset = useMedalCeremonyStore((state) => state.saveFlagAsset);
  const deleteFlagAsset = useMedalCeremonyStore((state) => state.deleteFlagAsset);
  const saving = useMedalCeremonyStore((state) => state.saving);
  const loading = useMedalCeremonyStore((state) => state.loading);
  const error = useMedalCeremonyStore((state) => state.error);
  const setError = useMedalCeremonyStore((state) => state.setError);
  const assetsInitialized = useMedalCeremonyStore((state) => state.assetsInitialized);

  const [form, setForm] = useState<FlagAssetForm>(defaultForm);
  const [message, setMessage] = useState<string | null>(null);
  const [search, setSearch] = useState('');
  const [isPreviewOpen, setIsPreviewOpen] = useState(false);
  const [previewData, setPreviewData] = useState<any | null>(null);
  const [isLoadingPreview, setIsLoadingPreview] = useState(false);
  const [previewError, setPreviewError] = useState<string | null>(null);
  const hasLoadedRef = useRef(false);

  useEffect(() => {
    if (hasLoadedRef.current) {
      return;
    }
    if (assetsInitialized) {
      hasLoadedRef.current = true;
      return;
    }
    hasLoadedRef.current = true;
    void loadAssets();
  }, [assetsInitialized, loadAssets]);

  const filteredAssets = useMemo(() => {
    const term = search.trim().toLowerCase();
    if (!term) {
      return flagAssets;
    }
    return flagAssets.filter((asset) =>
      [asset.ioc_code, asset.display_name, asset.file_name]
        .filter(Boolean)
        .some((value) => (value ?? '').toLowerCase().includes(term)),
    );
  }, [flagAssets, search]);

  const handleRowSelect = useCallback((asset: FlagAnimationAsset) => {
    setForm({
      id: asset.id,
      ioc_code: asset.ioc_code,
      file_name: asset.file_name,
      file_path: asset.file_path,
      display_name: asset.display_name || '',
      duration_ms: asset.duration_ms ?? undefined,
      is_default: asset.is_default,
    });
  }, []);

  const handleReset = useCallback(() => {
    setForm(defaultForm);
    setMessage(null);
    setError(null);
  }, [setError]);

  const handlePickFile = useCallback(async () => {
    const path = await pickFilePath(['json', 'lottie']);
    if (!path) return;
    const fileName = path.split(/[\\/]/).pop() || '';
    setForm((prev) => ({
      ...prev,
      file_path: path,
      file_name: fileName,
    }));
  }, []);

  const handlePreview = useCallback(async () => {
    const path = form.file_path.trim();
    if (!path) {
      setError('Select an animation file before previewing.');
      return;
    }
    setIsPreviewOpen(true);
    setIsLoadingPreview(true);
    setPreviewError(null);
    setPreviewData(null);
    try {
      const raw = await readAnimationFile(path);
      const json = JSON.parse(raw);
      setPreviewData(json);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setPreviewError(message);
      setError(`Failed to load animation preview: ${message}`);
    } finally {
      setIsLoadingPreview(false);
    }
  }, [form.file_path, setError]);

  const handleSubmit = useCallback(async () => {
    if (!form.ioc_code.trim() || !form.file_path.trim()) {
      setError('IOC code and file path are required.');
      return;
    }
    const payload: FlagAnimationAsset = {
      id: form.id ?? undefined,
      ioc_code: form.ioc_code.trim().toUpperCase(),
      file_name: form.file_name || form.file_path.split(/[\\/]/).pop() || '',
      file_path: form.file_path,
      display_name: form.display_name || undefined,
      duration_ms: form.duration_ms,
      is_default: form.is_default,
      created_at: undefined,
      updated_at: undefined,
    };
    const id = await saveFlagAsset(payload);
    if (id) {
      setMessage('Animation saved successfully.');
      setForm(defaultForm);
    }
  }, [form, saveFlagAsset, setError]);

  const handleClosePreview = useCallback(() => {
    setIsPreviewOpen(false);
    setPreviewData(null);
    setIsLoadingPreview(false);
    setPreviewError(null);
  }, []);

  const handleDelete = useCallback(
    async (asset: FlagAnimationAsset) => {
      if (!asset.id) return;
      if (!window.confirm(`Delete animation for ${asset.ioc_code}?`)) {
        return;
      }
      await deleteFlagAsset(asset.id);
      setMessage('Animation deleted successfully.');
      setForm(defaultForm);
    },
    [deleteFlagAsset, setMessage],
  );

  const handleDeleteFromForm = useCallback(async () => {
    if (!form.id) return;
    await handleDelete({
      id: form.id || undefined,
      ioc_code: form.ioc_code,
      file_name: form.file_name,
      file_path: form.file_path,
      display_name: form.display_name,
      duration_ms: form.duration_ms,
      is_default: form.is_default,
      created_at: undefined,
      updated_at: undefined,
    } as FlagAnimationAsset);
  }, [form, handleDelete]);

  const handleSetDefault = useCallback(
    async (asset: FlagAnimationAsset) => {
      const updated: FlagAnimationAsset = {
        ...asset,
        is_default: true,
      };
      await saveFlagAsset(updated);
      setMessage(`${asset.ioc_code} default animation updated.`);
    },
    [saveFlagAsset],
  );

  return (
    <>
      <div className="space-y-6">
      {(error || message) && (
        <div className="space-y-2">
          {error && (
            <div className="rounded-md border border-red-500/40 bg-red-500/10 px-3 py-2 text-sm text-red-200">
              {error}
            </div>
          )}
          {message && (
            <div className="rounded-md border border-blue-500/40 bg-blue-500/10 px-3 py-2 text-sm text-blue-200">
              {message}
            </div>
          )}
        </div>
      )}

      <div className="theme-card space-y-4 p-6 shadow-lg">
        <div className="flex flex-wrap items-end gap-3">
          <div className="flex-1 min-w-[200px]">
            <Label htmlFor="flag-search">Search animations</Label>
            <Input
              id="flag-search"
              value={search}
              placeholder="Filter by IOC code or name..."
              onChange={(event) => setSearch(event.target.value)}
            />
          </div>
          <Button variant="secondary" size="sm" onClick={handleReset}>
            New animation
          </Button>
        </div>

        <div className="max-h-96 overflow-auto rounded-md border border-gray-700">
          <table className="min-w-full divide-y divide-gray-700 text-sm">
            <thead className="sticky top-0 z-10 bg-gray-900/95 text-xs uppercase tracking-wide text-gray-200 backdrop-blur shadow-lg shadow-blue-900/20 border-b border-blue-700/50">
              <tr>
                <th className="px-3 py-2 text-left">Nation</th>
                <th className="px-3 py-2 text-left">Display name</th>
                <th className="px-3 py-2 text-left">File</th>
                <th className="px-3 py-2 text-left">Duration (ms)</th>
                <th className="px-3 py-2 text-left">Default</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-800 bg-gray-950/40">
              {filteredAssets.map((asset) => {
                const isSelected =
                  (form.id && asset.id && form.id === asset.id) ||
                  (!form.id &&
                    !asset.id &&
                    form.ioc_code.trim().toUpperCase() === asset.ioc_code.toUpperCase());

                return (
                  <tr
                    key={asset.id ?? `${asset.ioc_code}-${asset.file_name}`}
                    onClick={() => handleRowSelect(asset)}
                    className={`cursor-pointer transition-colors ${
                      isSelected
                        ? 'bg-blue-900/25 border-l-4 border-blue-500'
                        : 'hover:bg-gray-800/60'
                    }`}
                  >
                    <td className="px-3 py-2 text-gray-200">
                      {asset.ioc_code ? (
                        <div className="flex items-center gap-2">
                          <FlagImage
                            countryCode={asset.ioc_code.toUpperCase()}
                            className="w-10 h-6 rounded-md shadow-md ring-1 ring-white/30"
                          />
                          <span>{asset.ioc_code.toUpperCase()}</span>
                        </div>
                      ) : (
                        <span className="text-xs text-gray-500">N/A</span>
                      )}
                    </td>
                    <td className="px-3 py-2 text-gray-300">{asset.display_name || 'N/A'}</td>
                    <td className="px-3 py-2 text-gray-400 truncate max-w-xs" title={asset.file_path}>
                      {asset.file_name}
                    </td>
                    <td className="px-3 py-2 text-gray-200">
                      {asset.duration_ms !== null && asset.duration_ms !== undefined
                        ? asset.duration_ms
                        : 'N/A'}
                    </td>
                    <td className="px-3 py-2 text-gray-200">
                    {asset.is_default ? (
                      <span className="rounded bg-green-600/20 px-2 py-1 text-xs text-green-300">
                        Default
                      </span>
                    ) : (
                      <Button
                        variant="secondary"
                        size="sm"
                        onClick={(event) => {
                          event.stopPropagation();
                          handleSetDefault(asset);
                        }}
                      >
                        Set default
                      </Button>
                    )}
                  </td>
                  </tr>
                );
              })}
              {filteredAssets.length === 0 && (
                <tr>
                  <td className="px-3 py-4 text-center text-gray-400" colSpan={5}>
                    No animations found.
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </div>

      <div className="theme-card space-y-4 p-6 shadow-lg">
        <h3 className="text-lg font-semibold text-gray-100">
          {form.id ? 'Edit animation' : 'Add new animation'}
        </h3>
        <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
          <div>
            <Label htmlFor="flag-ioc">IOC code</Label>
            <Input
              id="flag-ioc"
              value={form.ioc_code}
              onChange={(event) =>
                setForm((prev) => ({ ...prev, ioc_code: event.target.value.toUpperCase() }))
              }
              placeholder="e.g. USA"
            />
          </div>
          <div>
            <Label htmlFor="flag-display">Display name</Label>
            <Input
              id="flag-display"
              value={form.display_name}
              onChange={(event) => setForm((prev) => ({ ...prev, display_name: event.target.value }))}
              placeholder="Friendly name"
            />
          </div>
          <div className="md:col-span-2">
            <Label>Animation file (Lottie JSON)</Label>
            <div className="flex gap-2">
              <Input value={form.file_path} readOnly placeholder="Select animation file" />
              <Button variant="secondary" size="sm" onClick={handlePickFile}>
                Open
              </Button>
              {form.file_path && (
                <Button
                  variant="secondary"
                  size="sm"
                  onClick={() => setForm((prev) => ({ ...prev, file_path: '', file_name: '' }))}
                >
                  Clear
                </Button>
              )}
            </div>
          </div>
          <div>
            <Label htmlFor="flag-duration">Duration (ms)</Label>
            <Input
              id="flag-duration"
              type="number"
              min={0}
              value={form.duration_ms ?? ''}
              onChange={(event) =>
                setForm((prev) => ({
                  ...prev,
                  duration_ms: event.target.value ? Number(event.target.value) : undefined,
                }))
              }
            />
          </div>
          <div className="flex items-end">
            <Toggle
              label="Default for IOC"
              checked={form.is_default}
              onChange={(event) =>
                setForm((prev) => ({ ...prev, is_default: event.currentTarget.checked }))
              }
            />
          </div>
        </div>
        <div className="flex flex-wrap items-center justify-between gap-2">
          <div className="flex gap-2">
            <Button variant="secondary" onClick={handleReset} disabled={saving}>
              Add
            </Button>
            <Button
              variant="secondary"
              onClick={handlePreview}
              disabled={saving || !form.file_path || isLoadingPreview}
            >
              {isLoadingPreview ? 'Previewing...' : 'Preview'}
            </Button>
            <Button variant="primary" onClick={handleSubmit} disabled={saving}>
              {saving ? 'Saving...' : 'Save'}
            </Button>
          </div>
          {form.id && (
            <Button variant="danger" onClick={handleDeleteFromForm} disabled={saving}>
              Delete
            </Button>
          )}
        </div>
      </div>
      </div>

      {isPreviewOpen && (
        <div className="fixed inset-0 bg-black/60 flex items-center justify-center z-50">
          <div className="theme-card shadow-xl w-full max-w-3xl max-h-[80vh] flex flex-col overflow-hidden">
            <div className="flex items-start justify-between gap-4 px-6 pt-6">
              <div>
                <h3 className="text-xl font-semibold text-gray-100">
                  {form.display_name || form.ioc_code || 'Animation Preview'}
                </h3>
                {form.ioc_code && (
                  <div className="mt-2 flex items-center gap-2 text-sm text-gray-400">
                    <FlagImage
                      countryCode={form.ioc_code.toUpperCase()}
                      className="w-10 h-6 rounded-md shadow-md ring-1 ring-white/30"
                    />
                    <span>{form.ioc_code.toUpperCase()}</span>
                  </div>
                )}
              </div>
              <Button
                onClick={handleClosePreview}
                className="bg-gray-600 hover:bg-gray-700 text-white"
              >
                Close
              </Button>
            </div>
            <div className="flex-1 overflow-y-auto px-6 pb-6">
              {previewError && (
                <div className="mb-4 rounded border border-red-500/40 bg-red-500/10 px-3 py-2 text-sm text-red-200">
                  {previewError}
                </div>
              )}
              {isLoadingPreview && (
                <div className="py-12 text-center text-sm text-gray-300">Loading animation...</div>
              )}
              {!isLoadingPreview && previewData && (
                <div className="flex items-center justify-center rounded-lg bg-gray-900/40 p-6">
                  <Lottie
                    animationData={previewData}
                    loop
                    style={{ width: '100%', maxWidth: 420, height: 'auto' }}
                  />
                </div>
              )}
              {!isLoadingPreview && !previewData && !previewError && (
                <div className="py-12 text-center text-sm text-gray-400">
                  Select an animation to preview.
                </div>
              )}
            </div>
          </div>
        </div>
      )}
    </>
  );
};

export default MedalCeremonyAnimationManager;
