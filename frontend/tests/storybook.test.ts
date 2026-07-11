import { describe, it, expect } from 'vitest';
import { existsSync, readdirSync } from 'node:fs';
import { resolve } from 'node:path';

const root = resolve(__dirname, '..');

describe('Storybook', () => {
	it('main config exists at .storybook/main.ts', () => {
		const path = resolve(root, '.storybook', 'main.ts');
		expect(existsSync(path)).toBe(true);
	});

	it('preview config exists at .storybook/preview.ts', () => {
		const path = resolve(root, '.storybook', 'preview.ts');
		expect(existsSync(path)).toBe(true);
	});
	it('at least one .stories.svelte file exists in src/lib/components', () => {
		const compDir = resolve(root, 'src', 'lib', 'components');
		const stories = readdirSync(compDir).filter((f: string) => f.endsWith('.stories.svelte'));
		expect(stories.length).toBeGreaterThanOrEqual(1);
	});
});
