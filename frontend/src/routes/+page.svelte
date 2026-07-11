<script lang="ts">
	import { api, type User } from '$lib/api';
	import BoardCard from '$lib/components/BoardCard.svelte';
	import type { Board } from '$lib/types/bindings';

	let login = $state('');
	let password = $state('');
	let name = $state('');
	let error = $state('');
	let user = $state<User | null>(null);
	let boards = $state<Board[]>([]);
	let isRegister = $state(false);
	let creating = $state(false);
	let newBoardName = $state('');

	async function handleSubmit(e: Event) {
		e.preventDefault();
		error = '';
		try {
			const data = isRegister
				? await api<{ user: User }>('/auth/register', { method: 'POST', body: JSON.stringify({ email: login, password, name }) })
				: await api<{ user: User }>('/auth/login', { method: 'POST', body: JSON.stringify({ email: login, password }) });
			user = data.user;
			await loadBoards();
		} catch (err) {
			error = err instanceof Error ? err.message : 'login failed';
		}
	}

	async function handleLogout() {
		await api('/auth/logout', { method: 'POST' });
		user = null;
		boards = [];
	}

	async function checkSession() {
		try {
			const data = await api<{ user: User }>('/auth/me');
			user = data.user;
			await loadBoards();
		} catch {
			user = null;
		}
	}

	async function loadBoards() {
		try {
			const data = await api<Board[]>('/boards');
			boards = data;
		} catch {
			boards = [];
		}
	}

	async function createBoard() {
		if (!newBoardName.trim()) return;
		creating = true;
		try {
			await api('/boards', { method: 'POST', body: JSON.stringify({ name: newBoardName.trim() }) });
			newBoardName = '';
			await loadBoards();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to create board';
		} finally {
			creating = false;
		}
	}

	$effect(() => { checkSession(); });
</script>

{#if user}
	<div class="page">
		<header class="header">
			<h1>quest-board</h1>
			<div class="user-info">
				<span>{user.name}</span>
				<a href="/settings" class="link">Settings</a>
				<button class="link" onclick={handleLogout}>Logout</button>
			</div>
		</header>

		<div class="boards-section">
			<div class="boards-header">
				<h2>Your Boards</h2>
			</div>

			{#if boards.length > 0}
				<div class="board-grid">
					{#each boards as board (board.id)}
						<a href="/b/{board.slug}/{board.name.toLowerCase().replace(/\s+/g, '-')}" class="board-link">
							<BoardCard name={board.name} />
						</a>
					{/each}
				</div>
			{:else}
				<p class="empty">No boards yet. Create your first one.</p>
			{/if}

			<form class="create-form" onsubmit={(e) => { e.preventDefault(); createBoard(); }}>
				<input
					bind:value={newBoardName}
					placeholder="New board name"
					disabled={creating}
					required
				/>
				<button type="submit" disabled={creating || !newBoardName.trim()}>
					{creating ? 'Creating...' : 'Create Board'}
				</button>
			</form>
		</div>

		{#if error}
			<p class="error-msg">{error}</p>
		{/if}
	</div>
{:else}
	<div class="page">
		<h1>{isRegister ? 'Sign up' : 'Sign in'}</h1>
		<form onsubmit={handleSubmit}>
			<input bind:value={login} placeholder="Email or username" type="text" required />
			<input bind:value={password} placeholder="Password" type="password" required />
			{#if isRegister}
				<input bind:value={name} placeholder="Name" required />
			{/if}
			<button type="submit">{isRegister ? 'Register' : 'Login'}</button>
		</form>
		{#if error}
			<p style="color: var(--danger)">{error}</p>
		{/if}
		<button class="link" onclick={() => { isRegister = !isRegister; error = ''; }}>
			{isRegister ? 'Already have an account? Sign in' : "Don't have an account? Register"}
		</button>
	</div>
{/if}

<style>
	.page {
		max-width: 700px;
		margin: 0 auto;
		padding: 40px 20px;
		display: flex;
		flex-direction: column;
		gap: 24px;
	}
	.header {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}
	.header h1 {
		margin: 0;
		font-size: 24px;
	}
	.user-info {
		display: flex;
		align-items: center;
		gap: 12px;
		font-size: 14px;
		color: var(--text-muted);
	}
	.boards-header h2 {
		margin: 0 0 12px;
		font-size: 18px;
	}
	.board-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
		gap: 12px;
	}
	.board-link {
		text-decoration: none;
		color: inherit;
	}
	.empty {
		color: var(--text-muted);
		font-style: italic;
		padding: 24px 0;
	}
	.create-form {
		display: flex;
		gap: 8px;
	}
	.create-form input {
		flex: 1;
		padding: 8px 12px;
		border: 1px solid var(--border, #ddd);
		border-radius: 6px;
		font-size: 14px;
	}
	.create-form button {
		padding: 8px 20px;
		background: var(--accent, #0079bf);
		color: white;
		border: none;
		border-radius: 6px;
		font-size: 14px;
		cursor: pointer;
		font-weight: 600;
	}
	.create-form button:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}
	.error-msg {
		color: var(--danger, #d04444);
		font-size: 13px;
	}
	form {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}
	input {
		padding: 10px 12px;
		border: 1px solid var(--border, #ddd);
		border-radius: 6px;
		font-size: 14px;
	}
	.link {
		background: none;
		color: var(--accent);
		padding: 0;
		font-size: 14px;
		border: none;
		cursor: pointer;
	}
	.link:hover {
		text-decoration: underline;
	}
</style>
