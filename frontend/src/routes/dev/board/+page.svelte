<script lang="ts">
	import KanbanColumn from '$lib/components/KanbanColumn.svelte';
	import type { CardWithMembers, UserResponse, Label, TaskListWithTasks, Task } from '$lib/types/bindings';

	const randomUUID = () => crypto.randomUUID ? crypto.randomUUID()
		: 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, c => ((Math.random() * 16) | 0).toString(16));

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
			id: randomUUID(), card_id: '', name: 'Checklist', position: 0,
			hide_completed: false, tasks, created_at: '', updated_at: '',
		};
	}

	function mockCard(name: string, opts: {
		labels?: Label[]; members?: UserResponse[]; desc?: string;
		due?: string; dueDone?: boolean; comments?: bigint;
		checklist?: TaskListWithTasks[]; closed?: boolean;
	} = {}): CardWithMembers {
		return {
			id: randomUUID(), board_id: '', list_id: '', position: 0,
			name, description: opts.desc ?? null, due_date: opts.due ?? null,
			is_due_completed: opts.dueDone ?? false, is_closed: opts.closed ?? false,
			created_by: randomUUID(), members: opts.members ?? [], labels: opts.labels ?? [],
			comments_count: opts.comments ?? 0n, checklists: opts.checklist ?? [],
			created_at: '', updated_at: '',
		};
	}

	const users = {
		a: mockUser('Alice Chen'), b: mockUser('Bob Smith'), c: mockUser('Carol Davis'),
	};

	const labels = {
		bug: mockLabel('bug', '#d04444'), feat: mockLabel('feature', '#0079bf'),
		design: mockLabel('design', '#8b5cf6'), urgent: mockLabel('urgent', '#e8760a'),
		docs: mockLabel('docs', '#4caf50'),
	};

	const initialCards: Record<string, CardWithMembers[]> = {
		'todo': [
			mockCard('Setup CI pipeline'),
			mockCard('Write API docs', { desc: 'Document all endpoints', labels: [labels.docs], members: [users.b] }),
			mockCard('Fix login timeout', { labels: [labels.bug], members: [users.a, users.b] }),
			mockCard('Refactor auth', { labels: [labels.feat, labels.urgent], members: [users.a], checklist: [
				mockChecklist([mockTask('Design', true), mockTask('Implement', false)])
			]}),
		],
		'wip': [
			mockCard('API rate limiting', { labels: [labels.feat], members: [users.a], due: '2026-07-18T00:00:00Z' }),
			mockCard('Dashboard UI', { labels: [labels.design], members: [users.c], comments: 3n }),
		],
		'done': [
			mockCard('Init repo', { members: [users.a] }),
			mockCard('Design system', { labels: [labels.design], members: [users.a, users.c], comments: 5n, checklist: [
				mockChecklist([mockTask('Colors', true), mockTask('Typography', true), mockTask('Icons', true)])
			]}),
			mockCard('Deprecated feature', { labels: [labels.docs], closed: true, members: [users.c] }),
		],
	};

	const listMeta = [
		{ id: 'todo', title: 'To Do', color: null as string | null },
		{ id: 'wip', title: 'In Progress', color: '#0079bf' },
		{ id: 'done', title: 'Done', color: '#4caf50' },
	];

	let columns = $state(listMeta.map(m => ({
		...m,
		cards: [...initialCards[m.id]],
	})));

	function moveCard(cardId: string, sourceListId: string, targetListId: string) {
		const src = columns.find(c => c.id === sourceListId);
		const tgt = columns.find(c => c.id === targetListId);
		if (!src || !tgt) return;
		const idx = src.cards.findIndex(c => c.id === cardId);
		if (idx === -1) return;
		const [card] = src.cards.splice(idx, 1);
		card.list_id = targetListId;
		tgt.cards.push(card);
		columns = columns; // trigger reactivity
	}
</script>

<svelte:head>
	<title>Dev — Board with Drag & Drop</title>
</svelte:head>

<div class="dev-header">
	<h1>Dev: Board with Drag & Drop</h1>
	<p class="note">Drag cards between columns. Drop on empty state to move.</p>
</div>

<div class="board">
	{#each columns as col (col.id)}
		<KanbanColumn
			title={col.title}
			listId={col.id}
			cards={col.cards}
			color={col.color}
			cardCount={col.cards.length}
			onDropCard={moveCard}
		/>
	{/each}
</div>

<style>
	.dev-header {
		padding: 16px;
		border-bottom: 1px solid var(--border, #ddd);
		margin-bottom: 12px;
	}
	.note {
		font-size: 13px;
		color: var(--text-muted);
		margin: 4px 0 0;
	}
	.board {
		display: flex;
		gap: 12px;
		padding: 0 16px 16px;
		overflow-x: auto;
	}
</style>
