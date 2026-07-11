<script lang="ts">
	import KanbanColumn from '$lib/components/KanbanColumn.svelte';
	import { api, type User } from '$lib/api';
	import type { Board, ListWithCards, CardWithMembers } from '$lib/types/bindings';

	let { data: initial }: { data: { board: Board; lists: ListWithCards[]; members: import('$lib/types/bindings').UserResponse[] } } = $props();

	const boardData = JSON.parse(JSON.stringify(initial.lists)) as ListWithCards[];
	let columns = $state<ListWithCards[]>(boardData);
	let user = $state<User | null>(null);
	let error = $state('');

	async function checkSession() {
		try {
			const res = await api('/auth/me');
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
			// revert on failure
			const reverted = tgt.cards.pop();
			if (reverted) src.cards.splice(idx, 0, reverted);
			columns = columns;
			error = e instanceof Error ? e.message : 'Failed to move card';
		}
	}

	$effect(() => { checkSession(); });
</script>

<svelte:head>
	<title>{initial.board.name} — Quest Board</title>
</svelte:head>

<div class="board-header">
	<h1 class="board-title">{initial.board.name}</h1>
	<div class="board-actions">
		{#if user}
			<span class="user-badge">{user.name}</span>
		{/if}
	</div>
</div>

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
		/>
	{/each}
</div>

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
	}
</style>
