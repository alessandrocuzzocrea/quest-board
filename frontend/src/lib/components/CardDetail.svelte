<script lang="ts">
	import { api } from '$lib/api';
	import LabelBadge from './LabelBadge.svelte';
	import MemberAvatar from './MemberAvatar.svelte';
	import type { CardWithMembers, CommentWithUser, Action } from '$lib/types/bindings';

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

	function close() {
		card = null;
		comments = [];
		actions = [];
		onclose?.();
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') close();
	}

	function formatDate(iso: string): string {
		const d = new Date(iso);
		return d.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric', hour: '2-digit', minute: '2-digit' });
	}
	function formatAction(act: Action): string {
		switch (act.type) {
			case 'commentCard': return 'commented';
			case 'moveCard': return 'moved card';
			default: return act.type;
		}
	}

	$effect(() => { if (open && cardId) load(); });
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
	<div class="panel" class:open role="dialog" aria-label="Card details">
		<header class="panel-header">
			<h2 class="panel-title">{card?.name ?? 'Loading...'}</h2>
			<button class="close-btn" onclick={close}>✕</button>
		</header>

		<div class="panel-body">
			{#if loading}
				<div class="loading">Loading...</div>
			{:else if card}
				<!-- Labels -->
				{#if card.labels.length > 0}
					<section>
						<h3>Labels</h3>
						<div class="labels">
							{#each card.labels as label}
								<LabelBadge name={label.name} color={label.color} />
							{/each}
						</div>
					</section>
				{/if}

				<!-- Members -->
				{#if card.members.length > 0}
					<section>
						<h3>Members</h3>
						<div class="members">
							{#each card.members as member}
								<div class="member-chip">
									<MemberAvatar name={member.name} size={24} />
									<span>{member.name}</span>
								</div>
							{/each}
						</div>
					</section>
				{/if}

				<!-- Due Date -->
				{#if card.due_date}
					<section>
						<h3>Due Date</h3>
						<p class:overdue={!card.is_due_completed && new Date(card.due_date) < new Date()} class:done={card.is_due_completed}>
							{formatDate(card.due_date)}
							{#if card.is_due_completed}<span class="badge-complete">Completed</span>{/if}
						</p>
					</section>
				{/if}

				<!-- Description -->
				<section>
					<h3>Description</h3>
					<p class="description">{card.description || 'No description.'}</p>
				</section>

				<!-- Checklists -->
				{#if card.checklists.length > 0}
					<section>
						<h3>Checklists</h3>
						{#each card.checklists as tl}
							<div class="checklist">
								<h4>{tl.name}</h4>
								{#each tl.tasks as task}
									<label class="task">
										<input type="checkbox" checked={task.is_completed} disabled />
										<span class:strikethrough={task.is_completed}>{task.name}</span>
									</label>
								{/each}
							</div>
						{/each}
					</section>
				{/if}

				<!-- Comments -->
				<section>
					{#if comments.length === 0}
						<p class="empty">No comments yet.</p>
					{:else}
						<div class="comment-list">
							{#each comments as comment}
								<div class="comment">
									<div class="comment-header">
										<MemberAvatar name={comment.user?.name ?? '?'} size={20} />
										<strong>{comment.user?.name ?? 'Unknown'}</strong>
										<span class="comment-date">{formatDate(comment.created_at)}</span>
									</div>
									<p class="comment-text">{comment.text}</p>
								</div>
							{/each}
						</div>
					{/if}
				</section>

				<!-- Activity -->
				{#if actions.length > 0}
					<section>
						<h3>Activity</h3>
						<ul class="activity-list">
							{#each actions as act}
								<li><span class="action-type">{formatAction(act)}</span> — {formatDate(act.created_at)}</li>
							{/each}
						</ul>
					</section>
				{/if}
			{:else}
				<div class="error">Could not load card.</div>
			{/if}
		</div>
	</div>
{/if}

<style>
	.overlay {
		position: fixed;
		inset: 0;
		background: rgba(0,0,0,0.3);
		z-index: 100;
	}
	.panel {
		position: fixed;
		top: 0;
		right: 0;
		height: 100vh;
		width: 420px;
		max-width: 100vw;
		background: white;
		z-index: 101;
		display: flex;
		flex-direction: column;
		box-shadow: -4px 0 20px rgba(0,0,0,0.15);
		transform: translateX(100%);
		transition: transform 0.2s ease;
	}
	.panel.open {
		transform: translateX(0);
	}
	.panel-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 16px 20px;
		border-bottom: 1px solid #eee;
		flex-shrink: 0;
	}
	.panel-title {
		margin: 0;
		font-size: 18px;
		font-weight: 700;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.close-btn {
		background: none;
		border: none;
		font-size: 20px;
		cursor: pointer;
		color: #888;
		padding: 4px 8px;
		border-radius: 4px;
	}
	.close-btn:hover {
		background: #f0f0f0;
	}
	.panel-body {
		flex: 1;
		overflow-y: auto;
		padding: 16px 20px;
		display: flex;
		flex-direction: column;
		gap: 20px;
	}
	section h3 {
		margin: 0 0 8px;
		font-size: 12px;
		font-weight: 700;
		text-transform: uppercase;
		color: #888;
		letter-spacing: 0.5px;
	}
	.labels {
		display: flex;
		flex-wrap: wrap;
		gap: 4px;
	}
	.members {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
	.member-chip {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 14px;
	}
	.description {
		font-size: 14px;
		line-height: 1.5;
		color: #333;
		margin: 0;
		white-space: pre-wrap;
	}
	.overdue {
		color: #d04444;
		font-weight: 600;
	}
	.done {
		color: #1a8a1a;
	}
	.badge-complete {
		background: #1a8a1a;
		color: white;
		padding: 1px 6px;
		border-radius: 4px;
		font-size: 11px;
		margin-left: 6px;
	}
	.checklist h4 {
		margin: 0 0 4px;
		font-size: 13px;
		color: #555;
	}
	.task {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 13px;
		padding: 2px 0;
		cursor: default;
	}
	.strikethrough {
		text-decoration: line-through;
		color: #999;
	}
	.comment-list {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}
	.comment {
		font-size: 14px;
	}
	.comment-header {
		display: flex;
		align-items: center;
		gap: 6px;
		margin-bottom: 4px;
	}
	.comment-date {
		font-size: 11px;
		color: #999;
		margin-left: auto;
	}
	.comment-text {
		margin: 0;
		line-height: 1.4;
		color: #333;
	}
	.activity-list {
		list-style: none;
		padding: 0;
		margin: 0;
	}
	.activity-list li {
		font-size: 12px;
		color: #888;
		padding: 4px 0;
	}
	.action-type {
		text-transform: capitalize;
	}
	.loading, .error, .empty {
		color: #999;
		font-size: 13px;
		font-style: italic;
	}
	.error {
		color: #d04444;
	}
</style>
