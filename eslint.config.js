import svelte from 'eslint-plugin-svelte';
import ts from 'typescript-eslint';
import prettier from 'eslint-config-prettier';
import globals from 'globals';
import svelteParser from 'svelte-eslint-parser';

export default ts.config(
    ...ts.configs.recommended,
    ...svelte.configs['flat/recommended'],
    prettier,
    ...svelte.configs['flat/prettier'],
    {
        languageOptions: {
            globals: {
                ...globals.browser,
                ...globals.node
            }
        }
    },
    {
        files: ['**/*.svelte', '**/*.svelte.ts', '**/*.svelte.js'],
        languageOptions: {
            parser: svelteParser,
            parserOptions: {
                parser: ts.parser
            }
        }
    },
    {
        rules: {
            '@typescript-eslint/no-explicit-any': 'warn',
            '@typescript-eslint/no-unused-vars': [
                'warn',
                { argsIgnorePattern: '^_', varsIgnorePattern: '^_' }
            ],
            'svelte/no-at-html-tags': 'warn',
            'svelte/require-each-key': 'warn'
        }
    },
    {
        ignores: [
            'node_modules/',
            'dist/',
            'build/',
            'src-tauri/',
            '.svelte-kit/',
            'package-lock.json'
        ]
    }
);
