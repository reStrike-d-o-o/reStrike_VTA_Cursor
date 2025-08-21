import fs from 'fs';
import path from 'path';
import https from 'https';

// Configuration
const uiRoot = path.resolve(process.cwd());
const publicI18nDir = path.join(uiRoot, 'public', 'i18n');
const enJsonPath = path.join(publicI18nDir, 'en.json');

// Google Translate API endpoint
const GOOGLE_ENDPOINT = 'translation.googleapis.com';
const GOOGLE_PATH = '/language/translate/v2';

// CLI args parsing (very light)
function parseArgs(argv) {
	const args = {};
	for (const arg of argv.slice(2)) {
		if (arg.startsWith('--')) {
			const [k, v] = arg.replace(/^--/, '').split('=');
			args[k] = v === undefined ? true : v;
		}
	}
	return args;
}

// Locale mapping to Google target codes
// Defaults to the locale itself if not specified
const localeToGoogle = {
	es: 'es', // European Spanish
	it: 'it',
	bs: 'bs', // Bosnian (Latin)
	zh: 'zh-CN', // Simplified Chinese
	ru: 'ru',
	hr: 'hr',
	sr: 'sr-Cyrl', // force Cyrillic for Serbian
	de: 'de',
	fr: 'fr'
};

function listLocales() {
	if (!fs.existsSync(publicI18nDir)) return [];
	return fs.readdirSync(publicI18nDir)
		.filter((f) => f.toLowerCase().endsWith('.json'))
		.map((f) => path.basename(f, '.json'))
		.filter((loc) => loc !== 'en');
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
	const sorted = Object.keys(obj)
		.sort()
		.reduce((acc, k) => {
			acc[k] = obj[k];
			return acc;
		}, {});
	fs.writeFileSync(file, JSON.stringify(sorted, null, 2) + '\n', 'utf8');
}

function htmlUnescape(text) {
	return text
		.replace(/&amp;/g, '&')
		.replace(/&lt;/g, '<')
		.replace(/&gt;/g, '>')
		.replace(/&quot;/g, '"')
		.replace(/&#39;/g, "'");
}

// Protect placeholders so the translator does not alter them
function protectPlaceholders(input) {
	const placeholders = [];
	let protectedText = input;

	// Curly placeholders like {name} or {0}
	protectedText = protectedText.replace(/\{[^}]+\}/g, (m) => {
		const token = `__PH_${placeholders.length}__`;
		placeholders.push({ token, original: m });
		return token;
	});

	// printf-like placeholders: %s, %d, %1$s
	protectedText = protectedText.replace(/%(?:\d+\$)?[sdif]/g, (m) => {
		const token = `__PH_${placeholders.length}__`;
		placeholders.push({ token, original: m });
		return token;
	});

	return { protectedText, placeholders };
}

function restorePlaceholders(input, placeholders) {
	let out = input;
	for (const ph of placeholders) {
		const re = new RegExp(ph.token.replace(/[-/\\^$*+?.()|[\]{}]/g, '\\$&'), 'g');
		out = out.replace(re, ph.original);
	}
	return out;
}

function chunkArray(arr, size) {
	const out = [];
	for (let i = 0; i < arr.length; i += size) out.push(arr.slice(i, i + size));
	return out;
}

function httpsPostJson(host, pathName, queryParams, bodyObj) {
	const query = new URLSearchParams(queryParams).toString();
	const options = {
		host: host,
		path: `${pathName}?${query}`,
		method: 'POST',
		headers: {
			'Content-Type': 'application/json'
		}
	};
	return new Promise((resolve, reject) => {
		const req = https.request(options, (res) => {
			let data = '';
			res.on('data', (chunk) => (data += chunk));
			res.on('end', () => {
				try {
					const json = JSON.parse(data);
					if (res.statusCode && res.statusCode >= 200 && res.statusCode < 300) resolve(json);
					else reject(new Error(json.error?.message || `HTTP ${res.statusCode}`));
				} catch (e) {
					reject(e);
				}
			});
		});
		req.on('error', reject);
		req.write(JSON.stringify(bodyObj));
		req.end();
	});
}

async function translateBatchGoogle(apiKey, texts, target) {
	if (!texts.length) return [];
	const body = {
		q: texts,
		target,
		source: 'en',
		format: 'text'
	};
	const resp = await httpsPostJson(GOOGLE_ENDPOINT, GOOGLE_PATH, { key: apiKey }, body);
	const translations = resp?.data?.translations || [];
	return translations.map((t) => htmlUnescape(t.translatedText || ''));
}

async function translateAllGoogle({ apiKey, pairs, target }) {
	// pairs: Array<{ key, text }>
	const outputs = new Map();
	const BATCH_SIZE = 80; // conservative to avoid payload limits
	const chunks = chunkArray(pairs, BATCH_SIZE);
	for (let i = 0; i < chunks.length; i++) {
		const chunk = chunks[i];
		// Protect placeholders per entry
		const protectedEntries = chunk.map((p) => {
			const { protectedText, placeholders } = protectPlaceholders(p.text);
			return { key: p.key, protectedText, placeholders };
		});
		const toTranslate = protectedEntries.map((e) => e.protectedText);
		let translated;
		try {
			translated = await translateBatchGoogle(apiKey, toTranslate, target);
		} catch (e) {
			throw new Error(`Google translate failed (target=${target}): ${e.message}`);
		}
		translated.forEach((out, idx) => {
			const entry = protectedEntries[idx];
			const restored = restorePlaceholders(out, entry.placeholders);
			outputs.set(entry.key, restored);
		});
		console.log(`[i18n-translate] Batch ${i + 1}/${chunks.length} -> ${translated.length} items`);
	}
	return outputs;
}

async function main() {
	const args = parseArgs(process.argv);
	const apiKey = process.env.GOOGLE_TRANSLATE_API_KEY || args.key || '';
	if (!apiKey) {
		throw new Error('Missing GOOGLE_TRANSLATE_API_KEY. Set env var or pass --key=...');
	}

	if (!fs.existsSync(enJsonPath)) {
		throw new Error(`Missing English catalog: ${enJsonPath}`);
	}
	const enMap = loadJson(enJsonPath);
	const enKeys = Object.keys(enMap);
	console.log(`[i18n-translate] English keys: ${enKeys.length}`);

	let targets;
	if (args.locales) {
		targets = args.locales.split(',').map((s) => s.trim()).filter(Boolean).filter((l) => l !== 'en');
	} else {
		targets = listLocales();
	}
	if (!targets.length) {
		console.log('[i18n-translate] No target locales found. Exiting.');
		return;
	}
	console.log('[i18n-translate] Target locales:', targets.join(', '));

	for (const loc of targets) {
		const file = path.join(publicI18nDir, `${loc}.json`);
		const data = loadJson(file);
		const googleTarget = localeToGoogle[loc] || loc;

		// Build translation worklist: missing, identical to English, or Cyrillic fix for sr
		const work = [];
		for (const k of enKeys) {
			const enVal = enMap[k];
			const cur = data[k];
			const needsCyrillicFix = loc === 'sr' && (args['force-cyrillic-sr'] === '1' || args['force-cyrillic-sr'] === 'true') && typeof cur === 'string' && /[A-Za-z]|\uFFFD/.test(cur);
			if (cur === undefined || cur === enVal || needsCyrillicFix) {
				work.push({ key: k, text: enVal });
			}
		}
		if (!work.length) {
			console.log(`[i18n-translate] ${loc}: nothing to translate (already complete)`);
			saveJson(file, data);
			continue;
		}
		console.log(`[i18n-translate] ${loc}: translating ${work.length} entries -> ${file}`);

		const translatedMap = await translateAllGoogle({ apiKey, pairs: work, target: googleTarget });
		for (const [k, v] of translatedMap.entries()) {
			data[k] = v;
		}
		saveJson(file, data);
		console.log(`[i18n-translate] ${loc}: wrote ${Object.keys(data).length} keys`);
	}
	console.log('[i18n-translate] Done. You can now run: npm run i18n:report');
}

try {
	await main();
} catch (e) {
	console.error('[i18n-translate] ERROR:', e.message || e);
	process.exit(1);
}


