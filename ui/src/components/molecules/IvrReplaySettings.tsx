import React, { useEffect, useState } from 'react';
import Label from '../atoms/Label';
import Input from '../atoms/Input';
import Toggle from '../atoms/Toggle';
import Button from '../atoms/Button';
import { obsObwsCommands } from '../../utils/tauriCommandsObws';
import { useI18n } from '../../i18n/index';

const IvrReplaySettings: React.FC = () => {
  const { t } = useI18n();
  const [mpvPath, setMpvPath] = useState<string>('');
  const [secondsFromEnd, setSecondsFromEnd] = useState<number>(10);
  const [maxWaitMs, setMaxWaitMs] = useState<number>(500);
  const [autoOnChallenge, setAutoOnChallenge] = useState<boolean>(false);
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState<string>('');

  const load = async () => {
    try {
      setLoading(true);
      const res = await obsObwsCommands.ivrGetReplaySettings();
      if (res.success && res.data) {
        const d = res.data;
        setMpvPath(d.mpv_path || '');
        setSecondsFromEnd(typeof d.seconds_from_end === 'number' ? d.seconds_from_end : 10);
        setMaxWaitMs(typeof d.max_wait_ms === 'number' ? d.max_wait_ms : 500);
        setAutoOnChallenge(!!d.auto_on_challenge);
      }
    } catch (e) {
      console.error('Failed to load IVR settings', e);
    } finally {
      setLoading(false);
    }
  };

  const save = async () => {
    try {
      setSaving(true);
      if (mpvPath) {
        const valid = await obsObwsCommands.ivrValidateMpvPath(mpvPath);
        if (!valid.success) {
          setMessage(t('ivr.settings.invalid_mpv', 'Invalid mpv path.'));
          setSaving(false);
          return;
        }
      }
      const res = await obsObwsCommands.ivrSaveReplaySettings({
        mpvPath: mpvPath || undefined,
        secondsFromEnd: Math.max(0, Math.min(20, Number(secondsFromEnd) || 10)),
        maxWaitMs: Math.max(50, Math.min(500, Number(maxWaitMs) || 500)),
        autoOnChallenge,
      });
      setMessage(res.success ? t('ivr.settings.saved', 'Settings saved.') : t('ivr.settings.save_failed', 'Save failed: {err}', { err: String(res.error || '') }));
    } catch (e) {
      setMessage(t('ivr.settings.save_failed', 'Save failed: {err}', { err: String(e) }));
    } finally {
      setSaving(false);
    }
  };

  useEffect(() => { load(); }, []);

  return (
    <div className="theme-card p-6 shadow-lg">
      <h3 className="text-lg font-semibold mb-4 text-gray-100">{t('ivr.settings.title', 'IVR Replay Settings')}</h3>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div>
          <Label htmlFor="mpv-path" className="block text-sm font-medium text-gray-300 mb-2">{t('ivr.settings.mpv_path', 'Path to mpv.exe')}</Label>
          <div className="flex gap-2">
            <Input id="mpv-path" type="text" value={mpvPath} onChange={(e) => setMpvPath(e.target.value)} placeholder="C:/Program Files/mpv/mpv.exe" className="flex-1" />
            <Button
              onClick={async () => {
                try {
                  // Prefer Tauri dialog plugin when available
                  if (typeof window !== 'undefined' && (window as any).__TAURI__) {
                    const { open } = await import('@tauri-apps/plugin-dialog');
                    const selected = await open({ multiple: false, filters: [{ name: 'Executable', extensions: ['exe'] }] });
                    if (selected && typeof selected === 'string') setMpvPath(selected);
                    return;
                  }
                  // Fallback: hidden file input in web
                  const input = document.createElement('input');
                  input.type = 'file';
                  input.accept = '.exe';
                  input.onchange = () => {
                    const file = (input.files && input.files[0]) || null;
                    if (file) setMpvPath(file.name);
                  };
                  input.click();
                } catch (e) {
                  setMessage(t('ivr.settings.err_dialog', 'Failed to open file dialog'));
                }
              }}
              className="bg-gray-600 hover:bg-gray-700"
            >
              {t('common.browse', 'Browse')}
            </Button>
          </div>
        </div>

        <div>
          <Label htmlFor="seconds-end" className="block text-sm font-medium text-gray-300 mb-2">{t('ivr.settings.seconds_from_end', 'Seconds from end (0–20)')}</Label>
          <Input id="seconds-end" type="number" value={secondsFromEnd} onChange={(e) => setSecondsFromEnd(parseInt(e.target.value) || 0)} />
        </div>

        <div>
          <Label htmlFor="max-wait" className="block text-sm font-medium text-gray-300 mb-2">{t('ivr.settings.max_wait', 'Max wait (ms, 50–500)')}</Label>
          <Input id="max-wait" type="number" value={maxWaitMs} onChange={(e) => setMaxWaitMs(parseInt(e.target.value) || 500)} />
        </div>

        <div className="flex items-end">
          <Toggle label={t('ivr.settings.auto_on_challenge', 'Auto on PSS Challenge')} checked={autoOnChallenge} onChange={(e) => setAutoOnChallenge(e.target.checked)} />
        </div>
      </div>

      <div className="flex gap-3 mt-4">
        <Button onClick={load} disabled={loading} className="bg-gray-600 hover:bg-gray-700">{loading ? t('common.loading', 'Loading...') : t('common.load', 'Load')}</Button>
        <Button onClick={save} disabled={saving} className="bg-blue-600 hover:bg-blue-700">{saving ? t('common.saving', 'Saving...') : t('common.save', 'Save')}</Button>
      </div>

      {message && <div className="text-sm text-gray-300 mt-2">{message}</div>}
    </div>
  );
};

export default IvrReplaySettings;


