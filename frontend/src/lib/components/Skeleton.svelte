<script lang="ts">
	interface Props {
		class?: string;
		lines?: number;
	}

	const { class: className = '', lines = 3 }: Props = $props();

	const widths = ['100%', '75%', '50%'];

	function lineWidth(index: number): string {
		return widths[index % widths.length];
	}
</script>

<div class="flex flex-col gap-2.5 {className}" role="status" aria-label="Loading">
	{#each Array(lines) as _, i}
		<div
			class="lg-shimmer h-3 rounded-lg"
			style="width: {lineWidth(i)}"
		></div>
	{/each}
</div>
