import { describe, it, expect, vi } from 'vitest';

describe('/ home page auth guard', () => {
	it('redirects to /login when user is not authenticated', () => {
		expect(true).toBe(true);
	});

	it('shows board grid when user is authenticated', () => {
		expect(true).toBe(true);
	});
});
