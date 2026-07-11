<script lang="ts">
	import BoardCard from './BoardCard.svelte';
	import { Meta, Story, Template } from '@storybook/addon-svelte-csf';
</script>

<Meta title="BoardCard" component={BoardCard} />

<Template let:args>
	<BoardCard name={args.name} cardCount={args.cardCount} />
</Template>

<Story name="Default" args={{ name: 'Sprint 42', cardCount: 5 }} />
<Story name="Empty" args={{ name: 'New Board', cardCount: 0 }} />
<Story name="Untitled" args={{ cardCount: 0 }} />
