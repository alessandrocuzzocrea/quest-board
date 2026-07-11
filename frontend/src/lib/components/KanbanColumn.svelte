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
		onCardClick = undefined as ((cardId: string) => void) | undefined,
		onDeleteCard = undefined as ((cardId: string) => void) | undefined,
		onDeleteList = undefined as ((listId: string) => void) | undefined,
	}: {
		title: string;
		cards: CardWithMembers[];
		listId: string;
		cardCount?: number;
		color?: string | null;
		onDropCard?: (cardId: string, sourceListId: string, targetListId: string) => void;
		onCardClick?: (cardId: string) => void;
		onDeleteCard?: (cardId: string) => void;
		onDeleteList?: (listId: string) => void;
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
		<button class="col-delete-btn" title="Delete list" onclick={() => onDeleteList?.(listId)}>✕</button>
	</header>

	<div class="card-list">
		{#each cards as card (card.id)}
			<div class="card-wrapper">
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
					onclick={() => onCardClick?.(card.id)}
				/>
				<button class="card-delete-btn" title="Delete card" onclick={() => onDeleteCard?.(card.id)}>✕</button>
			</div>
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
	.col-delete-btn, .card-delete-btn {
		background: none;
		border: none;
		cursor: pointer;
		color: #bbb;
		font-size: 12px;
		padding: 2px 4px;
		border-radius: 3px;
		opacity: 0;
		transition: opacity 0.12s, color 0.12s;
		flex-shrink: 0;
	}
	.column:hover .col-delete-btn,
	.card-wrapper:hover .card-delete-btn {
		opacity: 1;
	}
	.col-delete-btn:hover, .card-delete-btn:hover {
		color: #d04444;
	}
	.card-wrapper {
		display: flex;
		align-items: flex-start;
		gap: 4px;
	}
	.card-wrapper > :first-child {
		flex: 1;
	}
	.card-delete-btn {
		margin-top: 6px;
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
