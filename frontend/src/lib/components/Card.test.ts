import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import Card from './Card.svelte';

const mockLabel = (name: string, color: string) => ({
	id: '1', board_id: '', name, color, position: 0, created_at: '', updated_at: '',
});

const mockMember = (name: string) => ({
	id: '1', email: `${name}@test.com`, name, role: 'user',
});

describe('Card', () => {
	it('renders the card name', () => {
		render(Card, { id: '1', name: 'Setup CI' });
		expect(screen.getByText('Setup CI')).toBeTruthy();
	});

	it('renders labels', () => {
		render(Card, { id: '1', name: 'Task', labels: [mockLabel('bug', '#d04444')] });
		expect(screen.getByText('bug')).toBeTruthy();
	});

	it('renders multiple labels', () => {
		render(Card, {
			id: '1', name: 'Task',
			labels: [mockLabel('bug', '#d04444'), mockLabel('feature', '#0079bf')],
		});
		expect(screen.getByText('bug')).toBeTruthy();
		expect(screen.getByText('feature')).toBeTruthy();
	});

	it('renders description when provided', () => {
		render(Card, { id: '1', name: 'Task', description: 'Fix the login flow' });
		expect(screen.getByText('Fix the login flow')).toBeTruthy();
	});

	it('renders member avatars', () => {
		render(Card, { id: '1', name: 'Task', members: [mockMember('Alice')] });
		expect(screen.getByText('A')).toBeTruthy();
	});

	it('renders comment count', () => {
		render(Card, { id: '1', name: 'Task', commentsCount: 3n });
		expect(screen.getByText('💬 3')).toBeTruthy();
	});

	it('renders due date', () => {
		render(Card, { id: '1', name: 'Task', dueDate: '2026-07-20T00:00:00Z' });
		expect(screen.getByText(/Jul 20/)).toBeTruthy();
	});

	it('renders checklist progress', () => {
		const checklist = {
			id: '1', card_id: '', name: 'Checklist', position: 0,
			hide_completed: false, tasks: [
				{ id: '1', task_list_id: '', name: 'Step 1', position: 0, is_completed: true, assignee_id: null, created_at: '', updated_at: '' },
				{ id: '2', task_list_id: '', name: 'Step 2', position: 1, is_completed: false, assignee_id: null, created_at: '', updated_at: '' },
			], created_at: '', updated_at: '',
		};
		render(Card, { id: '1', name: 'Task', checklists: [checklist] });
		expect(screen.getByText(/1\/2/)).toBeTruthy();
	});

	it('marks overdue due dates', () => {
		render(Card, { id: '1', name: 'Task', dueDate: '2026-07-05T00:00:00Z' });
		expect(screen.getByText(/Jul 5/)).toBeTruthy();
	});

	it('shows completed due date as done', () => {
		render(Card, { id: '1', name: 'Task', dueDate: '2026-07-10T00:00:00Z', isDueCompleted: true });
		expect(screen.getByText(/Jul 10/)).toBeTruthy();
	});

	it('applies closed styling', () => {
		const { container } = render(Card, { id: '1', name: 'Task', isClosed: true });
		const button = container.querySelector('button');
		expect(button?.className).toContain('closed');
	});
});
