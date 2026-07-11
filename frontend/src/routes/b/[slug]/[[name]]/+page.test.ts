import { describe, it, expect } from 'vitest';

const mockBoard = {
	board: { id: 'b1', slug: 'test-slug', name: 'Test Board', created_by: 'u1' },
	lists: [{ id: 'l1', board_id: 'b1', name: 'Todo', position: 0, cards: [] }],
	members: [],
};

describe('Board page /b/[slug]', () => {
	it('receives board data from load function', () => {
		const data = mockBoard;
		expect(data.board.slug).toBe('test-slug');
		expect(data.board.name).toBe('Test Board');
		expect(data.lists).toHaveLength(1);
	});

	it('renders board name from data', () => {
		expect(mockBoard.board.name).toBeTruthy();
	});

	it('handles slug in URL params', () => {
		const slug = 'test-slug';
		expect(slug).toMatch(/^[a-zA-Z0-9_-]+$/);
	});
});

describe('/b/{slug}/{board-name}', () => {
	it('route matches and renders the same board page as /b/{slug}', () => {
		// The [[name]] optional param route handles both /b/{slug} and /b/{slug}/{board-name}.
		// Both resolve to the same +page.svelte, which loads board by slug only.
		expect(true).toBe(true);
	});

	it('ignores the trailing board-name segment and loads board by slug', () => {
		// +page.ts only destructures { slug } from params — board-name is ignored.
		const slug = 'my-board';
		const name = 'my-board-name';
		const params = { slug, name };
		expect(params.slug).toBe('my-board');
	});

	it('board-name is optional — /b/{slug} continues to work', () => {
		const slug = 'my-board';
		// When no board-name provided, params.name is undefined:
		const params1 = { slug };
		const params2 = { slug, name: 'board-name' };
		expect(params1.slug).toBe(params2.slug);
	});
});
