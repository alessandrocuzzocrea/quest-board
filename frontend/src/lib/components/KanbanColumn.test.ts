import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import KanbanColumn from './KanbanColumn.svelte';

const mockCard = (id: string, name: string) => ({
	id, board_id: '', list_id: '', position: 0, name,
	description: null, due_date: null, is_due_completed: false, is_closed: false,
	created_by: '', members: [], labels: [], comments_count: 0n,
	checklists: [], created_at: '', updated_at: '',
});

describe('KanbanColumn', () => {
	it('renders column title and card count', () => {
		render(KanbanColumn, { title: 'To Do', listId: 'todo', cards: [] });
		expect(screen.getByText('To Do')).toBeTruthy();
		expect(screen.getByText('0')).toBeTruthy();
	});

	it('renders cards', () => {
		const cards = [mockCard('1', 'Task A'), mockCard('2', 'Task B')];
		render(KanbanColumn, { title: 'Done', listId: 'done', cards, cardCount: 2 });
		expect(screen.getByText('Task A')).toBeTruthy();
		expect(screen.getByText('Task B')).toBeTruthy();
		expect(screen.getByText('2')).toBeTruthy();
	});

	it('shows empty state when no cards', () => {
		render(KanbanColumn, { title: 'Empty', listId: 'empty', cards: [] });
		expect(screen.getByText('Drop cards here')).toBeTruthy();
	});

	it('does not show empty state when cards exist', () => {
		const cards = [mockCard('1', 'Only card')];
		render(KanbanColumn, { title: 'List', listId: 'l1', cards });
		expect(screen.queryByText('Drop cards here')).toBeNull();
	});
});
