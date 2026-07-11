<script lang="ts">
	import LabelBadge from './LabelBadge.svelte';
	import MemberAvatar from './MemberAvatar.svelte';
	import type { Label, UserResponse, TaskListWithTasks } from '$lib/types/bindings';

	let {
		name = '',
		description = null as string | null,
		labels = [] as Label[],
		members = [] as UserResponse[],
		dueDate = null as string | null,
		isDueCompleted = false,
		isClosed = false,
		commentsCount = 0n,
		checklists = [] as TaskListWithTasks[],
		onclick = undefined as (() => void) | undefined,
	}: {
		name: string;
		description?: string | null;
		labels?: Label[];
		members?: UserResponse[];
		dueDate?: string | null;
		isDueCompleted?: boolean;
		isClosed?: boolean;
		commentsCount?: bigint;
		checklists?: TaskListWithTasks[];
		onclick?: () => void;
	} = $props();

	const totalChecklistItems = $derived(
		checklists.reduce((sum, tl) => sum + tl.tasks.length, 0)
	);
	const completedChecklistItems = $derived(
		checklists.reduce((sum, tl) => sum + tl.tasks.filter(t => t.is_completed).length, 0)
	);
	const checklistProgress = $derived(
		totalChecklistItems > 0 ? (completedChecklistItems / totalChecklistItems) * 100 : 0
	);

	const isDueOverdue = $derived(
		!!dueDate && !isDueCompleted && new Date(dueDate) < new Date()
	);

	function formatDate(iso: string): string {
		const d = new Date(iso);
		const months = ['Jan','Feb','Mar','Apr','May','Jun','Jul','Aug','Sep','Oct','Nov','Dec'];
		return `${months[d.getMonth()]} ${d.getDate()}`;
	}
</script>

<button
	class="card"
	class:closed={isClosed}
	class:clickable={!!onclick}
	onclick={onclick}
>
	{#if labels.length > 0}
		<div class="labels">
			{#each labels as label}
				<LabelBadge name={label.name} color={label.color} />
			{/each}
		</div>
	{/if}

	<div class="name">{name}</div>

	{#if description}
		<div class="desc-preview">{description}</div>
	{/if}

	<div class="meta">
		{#if dueDate}
			<span class="due-date" class:overdue={isDueOverdue} class:done={isDueCompleted}>
				{isDueCompleted ? '✓' : '🗓'} {formatDate(dueDate)}
			</span>
		{/if}

		{#if totalChecklistItems > 0}
			<span class="checklist-count">
				✓ {completedChecklistItems}/{totalChecklistItems}
			</span>
		{/if}

		{#if commentsCount > 0n}
			<span class="comment-count">
				💬 {Number(commentsCount)}
			</span>
		{/if}
	</div>

	<div class="footer">
		{#if members.length > 0}
			<div class="members">
				{#each members as member}
					<MemberAvatar name={member.name} size={26} />
				{/each}
			</div>
		{/if}
	</div>
</button>

<style>
	.card {
		background: white;
		border-radius: 8px;
		padding: 10px 12px;
		box-shadow: 0 1px 3px rgba(0,0,0,0.12);
		cursor: default;
		text-align: left;
		border: none;
		width: 100%;
		font-family: inherit;
		font-size: inherit;
		color: inherit;
		transition: box-shadow 0.12s, opacity 0.12s;
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
	.card.clickable:hover {
		box-shadow: 0 2px 6px rgba(0,0,0,0.18);
	}
	.card.closed {
		opacity: 0.55;
	}
	.labels {
		display: flex;
		flex-wrap: wrap;
		gap: 4px;
	}
	.name {
		font-weight: 600;
		font-size: 14px;
		line-height: 1.35;
	}
	.desc-preview {
		font-size: 12px;
		color: #666;
		line-height: 1.3;
		display: -webkit-box;
		-webkit-line-clamp: 2;
		-webkit-box-orient: vertical;
		line-clamp: 2;
		overflow: hidden;
	}
	.meta {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
		font-size: 11px;
		color: #666;
	}
	.due-date.done {
		color: #1a8a1a;
	}
	.due-date.overdue {
		color: #d04444;
		font-weight: 600;
	}
	.footer {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-top: 2px;
	}
	.members {
		display: flex;
		gap: -2px;
	}
	.members > :global(*) {
		margin-right: -4px;
	}
	.members > :global(*:last-child) {
		margin-right: 0;
	}
</style>
