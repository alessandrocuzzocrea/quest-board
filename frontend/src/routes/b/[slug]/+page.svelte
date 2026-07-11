<script lang="ts">
	import { api, type User } from '$lib/api';
	import { page } from '$app/stores';

	let data = $state<{
		board: { id: string; name: string; slug: string };
		lists: Array<{ id: string; name: string | null; cards: Array<{ id: string; name: string }> }>;
	} | null>(null);
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

	async function loadBoard() {
		const slug = $page.params.slug;
		error = '';
		try {
			const res = await fetch(`/api/v1/boards/by-slug/${slug}`);
			if (!res.ok) throw new Error('Board not found');
			data = await res.json();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load board';
		}
	}

	$effect(() => {
		checkSession();
		loadBoard();
	});
</script>

<h1>{data?.board?.name ?? 'Loading...'}</h1>
{#if error}
	<p style="color: var(--danger)">{error}</p>
{/if}

{#if data}
	<div class="board">
		{#each data.lists as list}
			<div class="list">
				<h3>{list.name ?? 'Untitled'}</h3>
				{#each list.cards as card}
					<div class="card">{card.name}</div>
				{/each}
			</div>
		{/each}
	</div>
{/if}

<style>
	.board {
		display: flex;
		gap: 12px;
		padding: 16px;
		overflow-x: auto;
	}
	.list {
		background: var(--surface);
		border-radius: 8px;
		padding: 12px;
		min-width: 260px;
	}
	.card {
		background: var(--bg);
		border-radius: 4px;
		padding: 8px 12px;
		margin: 8px 0;
		cursor: pointer;
	}
</style>
