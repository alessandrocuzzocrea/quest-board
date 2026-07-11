import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import MemberAvatar from './MemberAvatar.svelte';

describe('MemberAvatar', () => {
	it('renders initials from name', () => {
		render(MemberAvatar, { name: 'Alice Chen' });
		expect(screen.getByText('AC')).toBeTruthy();
	});

	it('renders single initial for single name', () => {
		render(MemberAvatar, { name: 'Admin' });
		expect(screen.getByText('A')).toBeTruthy();
	});
});
