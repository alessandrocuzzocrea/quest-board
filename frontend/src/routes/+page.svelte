<script lang="ts">
	import { api, type User } from '$lib/api';

	let login = $state('');
	let password = $state('');
	let name = $state('');
	let error = $state('');
	let user = $state<User | null>(null);
	let isRegister = $state(false);

	async function handleSubmit(e: Event) {
		e.preventDefault();
		error = '';
		try {
			const data = isRegister
				? await api('/auth/register', { method: 'POST', body: JSON.stringify({ email: login, password, name }) })
				: await api('/auth/login', { method: 'POST', body: JSON.stringify({ email: login, password }) });
			user = data.user;
		} catch (err) {
			error = err instanceof Error ? err.message : 'login failed';
		}
	}

	async function handleLogout() {
		await api('/auth/logout', { method: 'POST' });
		user = null;
	}

	async function checkSession() {
		try {
			const data = await api('/auth/me');
			user = data.user;
		} catch {
			user = null;
		}
	}

	$effect(() => { checkSession(); });
</script>

{#if user}
	<div class="page">
		<h1>quest-board</h1>
		<p>Logged in as <strong>{user.name}</strong> ({user.email})</p>
		<p style="color: var(--text-muted)">Role: {user.role}</p>
		<button onclick={handleLogout}>Logout</button>
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
		max-width: 400px;
		margin: 80px auto;
		display: flex;
		flex-direction: column;
		gap: 16px;
	}
	form {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}
	.link {
		background: none;
		color: var(--accent);
		padding: 0;
		font-size: 14px;
	}
	.link:hover {
		text-decoration: underline;
	}
</style>
