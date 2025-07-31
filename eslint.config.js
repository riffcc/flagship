import eslint from '@eslint/js';
import pluginVue from 'eslint-plugin-vue';
import tseslint from 'typescript-eslint';
import globals from 'globals';

export default [
  // --- Global Ignores ---
  {
    ignores: [
      '**/node_modules/**',
      '**/dist/**',
      '**/coverage/**',
      'packages/renderer/public/**',
      'playwright-report/**',
      'test-results/**',
    ],
  },

  // Base configurations
  eslint.configs.recommended,
  ...tseslint.configs.recommended,
  ...pluginVue.configs['flat/recommended'],

  // --- Configuration for Browser/Vue Application Code ---
  {
    files: ['packages/renderer/src/**/*.{js,ts,vue}'],
    languageOptions: {
      globals: {
        ...globals.browser,
      },
      parserOptions: {
        sourceType: 'module',
        parser: tseslint.parser,
      },
    },
    rules: {
      // --- THIS IS THE FIX ---
      // Disable the base rule to prevent conflicts with the @typescript-eslint rule
      'no-unused-vars': 'off',

      // Your correctly configured @typescript-eslint rule
      '@typescript-eslint/no-unused-vars': [
        'error',
        { argsIgnorePattern: '^_', varsIgnorePattern: '^_', caughtErrorsIgnorePattern: '^_' },
      ],

      // Other rules...
      '@typescript-eslint/consistent-type-imports': 'error',
      'semi': ['error', 'always'],
      'comma-dangle': ['warn', 'always-multiline'],
      'quotes': ['warn', 'single', { avoidEscape: true }],
      'vue/html-self-closing': 'off',
      'vue/singleline-html-element-content-newline': 'off',
      'vue/multi-word-component-names': 'off',
    },
  },

  // --- Configuration for Node.js files (Configs, Scripts, etc.) ---
  {
    files: [
      '*.{js,cjs,mjs,ts}',
      'scripts/**/*.{js,mjs,ts}',
      'version/**/*.{js,mjs,ts}',
      'packages/main/src/**/*.{js,ts}',
      'packages/preload/src/**/*.{js,ts}',
      'packages/**/vite.config.{js,ts}',
      'tests/**/*.{js,ts}',
      'playwright.config.ts',
    ],
    languageOptions: {
      globals: {
        ...globals.node,
      },
    },
    rules: {
      // Also disable the base rule here for consistency
      'no-unused-vars': 'off',
      '@typescript-eslint/no-unused-vars': [
        'error',
        { argsIgnorePattern: '^_', varsIgnorePattern: '^_', caughtErrorsIgnorePattern: '^_' },
      ],
      '@typescript-eslint/no-var-requires': 'off',
    },
  },
];
