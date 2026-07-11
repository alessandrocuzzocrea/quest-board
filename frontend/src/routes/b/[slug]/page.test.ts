import { describe, it, expect, vi, beforeEach } from 'vitest';


const mockRedirect = vi.fn();
vi.mock('@sveltejs/kit', () => ({
	redirect: (status: number, location: string) => {
		mockRedirect(status, location);
		throw new Error(`Redirect ${status}: ${location}`);
	},
}));


function makeFetch(status: number, body: unknown) {
	return vi.fn().mockResolvedValue({
		ok: status >= 200 && status < 300,
		status,
		json: () => Promise.resolve(body),
	});
}

describe('/b/[slug] auth guard', () => {
	const slug = 'test-board';
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('redirects to /login when board API returns 401', async () => {
		const fetch = makeFetch(401, { error: 'not logged in' });
		const { load } = await import('./+page');

		await expect(load({ params: { slug }, fetch } as unknown as Parameters<typeof load>[0])).rejects.toThrow('Redirect');
		expect(mockRedirect).toHaveBeenCalledWith(302, '/login');
	});

	it('redirects to /login when board is not found', async () => {
		const fetch = makeFetch(404, { error: 'board not found' });
		const { load } = await import('./+page');

		await expect(load({ params: { slug }, fetch } as unknown as Parameters<typeof load>[0])).rejects.toThrow('Redirect');
		expect(mockRedirect).toHaveBeenCalledWith(302, '/login');
	});

	it('returns board data when API succeeds', async () => {
		const data = { board: { id: 'b1', slug }, lists: [] };
		const fetch = makeFetch(200, data);
		const mod = await import('./+page');

		const result = await mod.load({ params: { slug }, fetch } as unknown as Parameters<typeof mod.load>[0]);
		expect(result).toEqual(data);
	});
});
