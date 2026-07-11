
<script lang="ts">
	const randomUUID = () => crypto.randomUUID ? crypto.randomUUID() : 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, c => ((Math.random() * 16) | 0).toString(16));
	import Card from '$lib/components/Card.svelte';
	import BoardCard from '$lib/components/BoardCard.svelte';
	import type { Label, UserResponse, TaskListWithTasks, Task } from '$lib/types/bindings';

	function mockUser(name: string): UserResponse {
		return { id: randomUUID(), email: `${name.toLowerCase()}@test.com`, name, role: 'user' };
	}

	function mockLabel(name: string, color: string): Label {
		return { id: randomUUID(), board_id: '', name, color, position: 0, created_at: '', updated_at: '' };
	}

	function mockTask(name: string, done: boolean): Task {
		return {
			id: randomUUID(), task_list_id: '', name, position: 0,
			is_completed: done, assignee_id: null, created_at: '', updated_at: '',
		};
	}

	function mockChecklist(tasks: Task[]): TaskListWithTasks {
		return {
			id: randomUUID(), card_id: '', name: 'Checklist',
			position: 0, hide_completed: false, tasks,
			created_at: '', updated_at: '',
		};
	}

	const users = {
		alice: mockUser('Alice Chen'),
		bob: mockUser('Bob Smith'),
		carol: mockUser('Carol Davis'),
	};

	const labels = {
		bug: mockLabel('bug', '#d04444'),
		feature: mockLabel('feature', '#0079bf'),
		design: mockLabel('design', '#8b5cf6'),
		urgent: mockLabel('urgent', '#e8760a'),
		docs: mockLabel('docs', '#4caf50'),
	};

	const scenarios = ['basic', 'full', 'overdue', 'empty', 'closed'] as const;
	let scenario: string = $state('basic');
</script>

<svelte:head>
	<title>Dev — Card Component</title>
</svelte:head>

<div class="dev-header">
	<h1>Dev: Card Component</h1>
	<div class="controls">
		{#each scenarios as s}
			<button class={scenario === s ? 'active' : ''} onclick={() => scenario = s}>
				{s}
			</button>
		{/each}
	</div>
	<p class="note">Types from ts-rs: <code>CardWithMembers</code>, <code>Label</code>, <code>UserResponse</code></p>
</div>

<div class="demo">
	<div class="list">
		<h3>Card Preview</h3>

		{#if scenario === 'basic'}
			<Card name="Setup CI pipeline" />
			<Card name="Write API docs" description="Document all endpoints including auth, boards, cards, and search" />
			<Card name="Fix login timeout" labels={[labels.bug]} />

		{:else if scenario === 'full'}
			<Card
				name="Refactor authentication module"
				description="Replace the current basic auth with OAuth2. This involves updating the auth service, adding token refresh logic, and migrating existing users."
				labels={[labels.feature, labels.urgent]}
				members={[users.alice, users.bob]}
				dueDate="2026-07-20T00:00:00Z"
				commentsCount={5n}
				checklists={[mockChecklist([mockTask('Design', true), mockTask('Implement', true), mockTask('Test', false), mockTask('Deploy', false)])]}
			/>

		{:else if scenario === 'overdue'}
			<Card
				name="Fix critical security vulnerability"
				labels={[labels.bug, labels.urgent]}
				members={[users.alice]}
				dueDate="2026-07-05T00:00:00Z"
				isDueCompleted={false}
				commentsCount={8n}
			/>
			<Card
				name="Update dependencies"
				dueDate="2026-07-10T00:00:00Z"
				isDueCompleted={true}
				labels={[labels.docs]}
			/>

		{:else if scenario === 'empty'}
			<Card name="Empty scenario — no metadata" />
			<div class="empty-hint">Empty list: drop cards here</div>

		{:else if scenario === 'closed'}
			<Card
				name="Deprecated feature flag system"
				description="This was replaced by the new config system in Q2"
				labels={[labels.docs]}
				members={[users.carol]}
				isClosed={true}
				commentsCount={3n}
			/>
		{/if}
	</div>

	<div class="list">
		<h3>BoardCard</h3>
		<BoardCard name="My Board" cardCount={5} />
		<BoardCard name="Empty Board" cardCount={0} />
		<BoardCard name="Sprint 2026 Q3 — Really Long Board Name" cardCount={12} />
	</div>
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
		text-transform: capitalize;
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
	.demo {
		display: flex;
		gap: 32px;
		padding: 0 16px;
		flex-wrap: wrap;
	}
	.list {
		background: #f0f2f5;
		border-radius: 10px;
		padding: 12px;
		width: 290px;
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	.list h3 {
		margin: 0 0 4px 0;
		font-size: 14px;
		color: #444;
		padding: 0 4px;
	}
	.empty-hint {
		border: 2px dashed #ccc;
		border-radius: 8px;
		padding: 24px;
		text-align: center;
		color: #999;
		font-size: 13px;
		font-style: italic;
	}
</style>
