import { describe, it, expect } from 'vitest';

describe('/admin/monitor page', () => {
	it('route directory exists', () => {
		const { existsSync } = require('node:fs');
		const { resolve } = require('node:path');
		const dir = resolve(__dirname);
		expect(existsSync(dir)).toBe(true);
	});

	it('has a +page.svelte', () => {
		const { existsSync } = require('node:fs');
		const { resolve } = require('node:path');
		const file = resolve(__dirname, '+page.svelte');
		expect(existsSync(file)).toBe(true);
	});

	it('has a +page.ts load function', () => {
		const { existsSync } = require('node:fs');
		const { resolve } = require('node:path');
		const file = resolve(__dirname, '+page.ts');
		expect(existsSync(file)).toBe(true);
	});
});
