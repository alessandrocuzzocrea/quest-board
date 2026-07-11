<script lang="ts">
	let { name = '?', size = 28 }: { name: string; size?: number } = $props();

	const initials = $derived(
		name.split(' ').map(w => w[0]).filter(Boolean).slice(0, 2).join('').toUpperCase()
	);
	const hue = $derived(
		name.split('').reduce((acc, c) => acc + c.charCodeAt(0), 0) % 360
	);
</script>

<div class="avatar" style="width: {size}px; height: {size}px; background: hsl({hue}, 55%, 55%); font-size: {size * 0.42}px;">
	{initials}
</div>

<style>
	.avatar {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		border-radius: 50%;
		color: white;
		font-weight: 600;
		line-height: 1;
		user-select: none;
		flex-shrink: 0;
	}
</style>
