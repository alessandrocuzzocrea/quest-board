<script lang="ts">
	let { name = '', color = '#0079bf' }: { name: string; color?: string } = $props();
</script>

<span class="badge" style="background: {color}">
	{name}
</span>

<style>
	.badge {
		display: inline-block;
		padding: 2px 8px;
		border-radius: 4px;
		font-size: 11px;
		font-weight: 600;
		color: white;
		line-height: 1.4;
		white-space: nowrap;
	}
</style>
