<script lang="ts">
	import Card from './Card.svelte';
	import type { CardWithMembers } from '$lib/types/bindings';

	let {
		title = '',
		cards = [] as CardWithMembers[],
		listId = '',
		cardCount = 0,
		color = null as string | null,
		onDropCard = undefined as ((cardId: string, sourceListId: string, targetListId: string) => void) | undefined,
	}: {
		title: string;
		cards: CardWithMembers[];
		listId: string;
		cardCount?: number;
		color?: string | null;
		onDropCard?: (cardId: string, sourceListId: string, targetListId: string) => void;
	} = $props();

	let dropActive = $state(false);

	function handleDragOver(e: DragEvent) {
		e.preventDefault();
		if (e.dataTransfer) e.dataTransfer.dropEffect = 'move';
	}

	function handleDragEnter(e: DragEvent) {
		e.preventDefault();
		dropActive = true;
	}

	function handleDragLeave(e: DragEvent) {
		const target = e.currentTarget as HTMLElement;
		const related = e.relatedTarget as HTMLElement | null;
		if (target.contains(related)) return;
		dropActive = false;
	}

	function handleDrop(e: DragEvent) {
		e.preventDefault();
		dropActive = false;
		const raw = e.dataTransfer?.getData('text/plain');
		if (!raw) return;
		try {
			const { cardId, sourceListId } = JSON.parse(raw);
			if (sourceListId !== listId) {
				onDropCard?.(cardId, sourceListId, listId);
			}
		} catch { /* ignore */ }
	}
</script>

<div
	class="column"
	class:drop-active={dropActive}
	class:has-color={!!color}
	style={color ? `--col-accent: ${color}` : ''}
	role="region"
	aria-label={title}
	ondragover={handleDragOver}
	ondragenter={handleDragEnter}
	ondragleave={handleDragLeave}
	ondrop={handleDrop}
>
	<header class="col-header">
		<h3 class="col-title">{title}</h3>
		<span class="col-count">{cardCount}</span>
	</header>

	<div class="card-list">
		{#each cards as card (card.id)}
			<Card
				id={card.id}
				listId={listId}
				name={card.name}
				description={card.description}
				labels={card.labels}
				members={card.members}
				dueDate={card.due_date}
				isDueCompleted={card.is_due_completed}
				isClosed={card.is_closed}
				commentsCount={card.comments_count}
				checklists={card.checklists}
			/>
		{/each}

		{#if cards.length === 0}
			<div class="empty-state">Drop cards here</div>
		{/if}
	</div>
</div>

<style>
	.column {
		background: #f0f2f5;
		border-radius: 10px;
		padding: 10px;
		width: 280px;
		min-width: 280px;
		display: flex;
		flex-direction: column;
		gap: 8px;
		max-height: calc(100vh - 140px);
		transition: background 0.15s, box-shadow 0.15s;
	}
	.column.drop-active {
		background: #e8ecf0;
		box-shadow: inset 0 0 0 2px var(--col-accent, #0079bf);
	}
	.col-header {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 2px 4px;
	}
	.col-title {
		margin: 0;
		font-size: 14px;
		font-weight: 700;
		color: #333;
		flex: 1;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.col-count {
		font-size: 12px;
		color: #888;
		background: #e0e2e6;
		padding: 1px 7px;
		border-radius: 8px;
		font-weight: 600;
	}
	.card-list {
		display: flex;
		flex-direction: column;
		gap: 7px;
		overflow-y: auto;
		flex: 1;
		padding: 2px 0;
		min-height: 60px;
	}
	.empty-state {
		border: 2px dashed #ccc;
		border-radius: 8px;
		padding: 32px 16px;
		text-align: center;
		color: #999;
		font-size: 13px;
		font-style: italic;
	}
</style>
