<script lang="ts">
	import type { TaskInfo } from '$lib/types';
	import { t } from '$lib/i18n';
	import { formatTime, formatDate, statusVariant, taskStatusI18nKey } from '$lib/utils';
	import StatusChip from './StatusChip.svelte';

	interface Props {
		task: TaskInfo;
		oncancel: (id: string) => void;
	}

	const { task, oncancel }: Props = $props();

	let expanded = $state(false);
	let swipeX = $state(0);
	let touchStartX = $state(0);
	let isSwiping = $state(false);

	const SWIPE_THRESHOLD = 80;
	const CANCEL_THRESHOLD = 120;

	const providerClass = $derived(
		task.provider === 'SRT' ? 'lg-provider-srt' : 'lg-provider-ktx'
	);

	const circledNumbers = ['\u2460', '\u2461', '\u2462', '\u2463', '\u2464', '\u2465', '\u2466', '\u2467', '\u2468', '\u2469'];

	function handleTouchStart(e: TouchEvent): void {
		touchStartX = e.touches[0].clientX;
		swipeX = 0;
		isSwiping = true;
	}

	function handleTouchMove(e: TouchEvent): void {
		if (!isSwiping) return;
		const delta = e.touches[0].clientX - touchStartX;
		swipeX = Math.min(0, delta);
	}

	function handleTouchEnd(): void {
		if (Math.abs(swipeX) > CANCEL_THRESHOLD) {
			oncancel(task.id);
		}
		swipeX = 0;
		isSwiping = false;
	}

	function toggleExpand(): void {
		if (Math.abs(swipeX) < 5) {
			expanded = !expanded;
		}
	}

	const cardTransform = $derived(
		isSwiping && swipeX < 0
			? `translateX(${swipeX}px)`
			: 'translateX(0)'
	);

	const revealOpacity = $derived(
		Math.min(1, Math.abs(swipeX) / SWIPE_THRESHOLD)
	);
</script>

<div class="relative overflow-hidden rounded-[1.25rem]">
	<!-- Swipe-to-cancel background -->
	<div
		class="absolute inset-0 flex items-center justify-end px-6 rounded-[1.25rem]"
		style="background: var(--color-status-error-bg); opacity: {revealOpacity}"
	>
		<span class="text-sm font-semibold" style="color: var(--color-status-error)">
			{t('task.cancel')}
		</span>
	</div>

	<!-- Card body -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="lg-glass-card relative"
		style="
			transform: {cardTransform};
			transition: {isSwiping ? 'none' : 'transform 0.3s cubic-bezier(0.16, 1, 0.3, 1)'};
		"
		ontouchstart={handleTouchStart}
		ontouchmove={handleTouchMove}
		ontouchend={handleTouchEnd}
	>
		<button
			type="button"
			class="w-full text-left p-4"
			onclick={toggleExpand}
			aria-expanded={expanded}
		>
			<!-- Top row: provider badge + status -->
			<div class="flex items-center justify-between mb-2">
				<span
					class="inline-flex items-center px-2 py-0.5 rounded-lg text-[11px] font-bold tracking-tight {providerClass}"
				>
					{task.provider}
				</span>
				<StatusChip label={t(taskStatusI18nKey(task.status))} variant={statusVariant(task.status)} />
			</div>

			<!-- Route -->
			<div class="flex items-center gap-2 mb-1">
				<span class="text-base font-semibold" style="color: var(--color-text-primary)">
					{task.departure_station}
				</span>
				<svg class="w-4 h-4 shrink-0" style="color: var(--color-text-disabled)" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
					<line x1="5" y1="12" x2="19" y2="12" />
					<polyline points="12 5 19 12 12 19" />
				</svg>
				<span class="text-base font-semibold" style="color: var(--color-text-primary)">
					{task.arrival_station}
				</span>
			</div>

			<!-- Date / Time / Target count -->
			<div class="flex items-center gap-3 text-xs" style="color: var(--color-text-tertiary)">
				<span>{formatDate(task.travel_date)}</span>
				<span>{formatTime(task.departure_time)}</span>
				<span class="ml-auto">
					{task.target_trains.length} {t('task.schedules_title').toLowerCase()}
				</span>
			</div>
		</button>

		<!-- Expandable target list -->
		{#if expanded}
			<div class="border-t px-4 pb-4 pt-3" style="border-color: var(--color-border-subtle)">
				<p class="text-[11px] font-medium mb-2" style="color: var(--color-text-tertiary)">
					{t('task.schedules_title')}
				</p>
				<ul class="flex flex-col gap-1.5">
					{#each task.target_trains as train, i}
						{@const tProvClass = train.provider === 'SRT' ? 'lg-provider-srt' : 'lg-provider-ktx'}
						<li class="flex items-center gap-2 text-sm page-enter stagger-{Math.min(i + 1, 5)}">
							<span
								class="shrink-0 w-5 h-5 flex items-center justify-center rounded-full text-[10px] font-semibold"
								style="background: var(--lg-accent-bg); color: var(--lg-accent-text)"
							>
								{circledNumbers[i] ?? i + 1}
							</span>
							<span
								class="px-1.5 py-0.5 rounded text-[10px] font-bold {tProvClass}"
							>
								{train.provider}
							</span>
							<span style="color: var(--color-text-primary)">#{train.train_number}</span>
							<span class="ml-auto" style="color: var(--color-text-tertiary)">
								{formatTime(train.dep_time)}
							</span>
						</li>
					{/each}
				</ul>
			</div>
		{/if}
	</div>
</div>
