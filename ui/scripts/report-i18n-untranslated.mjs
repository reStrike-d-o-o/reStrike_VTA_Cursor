import fs from 'fs';
import path from 'path';

const uiRoot = path.resolve(process.cwd());
const publicI18nDir = path.join(uiRoot, 'public', 'i18n');
const enPath = path.join(publicI18nDir, 'en.json');

function loadJson(p) {
  try { return JSON.parse(fs.readFileSync(p, 'utf8')); } catch { return {}; }
}

function main() {
  if (!fs.existsSync(publicI18nDir)) throw new Error('Missing public/i18n');
  const en = loadJson(enPath);
  const enKeys = Object.keys(en);
  const files = fs.readdirSync(publicI18nDir).filter(f => f.endsWith('.json') && f !== 'en.json');
  const report = [];
  for (const f of files) {
    const loc = path.basename(f, '.json');
    const data = loadJson(path.join(publicI18nDir, f));
    let missing = 0; let identical = 0;
    for (const k of enKeys) {
      const v = data[k];
      if (v === undefined) missing++;
      else if (v === en[k]) identical++;
    }
    report.push({ loc, total: enKeys.length, missing, identical, translated: enKeys.length - missing - identical });
  }
  report.sort((a,b) => a.loc.localeCompare(b.loc));
  console.log('Locale translation report (identical values considered untranslated):');
  for (const r of report) {
    console.log(`${r.loc}: translated=${r.translated}, identical=${r.identical}, missing=${r.missing}, total=${r.total}`);
  }
}

try { main(); } catch (e) { console.error('[i18n-report] ERROR:', e.message); process.exit(1); }


