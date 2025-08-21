## Internationalization (i18n) Workflow

This document explains how translations are managed in the UI and how to keep all locales in sync.

### Overview
- **Canonical source**: `ui/public/i18n/en.json`
- **Locales** (JSON): `ui/public/i18n/<locale>.json` (e.g., `es.json`, `it.json`, `bs.json`, `zh.json`, `ru.json`, plus `hr`, `sr`, `de`, `fr`)
- **Runtime loading**: `ui/src/i18n/index.tsx` dynamically loads JSON catalogs and falls back to English.
- **Language switcher**: Alphabetically sorted; reads from available locale options.

### Common Commands
Run from the `ui` directory.

1) Extract keys from source and update English catalog:
```powershell
cd ui
npm run i18n:extract
```

2) Sync all non-English catalogs to ensure 100% key coverage (missing keys copied from English):
```powershell
npm run i18n:sync
```

3) Auto-translate missing or English-identical keys using Google Translate:
```powershell
# Option A: Using env var for this PowerShell session
$env:GOOGLE_TRANSLATE_API_KEY="YOUR_KEY"; npm run i18n:translate -- --locales=es,it,bs,zh,ru,hr,sr,de,fr

# Option B: Pass key as a flag
npm run i18n:translate -- --key=YOUR_KEY --locales=es,it,bs,zh,ru,hr,sr,de,fr
```

4) Report untranslated status (identical values count as untranslated):
```powershell
npm run i18n:report
```

Recommended sequence when adding new strings:
```powershell
cd ui
npm run i18n:update   # runs extract + sync
npm run i18n:translate -- --locales=es,it,bs,zh,ru,hr,sr,de,fr
npm run i18n:report
```

### Notes
- Placeholders like `{name}` and `%1$s` are preserved during translation.
- If some entries remain identical to English (proper nouns, acronyms), you can keep them as-is.
- English (`en.json`) is always the fallback; keep it complete and accurate.


