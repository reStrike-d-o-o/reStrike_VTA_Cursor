import React, { useCallback, useEffect, useMemo, useState } from 'react';
import Button from '../atoms/Button';
import Input from '../atoms/Input';
import Label from '../atoms/Label';
import Toggle from '../atoms/Toggle';
import { useMedalCeremonyStore } from '../../stores/medalCeremonyStore';
import { FlagAnimationAsset } from '../../types';
import { pickFilePath } from '../../utils/filePicker';

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

const MedalCeremonyAnimationManager: React.FC = () => {
  const {
    flagAssets,
    loadAssets,
    saveFlagAsset,
    deleteFlagAsset,
    saving,
    loading,
    error,
    setError,
  } = useMedalCeremonyStore((state) => ({
    flagAssets: state.flagAssets,
    loadAssets: state.loadAssets,
    saveFlagAsset: state.saveFlagAsset,
    deleteFlagAsset: state.deleteFlagAsset,
    saving: state.saving,
    loading: state.loading,
    error: state.error,
    setError: state.setError,
  }));

  const [form, setForm] = useState<FlagAssetForm>(defaultForm);
  const [message, setMessage] = useState<string | null>(null);
  const [search, setSearch] = useState('');

  useEffect(() => {
    void loadAssets();
  }, [loadAssets]);

  const filteredAssets = useMemo(() => {
    const term = search.trim().toLowerCase();
    if (!term) return flagAssets;
    return flagAssets.filter((asset) =>
      [asset.ioc_code, asset.display_name, asset.file_name]
        .filter(Boolean)
        .some((value) => value!.toLowerCase().includes(term)),
    );
  }, [flagAssets, search]);

  const handleEdit = useCallback((asset: FlagAnimationAsset) => {
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

  const handleDelete = useCallback(
    async (asset: FlagAnimationAsset) => {
      if (!asset.id) return;
      if (!window.confirm(`Delete animation for ${asset.ioc_code}?`)) {
        return;
      }
      await deleteFlagAsset(asset.id);
    },
    [deleteFlagAsset],
  );

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
              placeholder="Filter by IOC code or name…"
              onChange={(event) => setSearch(event.target.value)}
            />
          </div>
          <Button variant="secondary" size="sm" onClick={handleReset}>
            New animation
          </Button>
        </div>

        <div className="overflow-hidden rounded-md border border-gray-700">
          <table className="min-w-full divide-y divide-gray-700 text-sm">
            <thead className="bg-gray-900/80 text-xs uppercase tracking-wide text-gray-400">
              <tr>
                <th className="px-3 py-2 text-left">IOC</th>
                <th className="px-3 py-2 text-left">Display name</th>
                <th className="px-3 py-2 text-left">File</th>
                <th className="px-3 py-2 text-left">Duration (ms)</th>
                <th className="px-3 py-2 text-left">Default</th>
                <th className="px-3 py-2 text-right">Actions</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-800 bg-gray-950/40">
              {filteredAssets.map((asset) => (
                <tr key={asset.id ?? `${asset.ioc_code}-${asset.file_name}`}>
                  <td className="px-3 py-2 text-gray-200">{asset.ioc_code}</td>
                  <td className="px-3 py-2 text-gray-300">{asset.display_name || '—'}</td>
                  <td className="px-3 py-2 text-gray-400 truncate max-w-xs" title={asset.file_path}>
                    {asset.file_name}
                  </td>
                  <td className="px-3 py-2 text-gray-200">{asset.duration_ms ?? '—'}</td>
                  <td className="px-3 py-2 text-gray-200">
                    {asset.is_default ? (
                      <span className="rounded bg-green-600/20 px-2 py-1 text-xs text-green-300">
                        Default
                      </span>
                    ) : (
                      <Button
                        variant="secondary"
                        size="sm"
                        onClick={() => handleSetDefault(asset)}
                      >
                        Set default
                      </Button>
                    )}
                  </td>
                  <td className="px-3 py-2 text-right">
                    <div className="flex justify-end gap-2">
                      <Button variant="secondary" size="sm" onClick={() => handleEdit(asset)}>
                        Edit
                      </Button>
                      <Button variant="danger" size="sm" onClick={() => handleDelete(asset)}>
                        Delete
                      </Button>
                    </div>
                  </td>
                </tr>
              ))}
              {filteredAssets.length === 0 && (
                <tr>
                  <td className="px-3 py-4 text-center text-gray-400" colSpan={6}>
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
        <div className="flex flex-wrap gap-2">
          <Button variant="primary" onClick={handleSubmit} disabled={saving}>
            {saving ? 'Saving…' : form.id ? 'Update animation' : 'Add animation'}
          </Button>
          <Button variant="secondary" onClick={handleReset} disabled={saving}>
            Reset
          </Button>
        </div>
      </div>
    </div>
  );
};

export default MedalCeremonyAnimationManager;
