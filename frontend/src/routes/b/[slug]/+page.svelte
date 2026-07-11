<script lang="ts">
	import KanbanColumn from '$lib/components/KanbanColumn.svelte';
	import CardDetail from '$lib/components/CardDetail.svelte';
	import { api, type User } from '$lib/api';
	import type { Board, ListWithCards, CardWithMembers } from '$lib/types/bindings';

	let { data: initial }: { data: { board: Board; lists: ListWithCards[]; members: import('$lib/types/bindings').UserResponse[] } } = $props();

	const boardData = JSON.parse(JSON.stringify(initial.lists)) as ListWithCards[];
	let columns = $state<ListWithCards[]>(boardData);
	let selectedCardId = $state<string | null>(null);
	let user = $state<User | null>(null);
	let error = $state('');
	let newListName = $state('');

	async function checkSession() {
		try {
			const res = await api<{ user: User }>('/auth/me');
			user = res.user;
		} catch {
			user = null;
		}
	}

	async function moveCard(cardId: string, sourceListId: string, targetListId: string) {
		const src = columns.find(c => c.id === sourceListId);
		const tgt = columns.find(c => c.id === targetListId);
		if (!src || !tgt) return;

		const idx = src.cards.findIndex(c => c.id === cardId);
		if (idx === -1) return;

		const [card] = src.cards.splice(idx, 1);
		card.list_id = targetListId;
		card.position = tgt.cards.length > 0 ? tgt.cards[tgt.cards.length - 1].position + 65536 : 65536;
		tgt.cards.push(card);
		columns = columns;

		try {
			await api(`/cards/${cardId}/move`, {
				method: 'PUT',
				body: JSON.stringify({ list_id: targetListId, position: card.position }),
			});
		} catch (e) {
			const reverted = tgt.cards.pop();
			if (reverted) src.cards.splice(idx, 0, reverted);
			columns = columns;
			error = e instanceof Error ? e.message : 'Failed to move card';
		}
	}
	async function deleteCard(cardId: string) {
		try {
			await api(`/cards/${cardId}`, { method: 'DELETE' });
			for (const col of columns) {
				col.cards = col.cards.filter(c => c.id !== cardId);
			}
			columns = columns;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to delete card';
		}
	}

	async function deleteList(listId: string) {
		try {
			await api(`/lists/${listId}`, { method: 'DELETE' });
			columns = columns.filter(c => c.id !== listId);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to delete list';
		}
	}

	async function addCard(listId: string, name: string) {
		try {
			const card = await api<CardWithMembers>('/cards', {
				method: 'POST',
				body: JSON.stringify({ list_id: listId, name }),
			});
			const col = columns.find(c => c.id === listId);
			if (col) {
				col.cards.push(card);
				columns = columns;
			}
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to create card';
		}
	}

	async function addList() {
		const name = newListName.trim();
		if (!name) return;
		try {
			const list = await api<ListWithCards>('/lists', {
				method: 'POST',
				body: JSON.stringify({ board_id: initial.board.id, name }),
			});
			list.cards = [];
			columns = [...columns, list];
			newListName = '';
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to create list';
		}
	}

	$effect(() => { checkSession(); });

{#if error}
	<div class="error-bar">{error}</div>
{/if}

<div class="board">
	{#each columns as col (col.id)}
		<KanbanColumn
			title={col.name ?? 'Untitled'}
			listId={col.id}
			cards={col.cards}
			color={col.color}
			cardCount={col.cards.length}
			onDropCard={moveCard}
			onCardClick={(id) => selectedCardId = id}
			onAddCard={addCard}
		/>
	{/each}

	<div class="add-list-col">
		<input
			class="add-list-input"
			type="text"
			placeholder="+ Add list"
			bind:value={newListName}
			onkeydown={(e) => { if (e.key === 'Enter') addList(); }}
		/>
	</div>
</div>

<CardDetail
	cardId={selectedCardId ?? ''}
	open={!!selectedCardId}
	onclose={() => selectedCardId = null}
/>

<style>
	.board-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 12px 20px;
		border-bottom: 1px solid var(--border, #ddd);
	}
	.board-title {
		margin: 0;
		font-size: 20px;
		font-weight: 700;
	}
	.board-actions {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.user-badge {
		font-size: 13px;
		color: var(--text-muted, #666);
	}
	.error-bar {
		background: #fff0f0;
		border-bottom: 1px solid #ffcccc;
		padding: 8px 20px;
		color: #cc0000;
		font-size: 13px;
	}
	.board {
		display: flex;
		gap: 12px;
		padding: 16px 20px;
		overflow-x: auto;
		flex: 1;
		align-items: flex-start;
	}
	.add-list-col {
		background: rgba(0,0,0,0.08);
		border-radius: 10px;
		padding: 10px;
		width: 280px;
		min-width: 280px;
		flex-shrink: 0;
	}
	.add-list-input {
		width: 100%;
		border: none;
		border-radius: 6px;
		padding: 8px 10px;
		font-size: 14px;
		background: transparent;
		color: white;
		outline: none;
		box-sizing: border-box;
	}
	.add-list-input:focus {
		background: rgba(255,255,255,0.2);
	}
	.add-list-input::placeholder {
		color: rgba(255,255,255,0.7);
	}
</style>
