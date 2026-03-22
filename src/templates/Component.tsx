/**
 * i18n conventions for Loki components
 * ─────────────────────────────────────
 * • Always use t() for every string that is displayed to the user.
 *   Never hardcode display text — the ESLint rule `i18next/no-literal-string`
 *   enforces this at error level for all JSX expressions.
 *
 * • Default namespace is 'common' (toolbar, menu, dialog, status labels).
 *   Use the 'editor' namespace for canvas and document-specific strings:
 *     const { t } = useTranslation('editor');
 *
 * • Interpolation example:
 *     t('pagination.pageOf', { current: 1, total: 10 })
 *
 * • To change the application language at runtime, call:
 *     import { setLanguage } from '@/i18n';
 *     setLanguage('fr');
 *
 * • To add a new UI string:
 *     1. Add the key to public/locales/en/common.json (or editor.json).
 *     2. Use t('your.key') in the component.
 *     3. Run `node scripts/validate-locales.js` to surface gaps in other locales.
 */

import { useTranslation } from 'react-i18next';

interface Props {}

export function ComponentName({}: Props) {
  const { t } = useTranslation('common');

  return (
    <div>
      {t('dialog.ok')}
    </div>
  );
}
