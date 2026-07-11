import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import CardDetail from './CardDetail.svelte';

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

describe('CardDetail (editing)', () => {
	let originalFetch: typeof globalThis.fetch;

	beforeEach(() => {
		vi.useFakeTimers();
		originalFetch = globalThis.fetch;
		globalThis.fetch = vi.fn().mockResolvedValue({
			ok: true,
			json: () => Promise.resolve({ id: 'c1', name: 'Test Card', description: 'Hello', labels: [], members: [], comments_count: 0n, checklists: [], created_by: '', created_at: '', updated_at: '', board_id: '', list_id: '', position: 0, is_due_completed: false, is_closed: false, due_date: null }),
		});
	});

	afterEach(() => {
		vi.useRealTimers();
		globalThis.fetch = originalFetch;
	});

	it('renders card name after loading', async () => {
		render(CardDetail, { cardId: 'c1', open: true });
		await vi.advanceTimersByTimeAsync(0);
		expect(screen.getByText('Test Card')).toBeTruthy();
	});

	it('renders description', async () => {
		render(CardDetail, { cardId: 'c1', open: true });
		await vi.advanceTimersByTimeAsync(0);
		expect(screen.getByText('Hello')).toBeTruthy();
	});

	it('shows add description placeholder when null', async () => {
		globalThis.fetch = vi.fn().mockResolvedValue({
			ok: true,
			json: () => Promise.resolve({ id: 'c1', name: 'Card', description: null, labels: [], members: [], comments_count: 0n, checklists: [], created_by: '', created_at: '', updated_at: '', board_id: '', list_id: '', position: 0, is_due_completed: false, is_closed: false, due_date: null }),
		});
		render(CardDetail, { cardId: 'c1', open: true });
		await vi.advanceTimersByTimeAsync(0);
		expect(screen.getByText('Add a description...')).toBeTruthy();
	});
});

describe('CardDetail (due date)', () => {
	let originalFetch: typeof globalThis.fetch;

	beforeEach(() => {
		vi.useFakeTimers();
		originalFetch = globalThis.fetch;
	});

	afterEach(() => {
		vi.useRealTimers();
		globalThis.fetch = originalFetch;
	});

	it('shows set date button when no due date', async () => {
		globalThis.fetch = vi.fn().mockResolvedValue({
			ok: true,
			json: () => Promise.resolve({ id: 'c1', name: 'Card', description: null, due_date: null, labels: [], members: [], comments_count: 0n, checklists: [], created_by: '', created_at: '', updated_at: '', board_id: '', list_id: '', position: 0, is_due_completed: false, is_closed: false }),
		});
		render(CardDetail, { cardId: 'c1', open: true });
		await vi.advanceTimersByTimeAsync(0);
		expect(screen.getByText('Set due date...')).toBeTruthy();
	});

	it('shows due date when set', async () => {
		globalThis.fetch = vi.fn().mockResolvedValue({
			ok: true,
			json: () => Promise.resolve({ id: 'c1', name: 'Card', description: null, due_date: '2026-08-15T00:00:00Z', labels: [], members: [], comments_count: 0n, checklists: [], created_by: '', created_at: '', updated_at: '', board_id: '', list_id: '', position: 0, is_due_completed: false, is_closed: false }),
		});
		render(CardDetail, { cardId: 'c1', open: true });
		await vi.advanceTimersByTimeAsync(0);
		expect(screen.getByText(/Aug 15/)).toBeTruthy();
	});

	it('shows date input when clicking set date', async () => {
		globalThis.fetch = vi.fn().mockResolvedValue({
			ok: true,
			json: () => Promise.resolve({ id: 'c1', name: 'Card', description: null, due_date: null, labels: [], members: [], comments_count: 0n, checklists: [], created_by: '', created_at: '', updated_at: '', board_id: '', list_id: '', position: 0, is_due_completed: false, is_closed: false }),
		});
		render(CardDetail, { cardId: 'c1', open: true });
		await vi.advanceTimersByTimeAsync(0);
		const btn = screen.getByText('Set due date...');
		btn.click();
		await vi.advanceTimersByTimeAsync(0);
		expect(screen.getByTitle('Date picker')).toBeTruthy();
	});
});

describe('CardDetail (comments)', () => {
	let originalFetch: typeof globalThis.fetch;

	beforeEach(() => {
		vi.useFakeTimers();
		originalFetch = globalThis.fetch;
	});

	afterEach(() => {
		vi.useRealTimers();
		globalThis.fetch = originalFetch;
	});

	it('shows comment input', async () => {
		render(CardDetail, { cardId: 'c1', open: true });
		await vi.advanceTimersByTimeAsync(0);
		expect(screen.getByPlaceholderText('Write a comment...')).toBeTruthy();
	});
});
	let originalFetch: typeof globalThis.fetch;

	beforeEach(() => {
		vi.useFakeTimers();
		originalFetch = globalThis.fetch;
		globalThis.fetch = vi.fn().mockResolvedValue({
			ok: true,
			json: () => Promise.resolve({ id: 'c1', name: 'Card', description: null, labels: [], members: [], comments_count: 0n, checklists: [], created_by: '', created_at: '', updated_at: '', board_id: '', list_id: '', position: 0, is_due_completed: false, is_closed: false, due_date: null }),
		});
	});

	afterEach(() => {
		vi.useRealTimers();
		globalThis.fetch = originalFetch;
	});

	it('shows set date button when no due date', async () => {
		globalThis.fetch = vi.fn().mockResolvedValue({
			ok: true,
			json: () => Promise.resolve({ id: 'c1', name: 'Card', description: null, due_date: null, labels: [], members: [], comments_count: 0n, checklists: [], created_by: '', created_at: '', updated_at: '', board_id: '', list_id: '', position: 0, is_due_completed: false, is_closed: false }),
		});
		render(CardDetail, { cardId: 'c1', open: true });
		await vi.advanceTimersByTimeAsync(0);
		expect(screen.getByText('Set due date...')).toBeTruthy();
	});

	it('shows due date when set', async () => {
		globalThis.fetch = vi.fn().mockResolvedValue({
			ok: true,
			json: () => Promise.resolve({ id: 'c1', name: 'Card', description: null, due_date: '2026-08-15T00:00:00Z', labels: [], members: [], comments_count: 0n, checklists: [], created_by: '', created_at: '', updated_at: '', board_id: '', list_id: '', position: 0, is_due_completed: false, is_closed: false }),
		});
		render(CardDetail, { cardId: 'c1', open: true });
		await vi.advanceTimersByTimeAsync(0);
		expect(screen.getByText(/Aug 15/)).toBeTruthy();
	});

	it('shows date input when clicking set date', async () => {
		globalThis.fetch = vi.fn().mockResolvedValue({
			ok: true,
			json: () => Promise.resolve({ id: 'c1', name: 'Card', description: null, due_date: null, labels: [], members: [], comments_count: 0n, checklists: [], created_by: '', created_at: '', updated_at: '', board_id: '', list_id: '', position: 0, is_due_completed: false, is_closed: false }),
		});
		render(CardDetail, { cardId: 'c1', open: true });
		await vi.advanceTimersByTimeAsync(0);
		const btn = screen.getByText('Set due date...');
		btn.click();
		await vi.advanceTimersByTimeAsync(0);
		expect(screen.getByTitle('Date picker')).toBeTruthy();
	it('shows comment input', async () => {
		render(CardDetail, { cardId: 'c1', open: true });
		await vi.advanceTimersByTimeAsync(0);
		expect(screen.getByPlaceholderText('Write a comment...')).toBeTruthy();
	});
});

describe('CardDetail (archive)', () => {
	let originalFetch: typeof globalThis.fetch;

	beforeEach(() => {
		vi.useFakeTimers();
		originalFetch = globalThis.fetch;
		globalThis.fetch = vi.fn().mockResolvedValue({
			ok: true,
			json: () => Promise.resolve({ id: 'c1', name: 'Card', description: null, labels: [], members: [], comments_count: 0n, checklists: [], created_by: '', created_at: '', updated_at: '', board_id: '', list_id: '', position: 0, is_due_completed: false, is_closed: false, due_date: null }),
		});
	});

	afterEach(() => {
		vi.useRealTimers();
		globalThis.fetch = originalFetch;
	});

	it('shows archive button', async () => {
		render(CardDetail, { cardId: 'c1', open: true });
		await vi.advanceTimersByTimeAsync(0);
		expect(screen.getByText('Archive')).toBeTruthy();
	});

	it('shows delete button', async () => {
		render(CardDetail, { cardId: 'c1', open: true });
		await vi.advanceTimersByTimeAsync(0);
		expect(screen.getByText('Delete')).toBeTruthy();
	});

	it('shows unarchive when card is closed', async () => {
		globalThis.fetch = vi.fn().mockResolvedValue({
			ok: true,
			json: () => Promise.resolve({ id: 'c1', name: 'Card', description: null, labels: [], members: [], comments_count: 0n, checklists: [], created_by: '', created_at: '', updated_at: '', board_id: '', list_id: '', position: 0, is_due_completed: false, is_closed: true, due_date: null }),
		});
		render(CardDetail, { cardId: 'c1', open: true });
		await vi.advanceTimersByTimeAsync(0);
		expect(screen.getByText('Restore')).toBeTruthy();
	});
});
