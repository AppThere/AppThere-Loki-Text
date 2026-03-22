import type commonEn from '../../public/locales/en/common.json';
import type editorEn from '../../public/locales/en/editor.json';

declare module 'i18next' {
  interface CustomTypeOptions {
    defaultNS: 'common';
    resources: {
      common: typeof commonEn;
      editor: typeof editorEn;
    };
  }
}
