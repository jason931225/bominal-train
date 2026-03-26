<script lang="ts">
	import { t } from '$lib/i18n';

	interface Props {
		count: number;
		onconfirm: () => void;
		oncancel: () => void;
	}

	const { count, onconfirm, oncancel }: Props = $props();

	const visible = $derived(count > 0);
</script>

{#if visible}
	<div
		class="fixed bottom-20 md:bottom-6 left-1/2 z-40 w-[calc(100%-2rem)] max-w-md"
		style="
			transform: translateX(-50%);
			animation: page-enter 0.35s cubic-bezier(0.34, 1.56, 0.64, 1) both;
		"
	>
		<div class="lg-glass-panel flex items-center gap-3 px-4 py-3 rounded-2xl">
			<!-- Count badge -->
			<span
				class="shrink-0 inline-flex items-center justify-center w-7 h-7 rounded-full text-xs font-bold"
				style="background: var(--color-brand-primary); color: var(--color-brand-text)"
			>
				{count}
			</span>

			<!-- Label -->
			<span class="flex-1 text-sm font-medium" style="color: var(--color-text-primary)">
				{count}{t('selection.selected_count')}
			</span>

			<!-- Actions -->
			<div class="flex items-center gap-2 shrink-0">
				<button
					type="button"
					class="px-3 py-1.5 rounded-xl text-xs font-medium transition-colors"
					style="color: var(--color-text-tertiary); background: var(--color-bg-sunken)"
					onclick={oncancel}
				>
					{t('selection.clear')}
				</button>

				<button
					type="button"
					class="lg-btn-primary px-4 py-1.5 rounded-xl text-xs font-semibold"
					onclick={onconfirm}
				>
					{t('selection.review')}
				</button>
			</div>
		</div>
	</div>
{/if}
ear')}
				</button>

				<button
					type="button"
					class="lg-btn-primary px-4 py-1.5 rounded-xl text-xs font-semibold"
					onclick={onconfirm}
				>
					{t('selection.review')}
				</button>
			</div>
		</div>
	</div>
{/if}
