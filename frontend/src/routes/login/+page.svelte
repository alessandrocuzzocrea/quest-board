<script lang="ts">
	import { goto } from '$app/navigation';
	import { api, type User } from '$lib/api';

	let email = $state('');
	let password = $state('');
	let name = $state('');
	let error = $state('');
	let isRegister = $state(false);

	async function handleSubmit(e: Event) {
		e.preventDefault();
		error = '';
		try {
			if (isRegister) {
				await api<{ user: User }>('/auth/register', { method: 'POST', body: JSON.stringify({ email, password, name }) });
			} else {
				await api<{ user: User }>('/auth/login', { method: 'POST', body: JSON.stringify({ email, password }) });
			}
			goto('/');
		} catch (err) {
			error = err instanceof Error ? err.message : 'login failed';
		}
	}

	async function handleDemo() {
		email = 'demo@test.com';
		password = 'pass';
		await handleSubmit(new Event('submit'));
	}
</script>

<div class="page">
	<h1>{isRegister ? 'Sign up' : 'Sign in'}</h1>
	<form onsubmit={handleSubmit}>
		<input bind:value={email} placeholder="Email or username" type="text" required />
		<input bind:value={password} placeholder="Password" type="password" required />
		{#if isRegister}
			<input bind:value={name} placeholder="Name" required />
		{/if}
		<button type="submit">{isRegister ? 'Register' : 'Login'}</button>
	</form>
	{#if error}
		<p class="error">{error}</p>
	{/if}
	<button class="link" onclick={() => { isRegister = !isRegister; error = ''; }}>
		{isRegister ? 'Already have an account? Sign in' : "Don't have an account? Register"}
	</button>
</div>

<style>
	.page { max-width: 400px; margin: 80px auto; padding: 40px 20px; display: flex; flex-direction: column; gap: 16px; }
	h1 { margin: 0; font-size: 24px; }
	form { display: flex; flex-direction: column; gap: 12px; }
	input { padding: 10px 12px; border: 1px solid var(--border, #ddd); border-radius: 6px; font-size: 14px; }
	button[type=submit] { padding: 10px; background: var(--accent, #0079bf); color: white; border: none; border-radius: 6px; font-size: 14px; cursor: pointer; font-weight: 600; }
	.error { color: var(--danger, #d04444); font-size: 13px; margin: 0; }
	.link { background: none; color: var(--accent); padding: 0; font-size: 14px; border: none; cursor: pointer; }
	.link:hover { text-decoration: underline; }
</style>
