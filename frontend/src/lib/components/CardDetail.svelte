<script lang="ts">
	import { api } from '$lib/api';
	import LabelBadge from './LabelBadge.svelte';
	import MemberAvatar from './MemberAvatar.svelte';
	import type { CardWithMembers, CommentWithUser, Action, Task, TaskListWithTasks } from '$lib/types/bindings';

	let {
		cardId = '',
		open = false,
		onclose = undefined as (() => void) | undefined,
	}: {
		cardId: string;
		open: boolean;
		onclose?: () => void;
	} = $props();

	let card = $state<CardWithMembers | null>(null);
	let comments = $state<CommentWithUser[]>([]);
	let actions = $state<Action[]>([]);
	let loading = $state(true);
	let editingName = $state(false);
	let editName = $state('');
	let editingDesc = $state(false);
	let editDesc = $state('');
	let commentText = $state('');
	let saving = $state(false);

	async function load() {
		if (!cardId || !open) return;
		loading = true;
		try {
			const [cardData, commentsData, actionsData] = await Promise.all([
				api<CardWithMembers>(`/cards/${cardId}`),
				api<CommentWithUser[]>(`/cards/${cardId}/comments`),
				api<Action[]>(`/cards/${cardId}/actions`),
			]);
			card = cardData;
			comments = commentsData;
			actions = actionsData;
		} catch {
			card = null;
		} finally {
			loading = false;
		}
	}

	async function saveName() {
		if (!card || editName.trim() === card.name) { editingName = false; return; }
		saving = true;
		try {
			const updated = await api<CardWithMembers>(`/cards/${cardId}`, {
				method: 'PUT', body: JSON.stringify({ name: editName.trim() }),
			});
			card.name = updated.name;
			editingName = false;
		} catch { /* ignore */ }
		saving = false;
	}

	async function saveDescription() {
		if (!card) { editingDesc = false; return; }
		saving = true;
		try {
			const updated = await api<CardWithMembers>(`/cards/${cardId}`, {
				method: 'PUT', body: JSON.stringify({ description: editDesc }),
			});
			card.description = updated.description;
			editingDesc = false;
		} catch { /* ignore */ }
		saving = false;
	}

	async function toggleTask(task: Task, tl: TaskListWithTasks) {
		try {
			const updated = await api<Task>(`/cards/${cardId}/task-lists/${tl.id}/tasks/${task.id}`, {
				method: 'PUT', body: JSON.stringify({ is_completed: !task.is_completed }),
			});
			task.is_completed = updated.is_completed;
		} catch { /* ignore */ }
	}

	async function saveComment() {
		if (!commentText.trim() || !cardId) return;
		try {
			const newComment = await api<CommentWithUser>('/comments', {
				method: 'POST', body: JSON.stringify({ card_id: cardId, text: commentText.trim() }),
			});
			comments = [...comments, newComment];
			commentText = '';
		} catch { /* ignore */ }
	}

	function startEditName() { if (card) { editName = card.name; editingName = true; } }
	function startEditDesc() { if (card) { editDesc = card.description ?? ''; editingDesc = true; } }

	function close() {
		card = null; comments = []; actions = []; editingName = false; editingDesc = false; commentText = '';
		onclose?.();
	}

	function handleKeydown(e: KeyboardEvent) { if (e.key === 'Escape') close(); }

	function formatDate(iso: string): string {
		const d = new Date(iso);
		return d.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric', hour: '2-digit', minute: '2-digit' });
	}

	$effect(() => { if (open && cardId) load(); });
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
	<div class="overlay" onclick={close} role="presentation"></div>
	<div class="panel" class:open role="dialog" aria-label="Card details">
		<header class="panel-header">
			{#if editingName}
				<input class="name-input" bind:value={editName} onblur={saveName} onkeydown={(e) => { if (e.key === 'Enter') saveName(); if (e.key === 'Escape') editingName = false; }} autofocus disabled={saving} />
			{:else}
				<button class="title-btn" onclick={startEditName}>{card?.name ?? 'Loading...'}</button>
			{/if}
			<button class="close-btn" onclick={close}>✕</button>
		</header>

		<div class="panel-body">
			{#if loading}
				<div class="loading">Loading...</div>
			{:else if card}
				<section>
					<h3>Labels</h3>
					{#if card.labels.length > 0}<div class="labels">{#each card.labels as label}<LabelBadge name={label.name} color={label.color} />{/each}</div>{:else}<p class="empty">No labels.</p>{/if}
				</section>

				<section>
					<h3>Members</h3>
					{#if card.members.length > 0}<div class="members">{#each card.members as member}<div class="member-chip"><MemberAvatar name={member.name} size={24} /><span>{member.name}</span></div>{/each}</div>{:else}<p class="empty">No members.</p>{/if}
				</section>

				{#if card.due_date}<section><h3>Due Date</h3><p class:overdue={!card.is_due_completed && new Date(card.due_date) < new Date()} class:done={card.is_due_completed}>{formatDate(card.due_date)}{#if card.is_due_completed}<span class="badge-complete">Completed</span>{/if}</p></section>{/if}

				<section>
					<h3>Description</h3>
					{#if editingDesc}<textarea class="desc-textarea" bind:value={editDesc} onblur={saveDescription} disabled={saving} rows={4}></textarea>{:else}<button class="desc-btn" onclick={startEditDesc}>{card.description || 'Add a description...'}</button>{/if}
				</section>

				{#if card.checklists.length > 0}<section><h3>Checklists</h3>{#each card.checklists as tl (tl.id)}<div class="checklist"><h4>{tl.name}</h4>{#each tl.tasks as task (task.id)}<label class="task"><input type="checkbox" checked={task.is_completed} onchange={() => toggleTask(task, tl)} /><span class:strikethrough={task.is_completed}>{task.name}</span></label>{/each}</div>{/each}</section>{/if}

				<section>
					<h3>Comments ({comments.length})</h3>
					{#if comments.length > 0}<div class="comment-list">{#each comments as comment}<div class="comment"><div class="comment-header"><MemberAvatar name={comment.user?.name ?? '?'} size={20} /><strong>{comment.user?.name ?? 'Unknown'}</strong><span class="comment-date">{formatDate(comment.created_at)}</span></div><p class="comment-text">{comment.text}</p></div>{/each}</div>{:else}<p class="empty">No comments yet.</p>{/if}
					<div class="comment-form"><input class="comment-input" type="text" placeholder="Write a comment..." bind:value={commentText} onkeydown={(e) => { if (e.key === 'Enter' && commentText.trim()) saveComment(); }} /></div>
				</section>

				{#if actions.length > 0}<section><h3>Activity</h3><ul class="activity-list">{#each actions as act}<li>{act.type} — {formatDate(act.created_at)}</li>{/each}</ul></section>{/if}
			{:else}
				<div class="error">Could not load card.</div>
			{/if}
		</div>
	</div>
{/if}

<style>
	.overlay { position: fixed; inset: 0; background: rgba(0,0,0,0.3); z-index: 100; }
	.panel { position: fixed; top: 0; right: 0; height: 100vh; width: 420px; max-width: 100vw; background: white; z-index: 101; display: flex; flex-direction: column; box-shadow: -4px 0 20px rgba(0,0,0,0.15); transform: translateX(100%); transition: transform 0.2s ease; }
	.panel.open { transform: translateX(0); }
	.panel-header { display: flex; align-items: center; gap: 8px; padding: 16px 20px; border-bottom: 1px solid #eee; flex-shrink: 0; }
	.title-btn { flex: 1; font-size: 18px; font-weight: 700; cursor: pointer; background: none; border: none; text-align: left; padding: 4px 0; border-radius: 4px; font-family: inherit; color: inherit; }
	.title-btn:hover { background: #f0f0f0; }
	.name-input { flex: 1; font-size: 18px; font-weight: 700; padding: 4px 8px; border: 2px solid var(--accent, #0079bf); border-radius: 4px; font-family: inherit; }
	.close-btn { background: none; border: none; font-size: 20px; cursor: pointer; color: #888; padding: 4px 8px; border-radius: 4px; flex-shrink: 0; }
	.close-btn:hover { background: #f0f0f0; }
	.panel-body { flex: 1; overflow-y: auto; padding: 16px 20px; display: flex; flex-direction: column; gap: 20px; }
	section h3 { margin: 0 0 8px; font-size: 12px; font-weight: 700; text-transform: uppercase; color: #888; letter-spacing: 0.5px; }
	.labels { display: flex; flex-wrap: wrap; gap: 4px; }
	.members { display: flex; flex-direction: column; gap: 6px; }
	.member-chip { display: flex; align-items: center; gap: 8px; font-size: 14px; }
	.desc-btn { font-size: 14px; line-height: 1.5; color: #333; cursor: pointer; background: none; border: none; text-align: left; padding: 4px; border-radius: 4px; width: 100%; font-family: inherit; white-space: pre-wrap; }
	.desc-btn:hover { background: #f0f0f0; }
	.desc-textarea { width: 100%; padding: 8px; border: 2px solid var(--accent, #0079bf); border-radius: 4px; font-family: inherit; font-size: 14px; line-height: 1.5; resize: vertical; box-sizing: border-box; }
	.overdue { color: #d04444; font-weight: 600; }
	.done { color: #1a8a1a; }
	.badge-complete { background: #1a8a1a; color: white; padding: 1px 6px; border-radius: 4px; font-size: 11px; margin-left: 6px; }
	.checklist h4 { margin: 0 0 4px; font-size: 13px; color: #555; }
	.task { display: flex; align-items: center; gap: 6px; font-size: 13px; padding: 2px 0; cursor: pointer; }
	.strikethrough { text-decoration: line-through; color: #999; }
	.comment-list { display: flex; flex-direction: column; gap: 12px; }
	.comment { font-size: 14px; }
	.comment-header { display: flex; align-items: center; gap: 6px; margin-bottom: 4px; }
	.comment-date { font-size: 11px; color: #999; margin-left: auto; }
	.comment-text { margin: 0; line-height: 1.4; color: #333; }
	.comment-form { margin-top: 8px; }
	.comment-input { width: 100%; padding: 8px 10px; border: 1px solid #ddd; border-radius: 6px; font-size: 13px; box-sizing: border-box; font-family: inherit; outline: none; }
	.comment-input:focus { border-color: var(--accent, #0079bf); }
	.activity-list { list-style: none; padding: 0; margin: 0; }
	.activity-list li { font-size: 12px; color: #888; padding: 4px 0; }
	.loading, .error, .empty { color: #999; font-size: 13px; font-style: italic; }
	.error { color: #d04444; }
</style>
