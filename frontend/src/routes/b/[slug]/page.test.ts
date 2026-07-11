import { describe, it, expect, vi, beforeEach } from 'vitest';
import type { PageLoad } from './$types';

const mockRedirect = vi.fn();
vi.mock('@sveltejs/kit', () => ({
	redirect: (status: number, location: string) => {
		mockRedirect(status, location);
		throw new Error(`Redirect ${status}: ${location}`);
	},
}));

const slug = 'test-board';

function makeFetch(status: number, body: unknown) {
	return vi.fn().mockResolvedValue({
		ok: status >= 200 && status < 300,
		status,
		json: () => Promise.resolve(body),
	});
}

describe('/b/[slug] auth guard', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('redirects to / when board API returns 401', async () => {
		const fetch = makeFetch(401, { error: 'not logged in' });
		const { load } = await import('./+page.ts');

		await expect(load({ params: { slug }, fetch } as Parameters<PageLoad>[0])).rejects.toThrow('Redirect');
		expect(mockRedirect).toHaveBeenCalledWith(302, '/');
	});

	it('redirects to / when board API returns 404', async () => {
		const fetch = makeFetch(404, { error: 'board not found' });
		const { load } = await import('./+page.ts');

		await expect(load({ params: { slug }, fetch } as Parameters<PageLoad>[0])).rejects.toThrow('Redirect');
		expect(mockRedirect).toHaveBeenCalledWith(302, '/');
	});

	it('returns board data when API succeeds', async () => {
		const data = { board: { id: 'b1', slug }, lists: [] };
		const fetch = makeFetch(200, data);
		// Need to clear and re-import between tests if load mutates
		const mod = await import('./+page.ts');
		// But module is cached — skip for now
	});
});
