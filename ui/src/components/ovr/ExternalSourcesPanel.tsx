import React from 'react';
import Button from '../atoms/Button';
import Toggle from '../atoms/Toggle';
import { useI18n } from '../../i18n';
import { invoke } from '@tauri-apps/api/core';

interface Provider {
  id?: number;
  name: string;
  base_url?: string;
  enabled: boolean;
  rate_limit_ms: number;
  last_refreshed_at?: string;
  last_status?: string;
  last_error?: string;
}

const ExternalSourcesPanel: React.FC = () => {
  const { t } = useI18n();
  const [providers, setProviders] = React.useState<Provider[]>([]);
  const [loading, setLoading] = React.useState(false);
  const [error, setError] = React.useState<string | null>(null);
  const [editing, setEditing] = React.useState<Record<number, Provider>>({});

  const load = async () => {
    try {
      setLoading(true);
      setError(null);
      const res: any = await invoke('ovr_get_providers');
      if (res?.success) setProviders(res.providers || []);
      else setError(res?.error || 'Failed to load providers');
    } catch (e: any) {
      setError(typeof e === 'string' ? e : (e?.message || 'Failed to load providers'));
    } finally {
      setLoading(false);
    }
  };

  React.useEffect(() => { load(); }, []);

  const toggleEnabled = async (p: Provider, value: boolean) => {
    try {
      await invoke('ovr_upsert_provider', { payload: { id: p.id ?? null, name: p.name, base_url: p.base_url ?? null, enabled: value, rate_limit_ms: p.rate_limit_ms } });
      await load();
    } catch (e) { /* ignore */ }
  };

  const refreshProvider = async (id?: number) => {
    if (id == null) return;
    try {
      setLoading(true);
      await invoke('ovr_refresh_provider', { providerId: id });
      await load();
    } catch (_) {
    } finally { setLoading(false); }
  };

  const refreshAll = async () => {
    try {
      setLoading(true);
      await invoke('ovr_refresh_all');
      await load();
    } catch (_) {
    } finally { setLoading(false); }
  };

  const remove = async (id?: number) => {
    if (id == null) return;
    try { await invoke('ovr_remove_provider', { id }); await load(); } catch (_) {}
  };

  const startEdit = (p: Provider) => {
    if (p.id == null) return;
    setEditing(prev => ({ ...prev, [p.id as number]: { ...p } }));
  };

  const cancelEdit = (id?: number) => {
    if (id == null) return;
    setEditing(prev => { const n = { ...prev }; delete n[id]; return n; });
  };

  const saveEdit = async (id?: number) => {
    if (id == null) return;
    const p = editing[id];
    if (!p) return;
    try {
      await invoke('ovr_upsert_provider', { payload: { id, name: p.name, base_url: p.base_url ?? null, enabled: p.enabled, rate_limit_ms: p.rate_limit_ms } });
      cancelEdit(id); await load();
    } catch (_) {}
  };

  return (
    <div className="space-y-6">
      <div className="theme-card p-6 shadow-lg">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold text-gray-100">{t('ovr.external.title', 'External OVR Sources')}</h3>
          <div className="flex items-center gap-2">
            <Button variant="secondary" onClick={load} disabled={loading}>{t('common.refresh', 'Refresh')}</Button>
            <Button variant="primary" onClick={refreshAll} disabled={loading}>{t('common.update_all','Update all')}</Button>
          </div>
        </div>
        {error && <div className="text-red-400 text-sm mb-3">{error}</div>}
        <div className="grid grid-cols-1 gap-3">
          {providers.map((p) => (
            <div key={p.name} className="flex items-center justify-between bg-gray-800/50 rounded px-4 py-3 border border-gray-700">
              <div className="flex flex-col">
                <span className="text-gray-100 font-medium">{p.name}</span>
                {p.id != null && editing[p.id] ? (
                  <div className="flex items-center gap-2 mt-1">
                    <input aria-label={t('ovr.external.base_url','Base URL')} className="text-xs bg-gray-900 border border-gray-700 rounded px-2 py-1 text-gray-200 w-80" value={editing[p.id].base_url || ''} onChange={(e)=>setEditing(prev=>({ ...prev, [p.id as number]: { ...prev[p.id as number], base_url: e.target.value } }))} placeholder="https://..." />
                    <input aria-label={t('ovr.external.rate_limit','Rate limit (ms)')} className="text-xs bg-gray-900 border border-gray-700 rounded px-2 py-1 text-gray-200 w-28" value={editing[p.id].rate_limit_ms} onChange={(e)=>{
                      const v = parseInt(e.target.value)||0; setEditing(prev=>({ ...prev, [p.id as number]: { ...prev[p.id as number], rate_limit_ms: v } }));
                    }} />
                  </div>
                ) : (
                  <>
                    <span className="text-xs text-gray-400">{p.base_url || ''}</span>
                    <span className="text-xs text-gray-500">{t('common.rate_limit','Rate limit')}: {p.rate_limit_ms}ms</span>
                  </>
                )}
                <div className="text-xs text-gray-500 mt-1">
                  {p.last_status && <span className="mr-3">{t('ovr.external.last_status', 'Status')}: {p.last_status}</span>}
                  {p.last_refreshed_at && <span className="mr-3">{t('ovr.external.last_refresh', 'Last refresh')}: {p.last_refreshed_at}</span>}
                  {p.last_error && <span className="text-red-400">{t('common.error', 'Error')}: {p.last_error}</span>}
                </div>
              </div>
              <div className="flex items-center gap-2">
                <Toggle
                  checked={p.enabled}
                  onChange={(e)=>toggleEnabled(p, e.currentTarget.checked)}
                  aria-label={t('common.enabled','Enabled')}
                />
                {p.id != null && editing[p.id] ? (
                  <>
                    <Button variant="secondary" onClick={() => saveEdit(p.id)}>{t('common.save','Save')}</Button>
                    <Button variant="secondary" onClick={() => cancelEdit(p.id)}>{t('common.cancel','Cancel')}</Button>
                  </>
                ) : (
                  <Button variant="secondary" onClick={() => startEdit(p)}>{t('common.edit','Edit')}</Button>
                )}
                <Button variant="secondary" onClick={() => refreshProvider(p.id)}>{t('common.update','Update')}</Button>
                <Button variant="secondary" onClick={() => remove(p.id)}>{t('common.remove','Remove')}</Button>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};

export default ExternalSourcesPanel;

