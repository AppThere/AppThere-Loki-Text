import i18next from 'i18next';
import { initReactI18next } from 'react-i18next';
import HttpBackend from 'i18next-http-backend';
import { invoke } from '@tauri-apps/api/core';

const SUPPORTED_LANGUAGES: readonly string[] = [
  'bg', 'cs', 'da', 'de', 'el', 'en', 'es', 'et', 'fi', 'fr', 'ga',
  'hr', 'hu', 'it', 'lt', 'lv', 'mt', 'nl', 'pl', 'pt', 'ro', 'sk',
  'sl', 'sv', 'ar', 'hi', 'ja', 'ko', 'zh-Hans',
];

/**
 * Normalise a BCP 47 locale tag to the closest supported language code.
 * - 'zh-Hans-CN' → 'zh-Hans'  (preserve script subtag for Chinese)
 * - 'en-GB'      → 'en'        (strip region subtag)
 */
function normalizeLocale(locale: string): string {
  if (locale.startsWith('zh-Hans')) {
    return 'zh-Hans';
  }
  return locale.split('-')[0];
}

function resolveLanguage(raw: string): string {
  const normalized = normalizeLocale(raw);
  return SUPPORTED_LANGUAGES.includes(normalized) ? normalized : 'en';
}

export const i18n = i18next;

export async function initI18n(): Promise<void> {
  const stored = localStorage.getItem('loki.language');

  let lang: string;
  if (stored !== null && SUPPORTED_LANGUAGES.includes(stored)) {
    lang = stored;
  } else {
    try {
      const systemLocale = await invoke<string>('get_system_locale');
      lang = resolveLanguage(systemLocale);
    } catch {
      lang = 'en';
    }
  }

  await i18next
    .use(HttpBackend)
    .use(initReactI18next)
    .init({
      lng: lang,
      fallbackLng: 'en',
      ns: ['common', 'editor'],
      defaultNS: 'common',
      backend: {
        loadPath: '/locales/{{lng}}/{{ns}}.json',
      },
      interpolation: {
        escapeValue: false,
      },
    });
}

export function setLanguage(lang: string): Promise<void> {
  localStorage.setItem('loki.language', lang);
  document.documentElement.dir = lang === 'ar' ? 'rtl' : 'ltr';
  return i18next.changeLanguage(lang).then(() => undefined);
}
