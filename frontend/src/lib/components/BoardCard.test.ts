import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import BoardCard from './BoardCard.svelte';

describe('BoardCard', () => {
	it('renders the board name', () => {
		render(BoardCard, { name: 'Sprint 42' });
		expect(screen.getByText('Sprint 42')).toBeTruthy();
	});

	it('renders card count', () => {
		render(BoardCard, { name: 'Board', cardCount: 5 });
		expect(screen.getByText('5 cards')).toBeTruthy();
	});

	it('renders "0 cards" when count is zero', () => {
		render(BoardCard, { name: 'Empty' });
		expect(screen.getByText('0 cards')).toBeTruthy();
	});

	it('uses default name when none given', () => {
		render(BoardCard, {});
		expect(screen.getByText('Untitled')).toBeTruthy();
	});
});
