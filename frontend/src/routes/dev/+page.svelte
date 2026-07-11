<svelte:head>
	<title>Dev Components</title>
</svelte:head>

<h1>Dev Routes</h1>
<p>Component development with mock data. Guarded — only loads in development mode.</p>

<ul>
	<li><a href="/dev/board">Board View</a> — Board with lists and cards in various states</li>
</ul>

<style>
	ul { list-style: none; padding: 0; }
	li { margin: 8px 0; }
	a { color: var(--accent, #0079bf); text-decoration: none; font-size: 16px; }
	a:hover { text-decoration: underline; }
</style>
