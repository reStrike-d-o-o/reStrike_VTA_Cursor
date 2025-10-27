import React, { useCallback, useEffect, useState } from 'react';
import Button from '../atoms/Button';
import Input from '../atoms/Input';
import { useAppStore } from '../../stores';
import { useSettingsStore } from '../../stores/settingsStore';
import { windowCommands, licenseCommands, openApiCommands } from '../../utils/tauriCommands';
import { useEnvironment } from '../../hooks/useEnvironment';
import { logger, setLogLevel, LogLevel, applyConsolePatch } from '../../utils/logger';
import { useI18n } from '../../i18n/index';
import LanguageSelect from '../atoms/LanguageSelect';
import { SchemaFormat, SchemaValidationOutcome, OpenApiEndpoints } from '../../types';

const AppSettingsSection: React.FC = () => {
  const { locale, setLocale, t } = useI18n();
  const { tauriAvailable } = useEnvironment();
  const windowSettings = useAppStore((state) => state.windowSettings);
  const updateWindowSettings = useAppStore((state) => state.updateWindowSettings);
  const resetWindowSettings = useAppStore((state) => state.resetWindowSettings);
  const saveWindowSettings = useAppStore((state) => state.saveWindowSettings);

  const [isLoading, setIsLoading] = useState(false);
  const [message, setMessage] = useState('');
  const [logLevel, setLevel] = useState<LogLevel>('info');
  const [licenseKey, setLicenseKey] = useState('');
  const [licenseStatus, setLicenseStatus] = useState<any>(null);
  const [machineUid, setMachineUid] = useState<string>('');
  const [machineHash, setMachineHash] = useState<string>('');
  const theme = useSettingsStore((s) => s.theme);
  const setTheme = useSettingsStore((s) => s.setTheme);
  const sharp = useSettingsStore((s)=> (s as any).sharp);
  const setSharp = useSettingsStore((s)=> (s as any).setSharp);

  const [activeSubTab, setActiveSubTab] = useState<'visual' | 'system'>('visual');
  const [schemaFormat, setSchemaFormat] = useState<SchemaFormat>('yaml');
  const [schemaText, setSchemaText] = useState('');
  const [schemaValidation, setSchemaValidation] = useState<SchemaValidationOutcome | null>(null);
  const [schemaUpdatedAt, setSchemaUpdatedAt] = useState<string>('');
  const [schemaEndpoints, setSchemaEndpoints] = useState<OpenApiEndpoints | null>(null);
  const [schemaLoading, setSchemaLoading] = useState(false);
  const [schemaMessage, setSchemaMessage] = useState<string | null>(null);
  const [schemaError, setSchemaError] = useState<string | null>(null);
  const [schemaDirty, setSchemaDirty] = useState(false);

  const handleApplySettings = async () => {
    if (!tauriAvailable) {
      setMessage('Tauri not available - settings saved but not applied');
      return;
    }

    setIsLoading(true);
    setMessage('');

    try {
      // Save settings first
      await saveWindowSettings();
      
      // Apply compact size
      await windowCommands.setCustomSize(windowSettings.compactWidth, windowSettings.compactHeight);
      setMessage('Window settings applied and saved successfully!');
    } catch (error) {
      setMessage(`Error applying settings: ${error}`);
    } finally {
      setIsLoading(false);
    }
  };

  const handleReset = () => {
    resetWindowSettings();
    setMessage('Settings reset to defaults');
  };

  const applyLogLevel = (lvl: LogLevel) => {
    setLevel(lvl);
    setLogLevel(lvl);
    try { localStorage.setItem('logLevel', lvl); } catch {}
    applyConsolePatch();
    logger.info('Log level set to', lvl);
  };

  const loadOpenApiState = useCallback(
    async (format?: SchemaFormat) => {
      if (!tauriAvailable) {
        return;
      }
      setSchemaLoading(true);
      setSchemaError(null);

      try {
        const response = await openApiCommands.getState(format);
        setSchemaFormat(response.format);
        setSchemaText(response.schema);
        setSchemaValidation(response.validation);
        setSchemaUpdatedAt(response.updated_at);
        setSchemaEndpoints(response.endpoints);
        setSchemaDirty(false);
        setSchemaMessage(null);
      } catch (error) {
        setSchemaError(
          t(
            'settings.openapi.load_error',
            'Failed to load OpenAPI schema: {{error}}',
            { error: String(error) }
          )
        );
      } finally {
        setSchemaLoading(false);
      }
    },
    [tauriAvailable, t]
  );

  useEffect(() => {
    if (tauriAvailable) {
      loadOpenApiState();
    }
  }, [tauriAvailable, loadOpenApiState]);

  const handleSchemaChange = (value: string) => {
    setSchemaText(value);
    setSchemaDirty(true);
    setSchemaMessage(null);
    setSchemaError(null);
    setSchemaValidation(null);
  };

  const handleFormatChange = async (nextFormat: SchemaFormat) => {
    if (nextFormat === schemaFormat) {
      return;
    }

    if (schemaDirty && typeof window !== 'undefined') {
      const confirmChange = window.confirm(
        t(
          'settings.openapi.discard_prompt',
          'Switching format will discard unsaved changes. Continue?'
        )
      );
      if (!confirmChange) {
        return;
      }
    }

    await loadOpenApiState(nextFormat);
  };

  const handleValidateSchema = async () => {
    if (!tauriAvailable) {
      return;
    }
    setSchemaLoading(true);
    setSchemaError(null);

    try {
      const result = await openApiCommands.validateSchema(schemaText, schemaFormat);
      setSchemaValidation(result.validation);
      setSchemaMessage(
        result.validation.valid
          ? t('settings.openapi.validation_success', 'Schema validation passed.')
          : t('settings.openapi.validation_failed', 'Schema validation reported issues.')
      );
    } catch (error) {
      setSchemaError(
        t(
          'settings.openapi.validation_error',
          'Validation failed: {{error}}',
          { error: String(error) }
        )
      );
    } finally {
      setSchemaLoading(false);
    }
  };

  const handleSaveSchema = async () => {
    if (!tauriAvailable) {
      return;
    }
    setSchemaLoading(true);
    setSchemaError(null);

    try {
      const result = await openApiCommands.saveSchema(schemaText, schemaFormat);
      setSchemaValidation(result.validation);
      if (result.validation.valid) {
        await loadOpenApiState(schemaFormat);
        setSchemaMessage(
          t('settings.openapi.save_success', 'OpenAPI schema saved and applied.')
        );
        setSchemaDirty(false);
      } else {
        setSchemaMessage(
          t('settings.openapi.save_failed', 'Schema not saved. Fix validation errors.')
        );
      }
    } catch (error) {
      setSchemaError(
        t(
          'settings.openapi.save_error',
          'Failed to save schema: {{error}}',
          { error: String(error) }
        )
      );
    } finally {
      setSchemaLoading(false);
    }
  };

  const handleUploadSchema = async () => {
    if (!tauriAvailable) {
      return;
    }
    try {
      const { open } = await import('@tauri-apps/plugin-dialog');
      const selected = await open({
        multiple: false,
        filters: [{ name: 'OpenAPI Schema', extensions: ['yaml', 'yml', 'json'] }],
      });
      const pathSelection = Array.isArray(selected) ? selected[0] : selected;
      if (!pathSelection) {
        return;
      }
      setSchemaLoading(true);
      const response = await openApiCommands.uploadSchema(String(pathSelection));
      setSchemaFormat(response.format);
      setSchemaText(response.schema);
      setSchemaValidation(response.validation);
      setSchemaDirty(true);
      setSchemaMessage(
        t(
          'settings.openapi.upload_success',
          'Schema loaded from file. Remember to validate and save.'
        )
      );
      setSchemaError(null);
    } catch (error) {
      setSchemaError(
        t(
          'settings.openapi.upload_error',
          'Failed to load schema file: {{error}}',
          { error: String(error) }
        )
      );
    } finally {
      setSchemaLoading(false);
    }
  };

  const handleExportSchema = async () => {
    if (!tauriAvailable) {
      return;
    }
    try {
      const { save } = await import('@tauri-apps/plugin-dialog');
      const suggested = `restrike-openapi.${schemaFormat}`;
      const target = await save({
        defaultPath: suggested,
        filters: [
          {
            name: 'OpenAPI Schema',
            extensions: schemaFormat === 'yaml' ? ['yaml', 'yml'] : ['json'],
          },
        ],
      });
      if (!target) {
        return;
      }
      await openApiCommands.exportSchema(String(target), schemaFormat);
      setSchemaMessage(
        t('settings.openapi.export_success', 'Schema exported to {{path}}', {
          path: target,
        })
      );
      setSchemaError(null);
    } catch (error) {
      setSchemaError(
        t(
          'settings.openapi.export_error',
          'Failed to export schema: {{error}}',
          { error: String(error) }
        )
      );
    }
  };

  useEffect(() => {
    (async () => {
      try {
        const res = await licenseCommands.getStatus();
        if (res.success) setLicenseStatus(res.data);
        const mi = await licenseCommands.getMachineIdentity();
        if (mi.success && mi.data) {
          setMachineUid(mi.data.uid);
          setMachineHash(mi.data.machine_hash);
        }
      } catch {}
    })();
  }, []);

  const handleActivate = async () => {
    try {
      const res = await licenseCommands.activate(licenseKey.trim());
      if (res.success) {
        setLicenseStatus(res.data);
        setMessage('License activated.');
      } else {
        setMessage(res.error || 'Activation failed');
      }
    } catch (e) {
      setMessage(String(e));
    }
  };

  const handleValidate = async () => {
    try {
      const res = await licenseCommands.validate();
      if (res.success) {
        setLicenseStatus(res.data);
        setMessage('License validated.');
      } else {
        setMessage(res.error || 'Validation failed');
      }
    } catch (e) {
      setMessage(String(e));
    }
  };

  const formattedUpdatedAt = schemaUpdatedAt
    ? new Date(schemaUpdatedAt).toLocaleString()
    : t('settings.openapi.never', 'Never saved');

  return (
    <div className="space-y-6">
      <div className="flex flex-wrap items-center gap-2 border-b border-gray-700 pb-4">
        <Button
          size="sm"
          variant={activeSubTab === 'visual' ? 'primary' : 'secondary'}
          onClick={() => setActiveSubTab('visual')}
        >
          {t('settings.openapi.tab_visual', 'Visual & Localization')}
        </Button>
        <Button
          size="sm"
          variant={activeSubTab === 'system' ? 'primary' : 'secondary'}
          onClick={() => setActiveSubTab('system')}
        >
          {t('settings.openapi.tab_system', 'System')}
        </Button>
      </div>

      {activeSubTab === 'visual' ? (
        <div className="space-y-6">
      {/* Language */}
      <div>
        <h3 className="text-lg font-semibold text-white mb-2">{t('settings.language', 'Language')}</h3>
        <div className="flex items-center gap-2">
          <label className="text-sm text-gray-300" htmlFor="app-language">{t('settings.select_language', 'Select language')}</label>
          <LanguageSelect
            className="ml-2"
            value={locale}
            onChange={(code) => { console.log('[AppSettings] setLocale requested:', code); setLocale(code); }}
          />
        </div>
      </div>

      {/* Licensing */}
      <div className="space-y-3">
        <h3 className="text-lg font-semibold text-white">{t('settings.license.title', 'License')}</h3>
        <div className="text-sm text-gray-300">
          <div>{t('settings.license.status', 'Status')}: <span className="font-semibold">{licenseStatus?.state ?? 'Unknown'}</span></div>
          {licenseStatus?.plan && (
            <div>{t('settings.license.plan', 'Plan')}: <span className="font-semibold">{licenseStatus.plan}</span></div>
          )}
          {typeof licenseStatus?.days_remaining === 'number' && (
            <div>{t('settings.license.days_remaining', 'Days remaining')}: <span className="font-semibold">{licenseStatus.days_remaining}</span></div>
          )}
          {licenseStatus?.reason && (
            <div className="text-red-300">{t('settings.license.reason', 'Reason')}: {licenseStatus.reason}</div>
          )}
        </div>
        {/* Machine Identity */}
        <div className="bg-gray-700/50 border border-gray-600 rounded p-3 space-y-2">
          <div className="flex items-center justify-between gap-2">
            <span className="text-sm text-gray-300">{t('settings.license.machine_uid', 'Machine UID')}</span>
            <span className="text-sm font-mono text-gray-100 truncate max-w-[60%]" title={machineUid}>{machineUid || '...'}</span>
            <Button size="sm" variant="secondary" onClick={() => { navigator.clipboard?.writeText(machineUid); }}>{t('settings.copy', 'Copy')}</Button>
          </div>
          <div className="flex items-center justify-between gap-2">
            <span className="text-sm text-gray-300">{t('settings.license.machine_hash', 'Machine Hash')}</span>
            <span className="text-sm font-mono text-gray-100 truncate max-w-[60%]" title={machineHash}>{machineHash || '...'}</span>
            <Button size="sm" variant="secondary" onClick={() => { navigator.clipboard?.writeText(machineHash); }}>{t('settings.copy', 'Copy')}</Button>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <Input
            placeholder={t('settings.license.enter_key', 'Enter license key')}
            value={licenseKey}
            onChange={(e) => setLicenseKey(e.target.value)}
            className="flex-1"
          />
          <Button variant="primary" onClick={handleActivate}>{t('settings.license.activate', 'Activate')}</Button>
          <Button variant="secondary" onClick={handleValidate}>{t('settings.license.validate', 'Validate')}</Button>
        </div>
      </div>
      <div>
        <h3 className="text-lg font-semibold text-white mb-4">{t('settings.window.title', 'Window Settings')}</h3>
        <p className="text-gray-300 text-sm mb-4">{t('settings.window.help', 'Configure the window dimensions for compact and fullscreen modes.')}</p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {/* Compact Mode Settings */}
        <div className="space-y-4">
          <h4 className="text-md font-medium text-white">{t('settings.window.compact', 'Compact Mode (Default)')}</h4>
          <div className="space-y-3">
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-1">{t('settings.window.width', 'Width (px)')}</label>
              <Input
                type="number"
                value={windowSettings.compactWidth}
                onChange={(e) => updateWindowSettings({ compactWidth: Number(e.target.value) })}
                min="200"
                max="800"
                className="w-full"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-1">{t('settings.window.height', 'Height (px)')}</label>
              <Input
                type="number"
                value={windowSettings.compactHeight}
                onChange={(e) => updateWindowSettings({ compactHeight: Number(e.target.value) })}
                min="400"
                max="2000"
                className="w-full"
              />
            </div>
          </div>
        </div>

        {/* Fullscreen Mode Settings */}
        <div className="space-y-4">
          <h4 className="text-md font-medium text-white">{t('settings.window.full', 'Fullscreen Mode (Advanced)')}</h4>
          <div className="space-y-3">
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-1">{t('settings.window.width', 'Width (px)')}</label>
              <Input
                type="number"
                value={windowSettings.fullscreenWidth}
                onChange={(e) => updateWindowSettings({ fullscreenWidth: Number(e.target.value) })}
                min="800"
                max="4000"
                className="w-full"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-1">{t('settings.window.height', 'Height (px)')}</label>
              <Input
                type="number"
                value={windowSettings.fullscreenHeight}
                onChange={(e) => updateWindowSettings({ fullscreenHeight: Number(e.target.value) })}
                min="600"
                max="3000"
                className="w-full"
              />
            </div>
          </div>
        </div>
      </div>

      {/* Quick Presets */}
      <div className="space-y-3">
        <h4 className="text-md font-medium text-white">{t('settings.quick.title', 'Quick Presets')}</h4>
        <div className="flex flex-wrap gap-2">
          <Button
            variant="secondary"
            size="sm"
            onClick={() => updateWindowSettings({ compactWidth: 350, compactHeight: 1200 })}
            className="text-xs"
          >
            1920x1200 (Compact)
          </Button>
          <Button
            variant="secondary"
            size="sm"
            onClick={() => updateWindowSettings({ compactWidth: 350, compactHeight: 1080 })}
            className="text-xs"
          >
            1920x1080 (Compact)
          </Button>
          <Button
            variant="secondary"
            size="sm"
            onClick={() => updateWindowSettings({ fullscreenWidth: 1920, fullscreenHeight: 1200 })}
            className="text-xs"
          >
            1920x1200 (Full)
          </Button>
          <Button
            variant="secondary"
            size="sm"
            onClick={() => updateWindowSettings({ fullscreenWidth: 1920, fullscreenHeight: 1080 })}
            className="text-xs"
          >
            1920x1080 (Full)
          </Button>
        </div>
      </div>

      {/* Theme & Log verbosity */}
      <div className="space-y-3">
        <h4 className="text-md font-medium text-white">{t('settings.appearance.title', 'Appearance')}</h4>
        <div className="flex flex-wrap items-center gap-3">
          <span className="text-sm text-gray-300">{t('settings.appearance.theme', 'Theme')}</span>
          <Button size="sm" variant={theme==='dark' ? 'primary' : 'secondary'} onClick={() => setTheme('dark')}>{t('settings.appearance.dark', 'Dark')}</Button>
          <Button size="sm" variant={theme==='light' ? 'primary' : 'secondary'} onClick={() => setTheme('light')}>{t('settings.appearance.light', 'Light')}</Button>
          <span className="text-sm text-gray-300 ml-4">{t('settings.appearance.corners', 'Corners')}</span>
          <Button size="sm" variant={sharp ? 'primary' : 'secondary'} onClick={() => setSharp(true)}>{t('settings.appearance.square', 'Square')}</Button>
          <Button size="sm" variant={!sharp ? 'primary' : 'secondary'} onClick={() => setSharp(false)}>{t('settings.appearance.rounded', 'Rounded')}</Button>
        </div>
        <h4 className="text-md font-medium text-white mt-4">{t('settings.log.title', 'Log verbosity')}</h4>
        <div className="flex flex-wrap gap-2">
          {(['silent','error','warn','info','debug'] as LogLevel[]).map((lvl) => (
            <Button
              key={lvl}
              size="sm"
              className={`${logLevel===lvl ? 'bg-blue-600' : 'bg-gray-600 hover:bg-gray-700'}`}
              onClick={() => applyLogLevel(lvl)}
            >
              {lvl}
            </Button>
          ))}
        </div>
        <p className="text-xs text-gray-400">{t('settings.log.note', 'Lower levels reduce console noise in production.')}</p>
      </div>

      {/* Actions */}
      <div className="flex gap-3 pt-4 border-t border-gray-600">
        <Button
          variant="primary"
          onClick={handleApplySettings}
          disabled={isLoading}
          className="flex-1"
        >
          {isLoading ? 'Applying...' : t('settings.actions.apply', 'Apply Settings')}
        </Button>
        <Button
          variant="secondary"
          onClick={handleReset}
          className="px-4"
        >
          {t('settings.actions.reset', 'Reset')}
        </Button>
      </div>

      {/* Message */}
      {message && (
        <div className={`p-3 rounded-lg text-sm ${
          message.includes('Error') 
            ? 'bg-red-500/20 border border-red-500/30 text-red-300' 
            : 'bg-green-500/20 border border-green-500/30 text-green-300'
        }`}>
          {message}
        </div>
      )}

      {/* Info */}
      <div className="bg-blue-500/10 border border-blue-500/20 rounded-lg p-3">
        <p className="text-blue-300 text-sm">
          <strong>{t('settings.note.title', 'Note:')}</strong> {t('settings.note.text', 'Compact mode is used when the app starts and when Advanced mode is disabled. Fullscreen mode is used when Advanced mode is enabled.')}
        </p>
      </div>
        </div>
      ) : (
        <div className="theme-card p-6 bg-[#10161D] border border-gray-700 shadow-lg space-y-4">
          {!tauriAvailable && (
            <div className="bg-yellow-500/10 border border-yellow-500/30 rounded-lg p-3 text-sm text-yellow-200">
              {t('settings.openapi.desktop_required', 'OpenAPI management is available only in the desktop application.')}
            </div>
          )}
          {tauriAvailable && (
            <div className="space-y-4">
              <div className="flex flex-wrap items-center justify-between gap-3">
                <div>
                  <h3 className="text-lg font-semibold text-white">
                    {t('settings.openapi.system_heading', 'OpenAPI Schema Manager')}
                  </h3>
                  <p className="text-xs text-gray-400">
                    {t('settings.openapi.updated', 'Last updated: {{time}}', { time: formattedUpdatedAt })}
                  </p>
                  <p className="text-xs text-gray-400">
                    {t('settings.openapi.active_format', 'Active format: {{format}}', { format: schemaFormat.toUpperCase() })}
                  </p>
                </div>
                <div className="flex flex-wrap gap-2">
                  <Button
                    size="sm"
                    variant={schemaFormat === 'yaml' ? 'primary' : 'secondary'}
                    onClick={() => handleFormatChange('yaml')}
                    disabled={schemaLoading}
                  >
                    YAML
                  </Button>
                  <Button
                    size="sm"
                    variant={schemaFormat === 'json' ? 'primary' : 'secondary'}
                    onClick={() => handleFormatChange('json')}
                    disabled={schemaLoading}
                  >
                    JSON
                  </Button>
                </div>
              </div>

              {schemaEndpoints && (
                <div className="bg-slate-900/60 border border-slate-700 rounded-lg p-3 text-xs text-gray-300 space-y-2">
                  <div>
                    <span className="text-gray-400">{t('settings.openapi.endpoint_base', 'Base')}</span>
                    <code className="block text-blue-300 break-all">{schemaEndpoints.base_url}</code>
                  </div>
                  <div className="grid gap-2 md:grid-cols-3">
                    <div>
                      <span className="text-gray-400">{t('settings.openapi.endpoint_json', 'JSON')}</span>
                      <code className="block text-blue-300 break-all">{schemaEndpoints.json}</code>
                    </div>
                    <div>
                      <span className="text-gray-400">{t('settings.openapi.endpoint_yaml', 'YAML')}</span>
                      <code className="block text-blue-300 break-all">{schemaEndpoints.yaml}</code>
                    </div>
                    <div>
                      <span className="text-gray-400">{t('settings.openapi.endpoint_health', 'Health')}</span>
                      <code className="block text-blue-300 break-all">{schemaEndpoints.health}</code>
                    </div>
                  </div>
                </div>
              )}

              <div>
                <textarea
                  value={schemaText}
                  onChange={(e) => handleSchemaChange(e.target.value)}
                  spellCheck={false}
                  className="w-full h-96 font-mono text-sm bg-[#0B1118] border border-gray-700 rounded-lg p-4 text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
                {schemaDirty && (
                  <p className="text-xs text-amber-300 mt-2">
                    {t('settings.openapi.unsaved', 'You have unsaved changes. Validate and save to apply.')}
                  </p>
                )}
              </div>

              <div className="flex flex-wrap gap-3">
                <Button variant="primary" onClick={handleValidateSchema} disabled={schemaLoading}>
                  {schemaLoading ? t('settings.openapi.validating', 'Working...') : t('settings.openapi.validate', 'Validate')}
                </Button>
                <Button variant="primary" onClick={handleSaveSchema} disabled={schemaLoading}>
                  {schemaLoading ? t('settings.openapi.saving', 'Working...') : t('settings.openapi.save', 'Save')}
                </Button>
                <Button variant="secondary" onClick={handleExportSchema} disabled={schemaLoading}>
                  {t('settings.openapi.export', 'Export')}
                </Button>
                <Button variant="secondary" onClick={handleUploadSchema} disabled={schemaLoading}>
                  {t('settings.openapi.upload', 'Upload')}
                </Button>
              </div>

              {schemaMessage && (
                <div className="bg-green-500/10 border border-green-500/30 text-green-200 text-sm rounded-lg p-3">
                  {schemaMessage}
                </div>
              )}

              {schemaError && (
                <div className="bg-red-500/10 border border-red-500/30 text-red-200 text-sm rounded-lg p-3">
                  {schemaError}
                </div>
              )}

              {schemaValidation && schemaValidation.valid && !schemaDirty && (
                <div className="bg-green-500/10 border border-green-500/30 text-green-200 text-xs rounded-lg p-3">
                  {t('settings.openapi.last_validation_passed', 'Last validation passed with no errors.')}
                </div>
              )}

              {schemaValidation && !schemaValidation.valid && schemaValidation.errors.length > 0 && (
                <div className="bg-red-500/10 border border-red-500/40 rounded-lg p-4 space-y-2">
                  <h4 className="text-sm font-semibold text-red-200">
                    {t('settings.openapi.errors_title', 'Validation errors')}
                  </h4>
                  <ul className="space-y-2">
                    {schemaValidation.errors.map((err, idx) => (
                      <li
                        key={`${err.pointer || 'error'}-${idx}`}
                        className="bg-red-500/10 border border-red-500/30 rounded-md p-2 text-xs text-red-200"
                      >
                        <div className="font-medium">{err.message}</div>
                        <div className="mt-1 text-red-300 space-x-3">
                          {err.pointer && (
                            <span>
                              {t('settings.openapi.pointer', 'Pointer')}: <code>{err.pointer}</code>
                            </span>
                          )}
                          {typeof err.line === 'number' && typeof err.column === 'number' && (
                            <span>
                              {t('settings.openapi.location', 'Line {{line}}, Column {{column}}', {
                                line: err.line,
                                column: err.column,
                              })}
                            </span>
                          )}
                        </div>
                      </li>
                    ))}
                  </ul>
                </div>
              )}
            </div>
          )}
        </div>
      )}
    </div>
  );
};

export default AppSettingsSection;
