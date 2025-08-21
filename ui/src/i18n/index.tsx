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

const catalogs: Catalogs = {};

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

        // Ensure English catalog is loaded for fallback regardless of current locale
        useEffect(() => {
                if (loadedLocalesRef.current['en']) return;
                fetch('/i18n/en.json')
                        .then(res => { if (!res.ok) throw new Error(`http ${res.status}`); return res.json(); })
                        .then((data: Record<string, string>) => {
                                (catalogs as any)['en'] = data;
                                loadedLocalesRef.current['en'] = true;
                                setReloadVersion(v => v + 1);
                                console.log('[i18n] loaded external catalog', { code: 'en', keys: Object.keys(data).length });
                        })
                        .catch(() => { /* ignore */ });
        }, []);

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
