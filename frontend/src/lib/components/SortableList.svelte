<script lang="ts">
	import { t } from '$lib/i18n';

	interface SortableItem {
		id: string;
		label: string;
	}

	interface Props {
		items: SortableItem[];
		onReorder: (items: SortableItem[]) => void;
		onRemove: (id: string) => void;
	}

	const { items, onReorder, onRemove }: Props = $props();

	const circledNumbers = ['\u2460', '\u2461', '\u2462', '\u2463', '\u2464', '\u2465', '\u2466', '\u2467', '\u2468', '\u2469'];

	function badge(index: number): string {
		return circledNumbers[index] ?? `${index + 1}`;
	}

	function moveUp(index: number): void {
		if (index <= 0) return;
		const next = [...items];
		const temp = next[index - 1];
		next[index - 1] = next[index];
		next[index] = temp;
		onReorder(next);
	}

	function moveDown(index: number): void {
		if (index >= items.length - 1) return;
		const next = [...items];
		const temp = next[index + 1];
		next[index + 1] = next[index];
		next[index] = temp;
		onReorder(next);
	}

	function staggerClass(index: number): string {
		const n = Math.min(index + 1, 5);
		return `page-enter stagger-${n}`;
	}
</script>

<ul class="flex flex-col gap-2" role="list">
	{#each items as item, i (item.id)}
		<li class="lg-glass-card flex items-center gap-3 px-4 py-3 {staggerClass(i)}">
			<span
				class="shrink-0 w-7 h-7 flex items-center justify-center rounded-full text-sm font-semibold"
				style="background: var(--color-brand-primary); color: var(--color-brand-text)"
			>
				{badge(i)}
			</span>

			<span class="flex-1 text-sm font-medium truncate" style="color: var(--color-text-primary)">
				{item.label}
			</span>

			<div class="flex items-center gap-1 shrink-0">
				<button
					type="button"
					class="p-1.5 rounded-lg transition-colors hover:bg-[var(--color-interactive-hover)] disabled:opacity-30"
					disabled={i === 0}
					onclick={() => moveUp(i)}
					aria-label={t('task.move_up')}
				>
					<svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
						<polyline points="18 15 12 9 6 15" />
					</svg>
				</button>

				<button
					type="button"
					class="p-1.5 rounded-lg transition-colors hover:bg-[var(--color-interactive-hover)] disabled:opacity-30"
					disabled={i === items.length - 1}
					onclick={() => moveDown(i)}
					aria-label={t('task.move_down')}
				>
					<svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
						<polyline points="6 9 12 15 18 9" />
					</svg>
				</button>

				<button
					type="button"
					class="p-1.5 rounded-lg transition-colors hover:bg-[var(--color-status-error-bg)]"
					style="color: var(--color-status-error)"
					onclick={() => onRemove(item.id)}
					aria-label={t('task.remove_train')}
				>
					<svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
						<line x1="18" y1="6" x2="6" y2="18" />
						<line x1="6" y1="6" x2="18" y2="18" />
					</svg>
				</button>
			</div>
		</li>
	{/each}
</ul>
