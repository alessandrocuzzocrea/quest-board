<script lang="ts">
	let { name = 'Untitled', cardCount = 0 }: { name: string; cardCount: number } = $props();
</script>

<div class="card">
	<h3>{name}</h3>
	<span class="count">{cardCount} cards</span>
</div>

<style>
	.card {
		background: var(--surface);
		border-radius: 8px;
		padding: 16px;
		cursor: pointer;
		transition: box-shadow 0.15s;
	}
	.card:hover {
		box-shadow: 0 2px 8px rgba(0,0,0,0.15);
	}
	.count {
		color: var(--text-muted);
		font-size: 13px;
	}
</style>
