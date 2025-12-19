import { includeIgnoreFile } from '@eslint/compat';
import js from '@eslint/js';
import prettier from 'eslint-config-prettier';
import svelte from 'eslint-plugin-svelte';
import globals from 'globals';
import { fileURLToPath } from 'node:url';
import ts from 'typescript-eslint';

const gitignorePath = fileURLToPath(new URL('./.gitignore', import.meta.url));

export default [
	// Include gitignore patterns
	includeIgnoreFile(gitignorePath),

	// Base configs
	js.configs.recommended,
	...ts.configs.recommended,
	...svelte.configs['flat/recommended'],
	prettier,
	...svelte.configs['flat/prettier'],

	// Global configuration
	{
		languageOptions: {
			globals: {
				...globals.browser,
				...globals.node
			}
		},
		rules: {
			// typescript-eslint strongly recommend that you do not use the no-undef lint rule on TypeScript projects.
			// see: https://typescript-eslint.io/troubleshooting/faqs/eslint/#i-get-errors-from-the-no-undef-rule-about-global-variables-not-being-defined-even-though-there-are-no-typescript-errors
			'no-undef': 'off',
			'@typescript-eslint/typedef': 'error',
			'svelte/no-navigation-without-resolve': 'off' // very buggy with search params
		}
	},

	// Svelte-specific configuration
	{
		files: ['**/*.svelte'],
		languageOptions: {
			parserOptions: {
				parser: ts.parser
			}
		}
	},

	// Svelte TypeScript files with runes (.svelte.ts)
	{
		files: ['**/*.svelte.ts', '**/*.svelte.js'],
		languageOptions: {
			parserOptions: {
				parser: ts.parser
			}
		}
	},

	// Additional ignore patterns
	{
		ignores: [
			'**/build/**',
			'**/dist/**',
			'**/.svelte-kit/**',
			'**/package/**',
			'**/vite.config.*.timestamp-*',
			'src/lib/bindings.ts'
		]
	}
];
