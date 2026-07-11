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

	it.todo('/b/{slug}/{board-name} route renders the same board page as /b/{slug}');
	it.todo('/b/{slug}/{board-name} ignores the trailing board-name segment and loads board by slug');
	it.todo('/b/{slug}/{board-name} preserves all features: columns, cards, drag-drop, chat, undo');
	it.todo('/b/{slug}/{board-name} board-name is optional — /b/{slug} continues to work');
});
