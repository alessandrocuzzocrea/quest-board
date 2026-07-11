<script lang="ts">
	import { api, type User } from '$lib/api';
	import type { ApiKeyResponse } from '$lib/types/bindings';

	let user = $state<User | null>(null);
	let apiKeys = $state<ApiKeyResponse[]>([]);
	let newKeyName = $state('');
	let newKeyToken = $state('');
	let nameValue = $state('');
	let oldPassword = $state('');
	let newPassword = $state('');
	let error = $state('');
	let success = $state('');

	async function load() {
		try {
			const data = await api<{ user: User }>('/auth/me');
			user = data.user;
			nameValue = user.name;
			apiKeys = await api<ApiKeyResponse[]>('/api-keys');
		} catch {
			user = null;
		}
	}

	async function saveName() {
		error = '';
		success = '';
		try {
			const data = await api<{ user: User }>('/auth/me', {
				method: 'PUT',
				body: JSON.stringify({ name: nameValue }),
			});
			user = data.user;
			success = 'Name updated';
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to update name';
		}
	}

	async function savePassword() {
		error = '';
		success = '';
		if (newPassword.length < 4) {
			error = 'Password must be at least 4 characters';
			return;
		}
		try {
			await api('/auth/me/password', {
				method: 'PUT',
				body: JSON.stringify({ old_password: oldPassword, new_password: newPassword }),
			});
			oldPassword = '';
			newPassword = '';
			success = 'Password changed';
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to change password';
		}
	}

	async function createKey() {
		error = '';
		success = '';
		if (!newKeyName.trim()) return;
		try {
			const data = await api<{ token: string; api_key: ApiKeyResponse }>('/api-keys', {
				method: 'POST',
				body: JSON.stringify({ name: newKeyName.trim() }),
			});
			newKeyToken = data.token;
			newKeyName = '';
			apiKeys = await api<ApiKeyResponse[]>('/api-keys');
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to create key';
		}
	}

	async function deleteKey(id: string) {
		error = '';
		success = '';
		try {
			await api(`/api-keys/${id}`, { method: 'DELETE' });
			apiKeys = await api<ApiKeyResponse[]>('/api-keys');
			success = 'Key revoked';
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to delete key';
		}
	}

	$effect(() => { load(); });
</script>

<svelte:head>
	<title>Settings — Quest Board</title>
</svelte:head>

<h1>Settings</h1>

{#if error}
	<div class="msg error">{error}</div>
{/if}
{#if success}
	<div class="msg success">{success}</div>
{/if}

<section>
	<h2>Profile</h2>
	<form onsubmit={(e) => { e.preventDefault(); saveName(); }}>
		<label>
			Name
			<input bind:value={nameValue} required />
		</label>
		<button type="submit">Save</button>
	</form>
</section>

<section>
	<h2>Change Password</h2>
	<form onsubmit={(e) => { e.preventDefault(); savePassword(); }}>
		<label>
			Current Password
			<input type="password" bind:value={oldPassword} required />
		</label>
		<label>
			New Password
			<input type="password" bind:value={newPassword} required minlength={4} />
		</label>
		<button type="submit">Change Password</button>
	</form>
</section>

<section>
	<h2>API Keys</h2>

	<form onsubmit={(e) => { e.preventDefault(); createKey(); }}>
		<label>
			New key name
			<input bind:value={newKeyName} placeholder="e.g. ci-token" required />
		</label>
		<button type="submit">Create Key</button>
	</form>

	{#if newKeyToken}
		<div class="token-display">
			<strong>Token (shown once):</strong>
			<code>{newKeyToken}</code>
			<button onclick={() => { navigator.clipboard.writeText(newKeyToken); }}>Copy</button>
		</div>
	{/if}

	{#if apiKeys.length > 0}
		<table>
			<thead>
				<tr><th>Name</th><th>Prefix</th><th>Created</th><th></th></tr>
			</thead>
			<tbody>
				{#each apiKeys as key (key.id)}
					<tr>
						<td>{key.name}</td>
						<td><code>{key.prefix}...</code></td>
						<td>{new Date(key.created_at).toLocaleDateString()}</td>
						<td><button class="danger" onclick={() => deleteKey(key.id)}>Revoke</button></td>
					</tr>
				{/each}
			</tbody>
		</table>
	{:else}
		<p class="empty">No API keys yet.</p>
	{/if}
</section>

<style>
	h1 { margin: 20px; font-size: 24px; }
	section { margin: 20px; padding: 16px; border: 1px solid var(--border, #ddd); border-radius: 8px; max-width: 500px; }
	h2 { margin: 0 0 12px; font-size: 18px; }
	label { display: block; margin-bottom: 8px; font-size: 14px; }
	input { display: block; width: 100%; padding: 8px; border: 1px solid var(--border, #ddd); border-radius: 6px; margin-top: 4px; box-sizing: border-box; }
	button { padding: 8px 20px; background: var(--accent, #0079bf); color: white; border: none; border-radius: 6px; cursor: pointer; font-weight: 600; }
	button.danger { background: #d04444; }
	.msg { margin: 20px; padding: 8px 16px; border-radius: 6px; font-size: 14px; }
	.error { background: #fff0f0; color: #cc0000; border: 1px solid #ffcccc; }
	.success { background: #f0fff0; color: #1a8a1a; border: 1px solid #ccffcc; }
	.token-display { margin: 12px 0; padding: 12px; background: #f5f5f5; border-radius: 6px; font-size: 13px; word-break: break-all; }
	table { width: 100%; border-collapse: collapse; font-size: 14px; }
	td, th { padding: 8px; text-align: left; border-bottom: 1px solid #eee; }
	.empty { color: var(--text-muted); font-style: italic; }
</style>
