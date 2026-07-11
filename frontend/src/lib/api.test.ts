import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { api } from './api';

describe('api helper', () => {
	let originalFetch: typeof globalThis.fetch;

	beforeEach(() => {
		originalFetch = globalThis.fetch;
	});

	afterEach(() => {
		globalThis.fetch = originalFetch;
	});

	it('throws readable error on non-JSON response', async () => {
		globalThis.fetch = vi.fn().mockResolvedValue({
			ok: false,
			status: 502,
			json: () => Promise.reject(new SyntaxError('Unexpected end of JSON input')),
		});

		await expect(api('/test')).rejects.toThrow('request failed (502)');
	});

	it('throws server error message on JSON error response', async () => {
		globalThis.fetch = vi.fn().mockResolvedValue({
			ok: false,
			status: 401,
			json: () => Promise.resolve({ error: 'invalid email or password' }),
		});

		await expect(api('/auth/login')).rejects.toThrow('invalid email or password');
	});

	it('returns data on successful response', async () => {
		globalThis.fetch = vi.fn().mockResolvedValue({
			ok: true,
			status: 200,
			json: () => Promise.resolve({ user: { email: 'admin' } }),
		});

		const data = await api<{ user: { email: string } }>('/auth/login');
		expect(data.user.email).toBe('admin');
	});
});
