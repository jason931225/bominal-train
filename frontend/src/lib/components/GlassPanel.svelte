<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		class?: string;
		children: Snippet;
	}

	const { class: className = '', children }: Props = $props();
</script>

<div class="lg-glass-panel p-6 {className}">
	{@render children()}
</div>
