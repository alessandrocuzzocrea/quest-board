<script lang="ts">
	let { data } = $props();
	const h = data.health;
	const mem = h.memory;
	const memMb = mem.includes('kB') ? (parseInt(mem) / 1024).toFixed(1) : '?';
	const uptime = h.uptime_seconds;
	const days = Math.floor(uptime / 86400);
	const hours = Math.floor((uptime % 86400) / 3600);
	const mins = Math.floor((uptime % 3600) / 60);
</script>

<div class="page">
	<h1>System Monitor</h1>

	<div class="grid">
		<div class="card">
			<h2>Memory</h2>
			<p class="big">{mem}</p>
			<p class="sub">~{memMb} MB</p>
		</div>

		<div class="card">
			<h2>Uptime</h2>
			<p class="big">{days}d {hours}h {mins}m</p>
			<p class="sub">{uptime.toLocaleString()} seconds</p>
		</div>

		<div class="card">
			<h2>Rust</h2>
			<p class="big">{h.rust_version ?? '?'}</p>
		</div>

		<div class="card">
			<h2>Status</h2>
			<p class="big" class:ok={h.status === 'ok'}>{h.status}</p>
		</div>
	</div>

	{#if h.db_stats}
		<h2>Database</h2>
		<table>
			<thead><tr><th>Table</th><th>Rows</th></tr></thead>
			<tbody>
				{#each Object.entries(h.db_stats) as [table, count]}
					<tr><td>{table}</td><td>{count}</td></tr>
				{/each}
			</tbody>
		</table>
	{/if}
</div>

<style>
	.page { max-width: 800px; margin: 0 auto; padding: 40px 20px; }
	h1 { margin: 0 0 24px; font-size: 24px; }
	h2 { font-size: 14px; text-transform: uppercase; color: #888; margin: 0 0 8px; letter-spacing: 0.5px; }
	.grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(180px, 1fr)); gap: 16px; margin-bottom: 32px; }
	.card { background: var(--surface, #f5f5f5); border-radius: 8px; padding: 20px; }
	.big { font-size: 20px; font-weight: 700; margin: 0; }
	.big.ok { color: #2e7d32; }
	.sub { font-size: 13px; color: #999; margin: 4px 0 0; }
	table { width: 100%; border-collapse: collapse; font-size: 14px; }
	th, td { text-align: left; padding: 8px 12px; border-bottom: 1px solid var(--border, #eee); }
	th { color: #888; text-transform: uppercase; font-size: 12px; letter-spacing: 0.5px; }
</style>
