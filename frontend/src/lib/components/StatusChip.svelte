<script lang="ts">
	interface Props {
		label: string;
		variant: 'queued' | 'running' | 'success' | 'error' | 'warning' | 'info' | 'neutral';
	}

	const { label, variant }: Props = $props();
</script>

<span
	class="inline-flex items-center px-2 py-0.5 rounded-full text-[10px] font-medium lg-badge-{variant}"
>
	{label}
</span>
