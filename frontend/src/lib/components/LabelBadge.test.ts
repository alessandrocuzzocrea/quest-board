import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import LabelBadge from './LabelBadge.svelte';

describe('LabelBadge', () => {
	it('renders the label name', () => {
		render(LabelBadge, { name: 'bug', color: '#d04444' });
		expect(screen.getByText('bug')).toBeTruthy();
	});

	it('renders with the given color as background', () => {
		render(LabelBadge, { name: 'feature', color: '#0079bf' });
		const badge = screen.getByText('feature');
		expect(badge.style.backgroundColor).toBe('rgb(0, 121, 191)');
	});

	it('renders with default color when none given', () => {
		render(LabelBadge, { name: 'untagged' });
		const badge = screen.getByText('untagged');
		expect(badge.style.backgroundColor).toBe('rgb(0, 121, 191)');
	});
});
