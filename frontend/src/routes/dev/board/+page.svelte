<script lang="ts">
	import BoardCard from '$lib/components/BoardCard.svelte';
	import type { Board, ListWithCards, CardWithMembers } from '$lib/types/bindings';

	function mockBoard(name: string): Board {
		return {
			id: crypto.randomUUID(),
			name,
			slug: name.toLowerCase().replace(/\s+/g, '-'),
			position: 0,
			created_by: crypto.randomUUID(),
			created_at: new Date().toISOString(),
			updated_at: new Date().toISOString(),
		};
	}

	function mockCard(name: string, comments: bigint): CardWithMembers {
		return {
			id: crypto.randomUUID(),
			board_id: '',
			list_id: '',
			position: 0,
			name,
			description: null,
			due_date: null,
			is_due_completed: false,
			is_closed: false,
			created_by: crypto.randomUUID(),
			members: [],
			labels: [],
			comments_count: comments,
			checklists: [],
			created_at: new Date().toISOString(),
			updated_at: new Date().toISOString(),
		};
	}

	function mockList(name: string, cards: CardWithMembers[], color: string | null): ListWithCards {
		return {
			id: crypto.randomUUID(),
			board_id: '',
			name,
			position: 0,
			type: 'active',
			color,
			cards,
			created_at: new Date().toISOString(),
			updated_at: new Date().toISOString(),
		};
	}

	const board = mockBoard('Sprint 42');

	const normalLists: ListWithCards[] = [
		mockList('To Do', [
			mockCard('Setup CI', 0n),
			mockCard('Write docs', 2n),
			mockCard('Refactor auth', 5n),
		], null),
		mockList('In Progress', [
			mockCard('API rate limiting', 1n),
		], '#0079bf'),
		mockList('Done', [
			mockCard('Init repo', 0n),
			mockCard('Design system', 3n),
		], null),
	];

	const emptyLists: ListWithCards[] = [
		mockList('To Do', [], null),
		mockList('In Progress', [], '#0079bf'),
		mockList('Done', [], null),
	];

	const scenarios = ['normal', 'empty', 'error'] as const;
	let scenario: string = $state('normal');
</script>

<svelte:head>
	<title>Dev — Board Components</title>
</svelte:head>

<div class="dev-header">
	<h1>Dev: Board Components</h1>
	<div class="controls">
		{#each scenarios as s}
			<button
				class={scenario === s ? 'active' : ''}
				onclick={() => scenario = s}
			>{s}</button>
		{/each}
	</div>
	<p class="note">Types auto-generated from Rust via ts-rs</p>
</div>

<div class="board-scene">
	{#if scenario === 'normal'}
		<h2>{board.name}</h2>
		<div class="board">
			{#each normalLists as list}
				<div class="list">
					<h3>{list.name ?? 'Untitled'}</h3>
					{#each list.cards as card}
						<BoardCard name={card.name} cardCount={Number(card.comments_count)} />
					{/each}
					{#if list.cards.length === 0}
						<p class="empty">No cards</p>
					{/if}
				</div>
			{/each}
		</div>
	{:else if scenario === 'empty'}
		<h2>Empty Board</h2>
		<div class="board">
			{#each emptyLists as list}
				<div class="list">
					<h3>{list.name ?? 'Untitled'}</h3>
					<p class="empty">No cards</p>
				</div>
			{/each}
		</div>
	{:else if scenario === 'error'}
		<h2>Error State</h2>
		<div class="error-state">
			<p>Failed to load board: Network error</p>
			<button>Retry</button>
		</div>
	{/if}
</div>

<style>
	.dev-header {
		padding: 16px;
		border-bottom: 1px solid var(--border, #ddd);
		margin-bottom: 16px;
	}
	.controls {
		display: flex;
		gap: 8px;
		margin: 8px 0;
	}
	.controls button {
		padding: 6px 16px;
		border: 1px solid var(--border, #ddd);
		border-radius: 6px;
		background: var(--bg);
		cursor: pointer;
	}
	.controls button.active {
		background: var(--accent, #0079bf);
		color: white;
		border-color: var(--accent, #0079bf);
	}
	.note {
		font-size: 13px;
		color: var(--text-muted);
	}
	.board-scene {
		padding: 0 16px;
	}
	.board {
		display: flex;
		gap: 12px;
		overflow-x: auto;
		padding: 8px 0;
	}
	.list {
		background: var(--surface, #f5f5f5);
		border-radius: 8px;
		padding: 12px;
		min-width: 260px;
	}
	.empty {
		color: var(--text-muted, #888);
		font-style: italic;
		font-size: 13px;
		text-align: center;
		padding: 16px;
	}
	.error-state {
		background: #fff0f0;
		border: 1px solid #ffcccc;
		border-radius: 8px;
		padding: 24px;
		text-align: center;
		color: #cc0000;
	}
	.error-state button {
		margin-top: 12px;
		padding: 8px 24px;
		background: #cc0000;
		color: white;
		border: none;
		border-radius: 6px;
		cursor: pointer;
	}
</style>
