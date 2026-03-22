#!/usr/bin/env node
// Validates that every non-English locale contains the same key structure
// as the English reference files. Exits with code 1 if any keys are missing.

import { readFileSync, readdirSync, statSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const LOCALES_DIR = join(__dirname, '..', 'public', 'locales');
const NAMESPACES = ['common', 'editor'];

/**
 * Collect all dot-separated key paths from a nested object.
 * @param {Record<string, unknown>} obj
 * @param {string} prefix
 * @returns {string[]}
 */
function collectKeys(obj, prefix = '') {
  const keys = [];
  for (const [k, v] of Object.entries(obj)) {
    const path = prefix ? `${prefix}.${k}` : k;
    if (v !== null && typeof v === 'object' && !Array.isArray(v)) {
      keys.push(...collectKeys(/** @type {Record<string, unknown>} */ (v), path));
    } else {
      keys.push(path);
    }
  }
  return keys;
}

/**
 * Read and parse a JSON locale file.
 * @param {string} filePath
 * @returns {Record<string, unknown>}
 */
function readJson(filePath) {
  return JSON.parse(readFileSync(filePath, 'utf-8'));
}

// Build reference key sets from English locale files.
/** @type {Map<string, Set<string>>} */
const referenceKeys = new Map();
for (const ns of NAMESPACES) {
  const refPath = join(LOCALES_DIR, 'en', `${ns}.json`);
  const keys = collectKeys(readJson(refPath));
  referenceKeys.set(ns, new Set(keys));
}

// Get all locale directories except 'en'.
const localeDirs = readdirSync(LOCALES_DIR).filter((entry) => {
  return entry !== 'en' && statSync(join(LOCALES_DIR, entry)).isDirectory();
});

let totalMissing = 0;

for (const lang of localeDirs.sort()) {
  /** @type {string[]} */
  const langMissing = [];

  for (const ns of NAMESPACES) {
    const filePath = join(LOCALES_DIR, lang, `${ns}.json`);
    let data;
    try {
      data = readJson(filePath);
    } catch {
      const refKeys = referenceKeys.get(ns) ?? new Set();
      for (const key of refKeys) {
        langMissing.push(`${ns}:${key}`);
      }
      continue;
    }

    const presentKeys = new Set(collectKeys(data));
    const refKeys = referenceKeys.get(ns) ?? new Set();
    for (const key of refKeys) {
      if (!presentKeys.has(key)) {
        langMissing.push(`${ns}:${key}`);
      }
    }
  }

  if (langMissing.length > 0) {
    console.error(`[${lang}] ${langMissing.length} missing key(s):`);
    for (const key of langMissing) {
      console.error(`  - ${key}`);
    }
    totalMissing += langMissing.length;
  }
}

if (totalMissing === 0) {
  console.log(`All ${localeDirs.length} non-English locales are complete.`);
  process.exit(0);
} else {
  console.error(`\nTotal missing keys across all locales: ${totalMissing}`);
  process.exit(1);
}
