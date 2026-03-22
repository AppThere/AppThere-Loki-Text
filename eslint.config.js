import i18nextPlugin from 'eslint-plugin-i18next';

export default [
  {
    plugins: {
      i18next: i18nextPlugin,
    },
    rules: {
      'i18next/no-literal-string': [
        'error',
        {
          mode: 'jsx-only',
          ignore: [
            /^\d+$/,           // pure integers
            /^[A-Z][A-Z0-9_]+$/, // SCREAMING_SNAKE_CASE constants
            /^\s*$/,           // whitespace-only strings
            /^.$/,             // single characters
            /^\d+\.\d+/,       // version strings
          ],
        },
      ],
    },
  },
];
