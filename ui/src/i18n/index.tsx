import React, { createContext, useContext, useEffect, useMemo, useState } from 'react';

type Catalog = Record<string, string>;
type Catalogs = Record<string, Catalog>;

interface I18nContextValue {
	locale: string;
	setLocale: (locale: string) => void;
	t: (id: string, fallback?: string, values?: Record<string, string | number>) => string;
}

const I18N_STORAGE_KEY = 'app.locale';

const I18nContext = createContext<I18nContextValue | undefined>(undefined);

// Simple default catalogs. Extend as needed.
const defaultCatalogs: Catalogs = {
	en: {
		// examples
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
		// Settings tabs
		'settings.tabs.diagnostics': 'Diagnostics & Logs Manager',
		'settings.tabs.database': 'Database',
		'settings.tabs.backup': 'Backup & Restore',
		'settings.tabs.app': 'App Settings',
		// IVR tabs
		'ivr.tabs.history': 'Match History',
		'ivr.tabs.settings': 'IVR Video Settings',
		// OVR tabs
		'ovr.tabs.integration': 'Integration',
		'ovr.tabs.tournament': 'Tournament Management',
		// OBS tabs
		'obs.tabs.websocket': 'WebSocket',
		'obs.tabs.control_room': 'Control Room',
		'obs.tabs.integration': 'Integration',
		// PSS tabs
		'pss.tabs.udp': 'UDP Server & Protocol',
		'pss.tabs.flags': 'Flag Management',
		'pss.tabs.scoreboard': 'Scoreboard',
		'pss.tabs.simulation': 'Simulation',
		'pss.tabs.triggers': 'Triggers',
		// Settings → Language
		'settings.language': 'Language',
		'settings.select_language': 'Select language',
	},
	sr: {
		'drawer.pss': 'PSS',
		'drawer.obs': 'OBS',
		'drawer.ovr': 'OVR',
		'drawer.ivr': 'IVR',
		'drawer.ai': 'AI',
		'drawer.settings': 'Podešavanja',
		'app.title': 'reStrike VTA - Windows Desktop',
		'env.windows_native': 'Windows aplikacija',
		'env.web_mode': 'Veb režim',
		'status.ready': 'Status: Spremno',
		'action.generic': 'Akcija',
		// Settings tabs
		'settings.tabs.diagnostics': 'Dijagnostika i menadžer logova',
		'settings.tabs.database': 'Baza podataka',
		'settings.tabs.backup': 'Bekap i obnova',
		'settings.tabs.app': 'Podešavanja aplikacije',
		// IVR tabs
		'ivr.tabs.history': 'Istorija mečeva',
		'ivr.tabs.settings': 'IVR video podešavanja',
		// OVR tabs
		'ovr.tabs.integration': 'Integracija',
		'ovr.tabs.tournament': 'Upravljanje turnirom',
		// OBS tabs
		'obs.tabs.websocket': 'WebSocket',
		'obs.tabs.control_room': 'Kontrolna soba',
		'obs.tabs.integration': 'Integracija',
		// PSS tabs
		'pss.tabs.udp': 'UDP server i protokol',
		'pss.tabs.flags': 'Upravljanje zastavama',
		'pss.tabs.scoreboard': 'Semafor',
		'pss.tabs.simulation': 'Simulacija',
		'pss.tabs.triggers': 'Okidači',
		// Settings → Language
		'settings.language': 'Jezik',
		'settings.select_language': 'Izaberite jezik',
	},
};

function interpolate(input: string, values?: Record<string, string | number>): string {
	if (!input || !values) return input;
	return input.replace(/\{(.*?)\}/g, (_m, key) => {
		const v = values[key.trim()];
		return v === undefined || v === null ? '' : String(v);
	});
}

export const I18nProvider: React.FC<React.PropsWithChildren<{ catalogs?: Catalogs; defaultLocale?: string }>> = ({
	children,
	catalogs = defaultCatalogs,
	defaultLocale,
}) => {
	const initialLocale = useMemo(() => {
		try {
			const stored = localStorage.getItem(I18N_STORAGE_KEY);
			if (stored) return stored;
			if (defaultLocale) return defaultLocale;
			if (typeof navigator !== 'undefined' && navigator.language) {
				const short = navigator.language.split('-')[0];
				if (catalogs[short]) return short;
			}
		} catch {}
		return 'en';
	}, [catalogs, defaultLocale]);

	const [locale, setLocaleState] = useState<string>(initialLocale);

	useEffect(() => {
		try { localStorage.setItem(I18N_STORAGE_KEY, locale); } catch {}
	}, [locale]);

	const t = useMemo(() => {
		return (id: string, fallback?: string, values?: Record<string, string | number>) => {
			const cat = catalogs[locale] || catalogs.en || {};
			const raw = cat[id] ?? fallback ?? id;
			return interpolate(raw, values);
		};
	}, [catalogs, locale]);

	const setLocale = (loc: string) => {
		if (!catalogs[loc]) {
			// fallback silently if catalog missing
			setLocaleState('en');
			return;
		}
		setLocaleState(loc);
	};

	const value = useMemo<I18nContextValue>(() => ({ locale, setLocale, t }), [locale, t]);

	return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
};

export function useI18n(): I18nContextValue {
	const ctx = useContext(I18nContext);
	if (!ctx) throw new Error('useI18n must be used within I18nProvider');
	return ctx;
}

interface TProps {
	id: string;
	default?: string;
	values?: Record<string, string | number>;
	className?: string;
}

export const T: React.FC<TProps> = ({ id, default: fallback, values, className }) => {
	const { t } = useI18n();
	return <span className={className}>{t(id, fallback, values)}</span>;
};


