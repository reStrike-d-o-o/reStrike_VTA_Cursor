import fs from 'fs';
import path from 'path';

const uiRoot = path.resolve(process.cwd());
const i18nIndexPath = path.join(uiRoot, 'src', 'i18n', 'index.tsx');
const publicI18nDir = path.join(uiRoot, 'public', 'i18n');
const locales = ['hr', 'sr', 'de', 'fr', 'es', 'it', 'bs', 'zh', 'ru'];

function readFileSafe(p) {
  return fs.readFileSync(p, 'utf8');
}

function extractEnBlock(tsx) {
  const marker = 'en:';
  const idx = tsx.indexOf(marker);
  if (idx === -1) throw new Error('Could not find en: block in index.tsx');
  // Find first '{' after 'en:'
  let start = tsx.indexOf('{', idx);
  if (start === -1) throw new Error('Could not find opening { for en block');
  let i = start;
  let braceDepth = 0;
  let inSingle = false;
  let inDouble = false;
  let escape = false;
  for (; i < tsx.length; i++) {
    const ch = tsx[i];
    if (escape) { escape = false; continue; }
    if (ch === '\\') { escape = true; continue; }
    if (!inDouble && ch === '\'') { inSingle = !inSingle; continue; }
    if (!inSingle && ch === '"') { inDouble = !inDouble; continue; }
    if (inSingle || inDouble) continue;
    if (ch === '{') { braceDepth++; if (braceDepth === 1) { start = i; } continue; }
    if (ch === '}') { braceDepth--; if (braceDepth === 0) { const end = i; return tsx.slice(start, end + 1); } continue; }
  }
  throw new Error('Could not find closing } for en block');
}

function parsePairsFromTsObjectBlock(block) {
  // Extract pairs of the form 'key': 'value', tolerant of escaped quotes
  const map = {};
  const re = /'([^'\\]*(?:\\.[^'\\]*)*)'\s*:\s*'([^'\\]*(?:\\.[^'\\]*)*)'/g;
  let m;
  while ((m = re.exec(block)) !== null) {
    const rawKey = m[1];
    const rawVal = m[2];
    const key = rawKey.replace(/\\'/g, '\'');
    const val = rawVal.replace(/\\'/g, '\'').replace(/\\n/g, '\n');
    map[key] = val;
  }
  if (Object.keys(map).length === 0) {
    throw new Error('No key/value pairs parsed from en block. The parser may need adjustment.');
  }
  return map;
}

function loadJson(file) {
  try {
    const s = fs.readFileSync(file, 'utf8');
    return JSON.parse(s);
  } catch (e) {
    return {};
  }
}

function saveJson(file, obj) {
  const sorted = Object.keys(obj).sort().reduce((acc, k) => { acc[k] = obj[k]; return acc; }, {});
  const content = JSON.stringify(sorted, null, 2) + '\n';
  fs.writeFileSync(file, content, 'utf8');
}

function main() {
  console.log('[i18n-sync] Reading English catalog from', i18nIndexPath);
  const tsx = readFileSafe(i18nIndexPath);
  const enBlock = extractEnBlock(tsx);
  const enMap = parsePairsFromTsObjectBlock(enBlock);
  const enKeys = Object.keys(enMap);
  console.log('[i18n-sync] English keys:', enKeys.length);

  if (!fs.existsSync(publicI18nDir)) {
    throw new Error('Missing public/i18n directory: ' + publicI18nDir);
  }

  locales.forEach((loc) => {
    const file = path.join(publicI18nDir, `${loc}.json`);
    const data = loadJson(file);
    let added = 0;
    enKeys.forEach((k) => {
      if (data[k] === undefined) { data[k] = enMap[k]; added++; }
    });
    saveJson(file, data);
    console.log(`[i18n-sync] ${loc}: ensured ${enKeys.length} keys (added ${added}) -> ${file}`);
  });
}

try { main(); } catch (e) { console.error('[i18n-sync] ERROR:', e.message); process.exit(1); }


