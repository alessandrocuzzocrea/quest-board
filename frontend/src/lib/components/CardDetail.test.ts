import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/svelte';
import CardDetail from './CardDetail.svelte';

const cardData = {
	id: 'c1', name: 'Test Card', description: 'Hello', board_id: 'b1',
	list_id: '', position: 0, due_date: null, is_due_completed: false,
	is_closed: false, created_by: '', created_at: '', updated_at: '',
	members: [], labels: [], comments_count: 0n, checklists: [],
};

const cardWrapper = {
	card: cardData,
	members: [],
	labels: [],
	comments: [],
	checklists: [],
	actions: [],
};

describe('CardDetail (read-only)', () => {
	it('shows loading when open', () => {
		render(CardDetail, { cardId: 'some-id', open: true });
		expect(screen.getAllByText('Loading...').length).toBeGreaterThanOrEqual(1);
	});

	it('renders nothing when closed', () => {
		const { container } = render(CardDetail, { cardId: '', open: false });
		expect(container.querySelector('.panel')).toBeNull();
	});

	it('has close button', () => {
		render(CardDetail, { cardId: 'x', open: true });
		expect(screen.getByText('✕')).toBeTruthy();
	});
});

describe('CardDetail label loading', () => {
	let fetchCalls: string[] = [];

	beforeEach(() => {
		vi.useFakeTimers();
		fetchCalls = [];
		globalThis.fetch = vi.fn().mockImplementation((url: string) => {
			fetchCalls.push(url);
			if (url.includes('/cards/')) {
				return Promise.resolve({ ok: true, json: () => Promise.resolve(cardWrapper) });
			}
			return Promise.resolve({ ok: true, json: () => Promise.resolve([]) });
		});
	});

	afterEach(() => {
		vi.useRealTimers();
		vi.restoreAllMocks();
	});

	it('loads labels using the card board_id, not undefined', async () => {
		render(CardDetail, { cardId: 'c1', open: true });
		await vi.advanceTimersByTimeAsync(0);

		const labelCall = fetchCalls.find(u => u.includes('/labels/board/'));
		expect(labelCall).toBeTruthy();
		expect(labelCall).not.toContain('undefined');
		expect(labelCall).toContain('b1');
	});
});
