<script lang="ts">
	let count = $state(0);
</script>

<div class="page">
	<h1>quest-board</h1>
	<p style="color: var(--text-muted)">Hello from SvelteKit!</p>
	<button onclick={() => count++}>count is {count}</button>
</div>

<style>
	.page {
		max-width: 480px;
		margin: 80px auto;
		display: flex;
		flex-direction: column;
		gap: 16px;
		align-items: center;
	}
</style>
