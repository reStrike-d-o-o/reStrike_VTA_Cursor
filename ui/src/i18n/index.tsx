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
    'common.retry': 'Pokušaj ponovo',
    'common.unknown_error': 'Nepoznata greška',
    // Tabs
    'settings.tabs.diagnostics': 'Dijagnostika i menadžer logova',
    'settings.tabs.database': 'Baza podataka',
    'settings.tabs.backup': 'Bekap i obnova',
    'settings.tabs.app': 'Podešavanja aplikacije',
    'ivr.tabs.history': 'Istorija mečeva',
    'ivr.tabs.settings': 'IVR video podešavanja',
    'ovr.tabs.integration': 'Integracija',
    'ovr.tabs.tournament': 'Upravljanje turnirom',
    'obs.tabs.websocket': 'WebSocket',
    'obs.tabs.control_room': 'Kontrolna soba',
    'obs.tabs.integration': 'Integracija',
    'pss.tabs.udp': 'UDP server i protokol',
    'pss.tabs.flags': 'Upravljanje zastavama',
    'pss.tabs.scoreboard': 'Semafor',
    'pss.tabs.simulation': 'Simulacija',
    'pss.tabs.triggers': 'Okidači',
    // Settings content
    'settings.language': 'Jezik',
    'settings.select_language': 'Izaberite jezik',
    'settings.window.title': 'Podešavanja prozora',
    'settings.window.help': 'Podesite dimenzije prozora za kompaktni i fullscreen režim.',
    'settings.window.compact': 'Kompaktni režim (podrazumevano)',
    'settings.window.full': 'Fullscreen režim (napredno)',
    'settings.window.width': 'Širina (px)',
    'settings.window.height': 'Visina (px)',
    'settings.quick.title': 'Brze postavke',
    'settings.appearance.title': 'Izgled',
    'settings.appearance.theme': 'Tema',
    'settings.appearance.corners': 'Uglovi',
    'settings.appearance.dark': 'Tamna',
    'settings.appearance.light': 'Svetla',
    'settings.appearance.square': 'Kvadratni',
    'settings.appearance.rounded': 'Zaobljeni',
    'settings.log.title': 'Nivo zapisivanja',
    'settings.log.note': 'Niži nivoi smanjuju šum u konzoli u produkciji.',
    'settings.actions.apply': 'Primeni podešavanja',
    'settings.actions.reset': 'Resetuj',
    'settings.note.title': 'Napomena:',
    'settings.note.text': 'Kompaktni režim se koristi pri pokretanju aplikacije i kada je napredni režim isključen. Fullscreen režim se koristi kada je napredni režim uključen.',
    // Logs (subset)
    'logs.title': 'Preuzimanje logova',
    'logs.type': 'Tip:',
    'logs.select_type': 'Izaberite tip loga',
    'logs.table.file': 'Naziv fajla',
    'logs.table.size': 'Veličina',
    'logs.table.modified': 'Izmenjeno',
    'logs.loading': 'Učitavanje logova...',
    'logs.downloading': 'Preuzimanje...',
    'logs.archive': 'Arhiva',
    'logs.double_click': 'Dupli klik za preuzimanje',
    'logs.none': 'Nema pronađenih logova',
    'logs.none_error': 'Nema logova zbog greške',
    'logs.error.fetch_archives': 'Neuspešno preuzimanje arhiva',
    'logs.error.fetch_logs': 'Neuspešno preuzimanje logova',
    'logs.error.timeout': 'Komanda je istekla. Backend je zauzet ili ne odgovara. Pokušajte ponovo.',
    'logs.error.tauri_missing': 'Tauri nije dostupan. Pokrenite aplikaciju u desktop režimu.',
    'logs.error.generic_fetch': 'Greška pri preuzimanju {what}: {msg}',
    'logs.error.download_timeout': 'Isteklo vreme za preuzimanje {name}. Pokušajte ponovo.',
    'logs.error.download_failed': 'Neuspešno preuzimanje {name}: {msg}',
    'logs.error.tauri_no_download': 'Tauri nije dostupan. Ne može da se preuzme {name} u web režimu.',
    'logs.error.download_error': 'Greška pri preuzimanju {name}: {msg}',
  },
  hr: {
    'drawer.pss': 'PSS',
    'drawer.obs': 'OBS',
    'drawer.ovr': 'OVR',
    'drawer.ivr': 'IVR',
    'drawer.ai': 'AI',
    'drawer.settings': 'Postavke',
    'app.title': 'reStrike VTA - Windows Desktop',
    'env.windows_native': 'Windows aplikacija',
    'env.web_mode': 'Web način',
    'status.ready': 'Status: Spremno',
    'action.generic': 'Akcija',
    'settings.tabs.diagnostics': 'Dijagnostika i upravljanje logovima',
    'settings.tabs.database': 'Baza podataka',
    'settings.tabs.backup': 'Sigurnosna kopija i vraćanje',
    'settings.tabs.app': 'Postavke aplikacije',
    'ivr.tabs.history': 'Povijest mečeva',
    'ivr.tabs.settings': 'IVR video postavke',
    'ovr.tabs.integration': 'Integracija',
    'ovr.tabs.tournament': 'Upravljanje turnirom',
    'obs.tabs.websocket': 'WebSocket',
    'obs.tabs.control_room': 'Kontrolna soba',
    'obs.tabs.integration': 'Integracija',
    'pss.tabs.udp': 'UDP poslužitelj i protokol',
    'pss.tabs.flags': 'Upravljanje zastavama',
    'pss.tabs.scoreboard': 'Semafor',
    'pss.tabs.simulation': 'Simulacija',
    'pss.tabs.triggers': 'Okidači',
    'settings.language': 'Jezik',
    'settings.select_language': 'Odaberite jezik',
    'settings.window.title': 'Postavke prozora',
    'settings.window.help': 'Konfigurirajte dimenzije prozora za kompaktni i puni zaslon.',
    'settings.window.compact': 'Kompaktni način (zadano)',
    'settings.window.full': 'Način preko cijelog zaslona (napredno)',
    'settings.window.width': 'Širina (px)',
    'settings.window.height': 'Visina (px)',
    'settings.quick.title': 'Brze postavke',
    'settings.appearance.title': 'Izgled',
    'settings.appearance.theme': 'Tema',
    'settings.appearance.corners': 'Rubovi',
    'settings.appearance.dark': 'Tamna',
    'settings.appearance.light': 'Svjetla',
    'settings.appearance.square': 'Kvadratni',
    'settings.appearance.rounded': 'Zaobljeni',
    'settings.log.title': 'Razina zapisivanja',
    'settings.log.note': 'Niže razine smanjuju šum konzole u produkciji.',
    'settings.actions.apply': 'Primijeni postavke',
    'settings.actions.reset': 'Resetiraj',
    'settings.note.title': 'Napomena:',
    'settings.note.text': 'Kompaktni način se koristi pri pokretanju aplikacije i kada je napredni način isključen. Način preko cijelog zaslona se koristi kada je napredni način uključen.',
  },
  de: {
    'drawer.pss': 'PSS',
    'drawer.obs': 'OBS',
    'drawer.ovr': 'OVR',
    'drawer.ivr': 'IVR',
    'drawer.ai': 'AI',
    'drawer.settings': 'Einstellungen',
    'app.title': 'reStrike VTA - Windows Desktop',
    'env.windows_native': 'Windows-Anwendung',
    'env.web_mode': 'Web-Modus',
    'status.ready': 'Status: Bereit',
    'action.generic': 'Aktion',
    'settings.tabs.diagnostics': 'Diagnostik & Protokollverwaltung',
    'settings.tabs.database': 'Datenbank',
    'settings.tabs.backup': 'Sicherung & Wiederherstellung',
    'settings.tabs.app': 'App-Einstellungen',
    'ivr.tabs.history': 'Match-Verlauf',
    'ivr.tabs.settings': 'IVR Videoeinstellungen',
    'ovr.tabs.integration': 'Integration',
    'ovr.tabs.tournament': 'Turnierverwaltung',
    'obs.tabs.websocket': 'WebSocket',
    'obs.tabs.control_room': 'Kontrollraum',
    'obs.tabs.integration': 'Integration',
    'pss.tabs.udp': 'UDP-Server & Protokoll',
    'pss.tabs.flags': 'Flaggenverwaltung',
    'pss.tabs.scoreboard': 'Anzeigetafel',
    'pss.tabs.simulation': 'Simulation',
    'pss.tabs.triggers': 'Auslöser',
    'settings.language': 'Sprache',
    'settings.select_language': 'Sprache auswählen',
  },
  fr: {
    'drawer.pss': 'PSS',
    'drawer.obs': 'OBS',
    'drawer.ovr': 'OVR',
    'drawer.ivr': 'IVR',
    'drawer.ai': 'AI',
    'drawer.settings': 'Paramètres',
    'app.title': 'reStrike VTA - Windows Desktop',
    'env.windows_native': 'Application Windows',
    'env.web_mode': 'Mode Web',
    'status.ready': 'Statut : Prêt',
    'action.generic': 'Action',
    'settings.tabs.diagnostics': 'Diagnostic et gestion des journaux',
    'settings.tabs.database': 'Base de données',
    'settings.tabs.backup': 'Sauvegarde et restauration',
    'settings.tabs.app': "Paramètres de l'application",
    'ivr.tabs.history': 'Historique des matchs',
    'ivr.tabs.settings': 'Paramètres vidéo IVR',
    'ovr.tabs.integration': 'Intégration',
    'ovr.tabs.tournament': 'Gestion du tournoi',
    'obs.tabs.websocket': 'WebSocket',
    'obs.tabs.control_room': 'Salle de contrôle',
    'obs.tabs.integration': 'Intégration',
    'pss.tabs.udp': 'Serveur UDP et protocole',
    'pss.tabs.flags': 'Gestion des drapeaux',
    'pss.tabs.scoreboard': "Tableau d'affichage",
    'pss.tabs.simulation': 'Simulation',
    'pss.tabs.triggers': 'Déclencheurs',
    'settings.language': 'Langue',
    'settings.select_language': 'Sélectionnez la langue',
  },
};

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
      if (stored) return stored;
      if (defaultLocale) return defaultLocale;
      if (typeof navigator !== 'undefined' && navigator.language) {
        const short = navigator.language.split('-')[0];
        if (catalogs[short]) return short;
      }
    } catch {}
    return 'en';
  }, [defaultLocale]);

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
  }, [locale]);

  const setLocale = (loc: string) => {
    setLocaleState(catalogs[loc] ? loc : 'en');
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
