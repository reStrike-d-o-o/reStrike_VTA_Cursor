import React, { createContext, useContext, useEffect, useMemo, useState } from 'react';

type Catalog = Record<string, string>;
interface Catalogs { [locale: string]: Catalog }

interface I18nContextValue {
        locale: string;
        setLocale: (locale: string) => void;
        t: (id: string, fallback?: string, values?: Record<string, string | number>) => string;
}

const I18N_STORAGE_KEY = 'app.locale';
const I18nContext = createContext<I18nContextValue | undefined>(undefined);

const catalogs: Catalogs = {
        en: {
                'drawer.pss': 'PSS',
                'drawer.obs': 'OBS',
                'drawer.ovr': 'OVR',
                'drawer.ivr': 'IVR',
                'drawer.ai': 'AI',
                'drawer.settings': 'Settings',
                'app.title': 'reStrike VTA - Windows Desktop',
                'env.windows_native': 'Windows Native',
                'env.web_mode': 'Web Mode',
                'status.ready': 'Status: Ready',
                'action.generic': 'Action',
                'common.retry': 'Retry',
                'common.unknown_error': 'Unknown error',
                // Tabs
                'settings.tabs.diagnostics': 'Diagnostics & Logs Manager',
                'settings.tabs.database': 'Database',
                'settings.tabs.backup': 'Backup & Restore',
                'settings.tabs.app': 'App Settings',
                'ivr.tabs.history': 'Match History',
                'ivr.tabs.settings': 'IVR Video Settings',
                'ovr.tabs.integration': 'Integration',
                'ovr.tabs.tournament': 'Tournament Management',
                'obs.tabs.websocket': 'WebSocket',
                'obs.tabs.control_room': 'Control Room',
                'obs.tabs.integration': 'Integration',
                'pss.tabs.udp': 'UDP Server & Protocol',
                'pss.tabs.flags': 'Flag Management',
                'pss.tabs.scoreboard': 'Scoreboard',
                'pss.tabs.simulation': 'Simulation',
                'pss.tabs.triggers': 'Triggers',
                // Settings content
                'settings.language': 'Language',
                'settings.select_language': 'Select language',
                'settings.window.title': 'Window Settings',
                'settings.window.help': 'Configure the window dimensions for compact and fullscreen modes.',
                'settings.window.compact': 'Compact Mode (Default)',
                'settings.window.full': 'Fullscreen Mode (Advanced)',
                'settings.window.width': 'Width (px)',
                'settings.window.height': 'Height (px)',
                'settings.quick.title': 'Quick Presets',
                'settings.appearance.title': 'Appearance',
                'settings.appearance.theme': 'Theme',
                'settings.appearance.corners': 'Corners',
                'settings.appearance.dark': 'Dark',
                'settings.appearance.light': 'Light',
                'settings.appearance.square': 'Square',
                'settings.appearance.rounded': 'Rounded',
                'settings.log.title': 'Log verbosity',
                'settings.log.note': 'Lower levels reduce console noise in production.',
                'settings.actions.apply': 'Apply Settings',
                'settings.actions.reset': 'Reset',
                'settings.note.title': 'Note:',
                'settings.note.text': 'Compact mode is used when the app starts and when Advanced mode is disabled. Fullscreen mode is used when Advanced mode is enabled.',
                // Logs
                'logs.title': 'Download Logs',
                'logs.type': 'Type:',
                'logs.select_type': 'Select log type',
                'logs.types.pss': 'PSS',
                'logs.types.obs': 'OBS',
                'logs.types.udp': 'UDP',
                'logs.types.websocket': 'WebSocket',
                'logs.types.db': 'Database',
                'logs.types.app': 'Application',
                'logs.types.arc': 'Archive',
                'logs.table.file': 'File Name',
                'logs.table.size': 'Size',
                'logs.table.modified': 'Modified',
                'logs.loading': 'Loading logs...',
                'logs.downloading': 'Downloading...',
                'logs.archive': 'Archive',
                'logs.double_click': 'Double-click to download',
                'logs.none': 'No logs found',
                'logs.none_error': 'No logs found due to error',
                'logs.error.fetch_archives': 'Failed to fetch archive files',
                'logs.error.fetch_logs': 'Failed to fetch log files',
                'logs.error.timeout': 'Command timed out. The backend may be busy or unresponsive. Please try again.',
                'logs.error.tauri_missing': 'Tauri not available. Please ensure the app is running in desktop mode.',
                'logs.error.generic_fetch': 'Error fetching {what}: {msg}',
                'logs.error.download_timeout': 'Download timed out for {name}. Please try again.',
                'logs.error.download_failed': 'Failed to download {name}: {msg}',
                'logs.error.tauri_no_download': 'Tauri not available. Cannot download {name} in web mode.',
                'logs.error.download_error': 'Error downloading {name}: {msg}',
                // Archive manager & backup
                'logs.archive_mgr.title': 'Log Archive Manager',
                'logs.archive_mgr.load_failed': 'Failed to load configuration',
                'logs.archive_mgr.load_failed_with': 'Failed to load configuration: {err}',
                'logs.archive_mgr.save_ok': 'Configuration saved successfully',
                'logs.archive_mgr.save_failed': 'Failed to save configuration',
                'logs.archive_mgr.save_failed_with': 'Failed to save configuration: {err}',
                'logs.archive_mgr.create_ok': 'Archive created successfully: {name}',
                'logs.archive_mgr.create_failed': 'Failed to create archive',
                'logs.archive_mgr.create_failed_with': 'Failed to create archive: {err}',
                'logs.archive_mgr.upload_ok': 'Archive uploaded successfully',
                'logs.archive_mgr.upload_failed': 'Failed to upload archive',
                'logs.archive_mgr.upload_failed_with': 'Failed to upload archive: {err}',
                'logs.archive_mgr.upload_cleanup_ok': 'Archive uploaded and cleaned up successfully',
                'logs.archive_mgr.upload_cleanup_failed': 'Failed to upload and cleanup archive',
                'logs.archive_mgr.upload_cleanup_failed_with': 'Failed to upload and cleanup archive: {err}',
                'logs.archive_mgr.auto_ok': 'Auto-archive completed successfully',
                'logs.archive_mgr.auto_failed': 'Auto-archive failed',
                'logs.archive_mgr.auto_failed_with': 'Auto-archive failed: {err}',
                'logs.archive_mgr.create_title': 'Create local archive',
                'logs.archive_mgr.create_upload_title': 'Create and upload to Drive',
                'logs.archive_mgr.upload_delete_title': 'Upload and delete local copy',
                'logs.archive_mgr.auto_label': 'Auto-Archive',
                'logs.archive_mgr.schedule': 'Schedule',
                'logs.archive_mgr.weekly': 'Weekly',
                'logs.archive_mgr.monthly': 'Monthly',
                'logs.archive_mgr.quarterly': 'Every 3 months',
                'logs.archive_mgr.biannual': 'Every 6 months',
                'logs.archive_mgr.annual': 'Annually',
                'logs.archive_mgr.upload': 'Upload',
                'logs.archive_mgr.delete': 'Delete',
                'logs.archive_mgr.run_now': 'Run Now',
                'logs.archive_mgr.status': 'Status',
                'logs.archive_mgr.enabled': 'Enabled',
                'logs.archive_mgr.disabled': 'Disabled',
                'logs.archive_mgr.next': 'Next',
                'logs.archive_mgr.due': 'Archive is due',
                'common.creating': 'Creating…',
                'common.create': 'Create',
                'common.uploading': 'Uploading…',
                'common.processing': 'Processing…',
                'common.running': 'Running…',
                'common.error': 'Error',
                'common.success': 'Success',
                'backup.title': 'Backup & Restore',
                'backup.subtitle': 'Manage local backups and Google Drive integration',
                'backup.tabs.local': 'Local Backup',
                'backup.tabs.drive': 'Google Drive',
                'backup.local.title': 'Local Backup',
                'backup.local.create': 'Create Backup',
                'backup.table.file': 'File Name',
                'backup.table.size': 'Size',
                'backup.table.modified': 'Modified',
                'backup.table.action': 'Action',
                'backup.none': 'No backup files found',
                'backup.none_hint': 'Create a backup to see files here',
                'backup.restoring': 'Restoring...',
                'backup.restore': 'Restore',
                'backup.create_ok': 'Backup created successfully',
                'backup.create_failed': 'Failed to create backup',
                'backup.confirm_restore': 'Are you sure you want to restore from this backup? This will overwrite current settings.',
                'backup.restore_ok': 'Backup restored successfully',
                'backup.restore_failed': 'Failed to restore backup',
                'backup.drive.title': 'Google Drive Integration',
                'backup.drive.subtitle': 'Backup and restore using Google Drive',
                // Analytics
                'analytics.title': '📊 Analytics Dashboard',
                'analytics.realtime': 'Real-time',
                'analytics.tabs.tournament': 'Tournament',
                'analytics.tabs.athlete': 'Athlete',
                'analytics.tabs.match': 'Match',
                'analytics.tabs.day': 'Day',
                'analytics.select.tournament': 'Select Tournament',
                'analytics.select.athlete': 'Select Athlete',
                'analytics.select.match': 'Select Match',
                'analytics.select.date': 'Select Date',
                'analytics.placeholder.tournament': 'Choose a tournament',
                'analytics.placeholder.athlete': 'Choose an athlete',
                'analytics.placeholder.match': 'Choose a match',
                'analytics.placeholder.date': 'Choose a date',
                'analytics.all_tournaments': 'All Tournaments',
                'analytics.item.tournament': 'Tournament {id}',
                'analytics.item.match': 'Match {id}',
                'analytics.athlete.title': 'Athlete Analytics',
                'analytics.athlete.help': 'Select an athlete from the dropdown above to view their detailed analytics and performance metrics.',
                'analytics.match.title': 'Match Analytics',
                'analytics.match.help': 'Select a match from the dropdown above to view detailed match analytics and performance metrics.',
                'analytics.overview.title': '📈 Analytics Overview',
                'analytics.metrics.total_events': 'Total Events',
                'analytics.metrics.unique_matches': 'Unique Matches',
                'analytics.metrics.unique_athletes': 'Unique Athletes',
                'analytics.metrics.tournaments': 'Tournaments',
                'analytics.badge': 'Analytics',
                'analytics.sections.overview': 'Overview',
                'analytics.sections.performance': 'Performance',
                'analytics.sections.matches': 'Matches',
                'analytics.sections.trends': 'Statistics',
                'analytics.athlete.total_matches': 'Total Matches',
                'analytics.athlete.win_rate': 'Win Rate',
                'analytics.athlete.total_points': 'Total Points',
                'analytics.athlete.avg_points': 'Avg Points/Match',
                'analytics.athlete.wins': 'Wins',
                'analytics.athlete.losses': 'Losses',
                'analytics.athlete.warnings': 'Warnings',
                'analytics.athlete.best_performance': 'Best Performance',
                'analytics.athlete.match_points': 'Match Points:',
                'analytics.athlete.match_id': 'Match ID:',
                'analytics.athlete.date': 'Date:',
                'analytics.athlete.no_performance': 'No performance data available',
                'analytics.athlete.total_points_scored': 'Total Points Scored',
                'analytics.athlete.total_warnings': 'Total Warnings',
                'analytics.athlete.recent_history': 'Recent Match History',
                'analytics.athlete.recent_summary': '{wins} wins, {losses} losses in {total} total matches',
                'analytics.athlete.no_history': 'No match history available',
                'analytics.athlete.performance_trends': 'Performance Trends',
                'analytics.athlete.win_rate_trend': 'Win Rate Trend:',
                'analytics.athlete.improving': '↗️ Improving',
                'analytics.athlete.declining': '↘️ Declining',
                'analytics.athlete.points_per_match': 'Points Per Match:',
                'analytics.athlete.discipline': 'Discipline:',
                'analytics.athlete.good': 'Good',
                'analytics.athlete.needs_improvement': 'Needs Improvement',
                // Match analytics
                'analytics.match.header': 'Match Analytics',
                'analytics.match.completed': 'Completed',
                'analytics.match.in_progress': 'In Progress',
                'analytics.match.winner': 'Winner: {name}',
                'analytics.match.tabs.athletes': 'Athletes',
                'analytics.match.tabs.events': 'Events',
                'analytics.match.duration': 'Duration',
                'analytics.match.intensity': 'Match Intensity',
                'analytics.match.events_per_min': 'events/min',
                'analytics.match.points_scored': 'Points Scored',
                'analytics.match.points': 'Points',
                'analytics.match.warnings': 'Warnings',
                'analytics.match.injuries': 'Injuries',
                'analytics.match.total_events': 'Total Events',
                'analytics.match.points_events': 'Points Events',
                'analytics.match.warning_events': 'Warning Events',
                'analytics.match.injury_events': 'Injury Events',
                'analytics.match.other_events': 'Other Events',
                'analytics.match.event_distribution': 'Event Distribution',
                'analytics.match.other': 'Other',
                'analytics.match.timeline': 'Match Timeline',
                'analytics.match.start_time': 'Start Time:',
                'analytics.match.end_time': 'End Time:',
                'analytics.match.events_per_minute': 'Events per Minute:',
                'analytics.match.status': 'Match Status',
                'analytics.match.status_label': 'Status:',
                'analytics.match.winner_label': 'Winner:',
                'analytics.match.id': 'Match ID:',
        },
};

// Prefer external JSON catalogs: clear built-in non-English locales so JSON becomes authoritative
try {
        ['hr', 'sr', 'de', 'fr', 'es', 'it', 'bs', 'zh', 'ru'].forEach((loc) => { (catalogs as any)[loc] = {}; });
} catch { }

function validateCatalogsForMissingKeys() {
        try {
                const base = (catalogs as any).en || {};
                const baseKeys = Object.keys(base);
                Object.keys(catalogs).forEach((loc) => {
                        const missing = baseKeys.filter((k) => (catalogs as any)[loc]?.[k] === undefined);
                        if (missing.length > 0) {
                                console.warn(`[i18n] Locale ${loc} is missing ${missing.length} keys compared to en`);
                        }
                });
        } catch { }
}

try { validateCatalogsForMissingKeys(); } catch { }

function normalizeLocale(input: string | undefined | null): string {
        if (!input || typeof input !== 'string') return 'en';
        try {
                const short = input.toLowerCase().split('-')[0];
                const exists = Boolean((catalogs as any)[short]);
                return exists ? short : 'en';
        } catch {
                return 'en';
        }
}

function interpolate(str: string, values?: Record<string, string | number>): string {
        if (!str || !values) return str;
        return str.replace(/\{(.*?)\}/g, (_m, key) => {
                const v = values[key.trim()];
                return v === undefined || v === null ? '' : String(v);
        });
}

export const I18nProvider: React.FC<React.PropsWithChildren<{ defaultLocale?: string }>> = ({ children, defaultLocale }) => {
        const initialLocale = useMemo(() => {
                try {
                        const stored = localStorage.getItem(I18N_STORAGE_KEY);
                        if (stored) {
                                const norm = normalizeLocale(stored);
                                console.log('[i18n] initial from localStorage:', { stored, norm });
                                return norm;
                        }
                        if (defaultLocale) {
                                const norm = normalizeLocale(defaultLocale);
                                console.log('[i18n] initial from defaultLocale prop:', { defaultLocale, norm });
                                return norm;
                        }
                        if (typeof navigator !== 'undefined' && navigator.language) {
                                const short = normalizeLocale(navigator.language);
                                const exists = Boolean((catalogs as any)[short]);
                                console.log('[i18n] initial from navigator:', { navigatorLanguage: navigator.language, short, exists });
                                if (exists) return short;
                        }
                } catch { }
                return 'en';
        }, [defaultLocale]);

        const [locale, setLocaleState] = useState<string>(initialLocale);
        const [reloadVersion, setReloadVersion] = useState<number>(0);
        const loadedLocalesRef = React.useRef<Record<string, boolean>>({});

        useEffect(() => {
                try {
                        localStorage.setItem(I18N_STORAGE_KEY, locale);
                        console.log('[i18n] persisted locale to storage:', locale);
                } catch (e) {
                        console.warn('[i18n] failed to persist locale:', e);
                }
        }, [locale]);

        const t = useMemo(() => {
                return (id: string, fallback?: string, values?: Record<string, string | number>) => {
                        const primary = catalogs[locale] || {};
                        const en = catalogs.en || {};
                        let raw: string | undefined = primary[id];
                        if (raw === undefined) {
                                raw = en[id];
                        }
                        if (raw === undefined) {
                                raw = fallback ?? id;
                                if (primary[id] === undefined && fallback === undefined) {
                                        console.debug('[i18n] missing key; using id as string', { locale, id });
                                }
                        }
                        return interpolate(raw, values);
                };
        }, [locale, reloadVersion]);

        // Attempt to load external JSON catalog for the active locale and replace it
        useEffect(() => {
                const code = locale;
                if (!code) return;
                if (loadedLocalesRef.current[code]) return;
                const url = `/i18n/${code}.json`;
                fetch(url)
                        .then(res => {
                                if (!res.ok) throw new Error(`http ${res.status}`);
                                return res.json();
                        })
                        .then((data: Record<string, string>) => {
                                try {
                                        (catalogs as any)[code] = data;
                                        loadedLocalesRef.current[code] = true;
                                        setReloadVersion(v => v + 1);
                                        console.log('[i18n] loaded external catalog', { code, keys: Object.keys(data).length });
                                } catch (e) {
                                        console.warn('[i18n] failed to apply external catalog', { code, e });
                                }
                        })
                        .catch(() => {
                                // Silently ignore if file not present
                        });
        }, [locale]);

        const setLocale = (loc: string) => {
                const normalized = normalizeLocale(loc);
                setLocaleState(normalized);
                try { localStorage.setItem(I18N_STORAGE_KEY, normalized); } catch { }
                console.log('[i18n] setLocale called:', { requested: loc, normalized });
        };

        const value: I18nContextValue = { locale, setLocale, t };
        return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
};

export function useI18n(): I18nContextValue {
        const ctx = useContext(I18nContext);
        if (!ctx) throw new Error('useI18n must be used within I18nProvider');
        return ctx;
}

interface TProps { id: string; default?: string; values?: Record<string, string | number>; className?: string }
export const T: React.FC<TProps> = ({ id, default: fb, values, className }) => {
        const { t } = useI18n();
        return <span className={className}>{t(id, fb, values)}</span>;
};
