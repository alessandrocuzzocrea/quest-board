import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import ChatPanel from './ChatPanel.svelte';

describe('ChatPanel', () => {
	it('renders nothing when closed', () => {
		const { container } = render(ChatPanel, { open: false, boardId: 'b1' });
		expect(container.querySelector('.panel')).toBeNull();
	});

	it('renders panel when open', () => {
		const { container } = render(ChatPanel, { open: true, boardId: 'b1' });
		expect(container.querySelector('.panel')).toBeTruthy();
	});

	it('has close button with aria-label', () => {
		render(ChatPanel, { open: true, boardId: 'b1' });
		expect(screen.getByLabelText('Close')).toBeTruthy();
	});

	it('shows chat input', () => {
		render(ChatPanel, { open: true, boardId: 'b1' });
		const textarea = document.querySelector('textarea');
		expect(textarea).toBeTruthy();
	});

	it('shows send button', () => {
		render(ChatPanel, { open: true, boardId: 'b1' });
		expect(screen.getByText('Send')).toBeTruthy();
	});
});
