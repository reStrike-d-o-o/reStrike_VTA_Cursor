import fs from 'fs';
import path from 'path';

const uiRoot = path.resolve(process.cwd());
const srcDir = path.join(uiRoot, 'src');
const publicI18nDir = path.join(uiRoot, 'public', 'i18n');
const enJsonPath = path.join(publicI18nDir, 'en.json');
const i18nIndexPath = path.join(uiRoot, 'src', 'i18n', 'index.tsx');

function walk(dir, exts) {
  const out = [];
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const p = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      // skip obvious non-source dirs
      if (entry.name === 'node_modules' || entry.name === 'public' || entry.name === 'dist' || entry.name === 'build') continue;
      out.push(...walk(p, exts));
    } else {
      const lower = entry.name.toLowerCase();
      if (exts.some((e) => lower.endsWith(e))) out.push(p);
    }
  }
  return out;
}

function extractKeysFromFile(content) {
  const keys = new Map(); // key -> fallback (if any)

  // Pattern 1: t('key', 'fallback') or t("key", "fallback")
  // Capture both key and optional fallback
  const tCallRe = /\bt\(\s*(["'])((?:\\.|(?!\1).)+)\1\s*(?:,\s*(["'])((?:\\.|(?!\3).)+)\3)?/g;
  let m;
  while ((m = tCallRe.exec(content)) !== null) {
    const key = m[2];
    const fallback = m[4];
    if (!keys.has(key)) keys.set(key, fallback || undefined);
  }

  // Pattern 2: <T id="key" default="text" ...>
  const tTagRe = /<T\s+([^>]*?)>/g;
  let tag;
  while ((tag = tTagRe.exec(content)) !== null) {
    const attrs = tag[1];
    const idMatch = attrs.match(/\bid\s*=\s*(["'])((?:\\.|(?!\1).)+)\1/);
    if (idMatch) {
      const key = idMatch[2];
      const defMatch = attrs.match(/\bdefault\s*=\s*(["'])((?:\\.|(?!\1).)+)\1/);
      const fallback = defMatch ? defMatch[2] : undefined;
      if (!keys.has(key)) keys.set(key, fallback);
    }
  }

  return keys;
}

function loadExistingEnMap() {
  // Prefer existing en.json if present
  if (fs.existsSync(enJsonPath)) {
    try {
      const data = JSON.parse(fs.readFileSync(enJsonPath, 'utf8'));
      return data && typeof data === 'object' ? data : {};
    } catch {}
  }
  // Fallback: parse en block from index.tsx
  try {
    const tsx = fs.readFileSync(i18nIndexPath, 'utf8');
    const marker = 'en:';
    const idx = tsx.indexOf(marker);
    if (idx !== -1) {
      let start = tsx.indexOf('{', idx);
      let i = start;
      let depth = 0;
      let inS = false, inD = false, esc = false;
      for (; i < tsx.length; i++) {
        const ch = tsx[i];
        if (esc) { esc = false; continue; }
        if (ch === '\\') { esc = true; continue; }
        if (!inD && ch === '\'') { inS = !inS; continue; }
        if (!inS && ch === '"') { inD = !inD; continue; }
        if (inS || inD) continue;
        if (ch === '{') { depth++; continue; }
        if (ch === '}') { depth--; if (depth === 0) break; }
      }
      const block = tsx.slice(start, i + 1);
      const map = {};
      const re = /'([^'\\]*(?:\\.[^'\\]*)*)'\s*:\s*'([^'\\]*(?:\\.[^'\\]*)*)'/g;
      let m;
      while ((m = re.exec(block)) !== null) {
        map[m[1].replace(/\\'/g, '\'')] = m[2].replace(/\\'/g, '\'').replace(/\\n/g, '\n');
      }
      return map;
    }
  } catch {}
  return {};
}

function saveEnJson(obj) {
  if (!fs.existsSync(publicI18nDir)) fs.mkdirSync(publicI18nDir, { recursive: true });
  const sorted = Object.keys(obj).sort().reduce((acc, k) => { acc[k] = obj[k]; return acc; }, {});
  fs.writeFileSync(enJsonPath, JSON.stringify(sorted, null, 2) + '\n', 'utf8');
}

function main() {
  const files = walk(srcDir, ['.ts', '.tsx', '.js', '.jsx']);
  console.log(`[i18n-extract] Scanning ${files.length} files under src/`);

  const merged = new Map();
  for (const f of files) {
    const content = fs.readFileSync(f, 'utf8');
    const keys = extractKeysFromFile(content);
    keys.forEach((fallback, key) => {
      if (!merged.has(key)) merged.set(key, fallback);
    });
  }
  console.log(`[i18n-extract] Collected ${merged.size} unique keys`);

  const enMap = loadExistingEnMap();
  let added = 0;
  merged.forEach((fallback, key) => {
    if (enMap[key] === undefined) {
      enMap[key] = fallback !== undefined ? fallback : key;
      added++;
    }
  });
  console.log(`[i18n-extract] Added ${added} keys to English catalog`);

  saveEnJson(enMap);
  console.log(`[i18n-extract] Wrote ${Object.keys(enMap).length} keys -> ${path.relative(uiRoot, enJsonPath)}`);
}

try { main(); } catch (e) { console.error('[i18n-extract] ERROR:', e); process.exit(1); }


