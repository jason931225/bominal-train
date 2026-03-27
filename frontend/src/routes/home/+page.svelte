<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { goto } from '$app/navigation';
	import { t } from '$lib/i18n';
	import { sseStore } from '$lib/stores/sse.svelte';
	import { listTasks } from '$lib/api/tasks';
	import type { TaskInfo, TaskStatus } from '$lib/types';
	import { formatTime, formatDate, statusVariant, taskStatusI18nKey } from '$lib/utils';
	import GlassPanel from '$lib/components/GlassPanel.svelte';
	import StatusChip from '$lib/components/StatusChip.svelte';
	import Skeleton from '$lib/components/Skeleton.svelte';

	const ACTIVE_STATUSES: TaskStatus[] = ['queued', 'running', 'idle'];

	let tasks = $state<TaskInfo[]>([]);
	let loading = $state(true);
	let error = $state('');

	/* ── Pull-to-refresh state ── */
	let pullStartY = $state(0);
	let pullDistance = $state(0);
	let refreshing = $state(false);
	let scrollContainer: HTMLElement | undefined = $state();

	const activeTasks = $derived(tasks.filter((task) => ACTIVE_STATUSES.includes(task.status)));

	let unsubscribeSse: (() => void) | undefined;

	async function fetchTasks(): Promise<void> {
		try {
			tasks = await listTasks();
			error = '';
		} catch (err) {
			error = err instanceof Error ? err.message : t('error.load_failed');
		} finally {
			loading = false;
		}
	}

	async function handleRefresh(): Promise<void> {
		refreshing = true;
		await fetchTasks();
		refreshing = false;
	}

	function onTouchStart(e: TouchEvent): void {
		if (!scrollContainer || scrollContainer.scrollTop > 0) return;
		pullStartY = e.touches[0].clientY;
	}

	function onTouchMove(e: TouchEvent): void {
		if (pullStartY === 0) return;
		const distance = e.touches[0].clientY - pullStartY;
		pullDistance = Math.max(0, Math.min(distance * 0.5, 80));
	}

	function onTouchEnd(): void {
		if (pullDistance > 50 && !refreshing) {
			handleRefresh();
		}
		pullStartY = 0;
		pullDistance = 0;
	}

	onMount(async () => {
		await fetchTasks();
		unsubscribeSse = sseStore.subscribe(() => {
			fetchTasks();
		});
	});

	onDestroy(() => {
		unsubscribeSse?.();
	});
</script>

<svelte:head><title>{t('nav.home')} | Bominal</title></svelte:head>

<div
	class="page-container page-enter"
	bind:this={scrollContainer}
	ontouchstart={onTouchStart}
	ontouchmove={onTouchMove}
	ontouchend={onTouchEnd}
>
	<!-- Pull-to-refresh indicator -->
	{#if pullDistance > 0 || refreshing}
		<div
			class="flex justify-center pb-4 transition-transform"
			style="transform: translateY({pullDistance}px)"
		>
			<svg
				class="w-5 h-5 animate-spin"
				style="color: var(--color-brand-text)"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
			>
				<path d="M21 12a9 9 0 1 1-6.22-8.56" />
			</svg>
		</div>
	{/if}

	<!-- Hero -->
	<div class="text-center mb-8">
		<h1 class="app-brand-wordmark bg-clip-text text-5xl font-bold tracking-tight text-transparent">
			bominal
		</h1>
		<p class="mt-2 text-sm" style="color: var(--color-text-tertiary)">
			{t('home.description')}
		</p>
	</div>

	<!-- Quick-action cards -->
	<div class="grid grid-cols-2 gap-3 mb-8">
		<!-- Search card -->
		<button
			class="lg-btn-primary squish flex flex-col items-center gap-3 rounded-2xl px-4 py-6 text-center"
			onclick={() => goto('/search')}
		>
			<svg
				class="w-7 h-7"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
				stroke-linecap="round"
				stroke-linejoin="round"
			>
				<circle cx="11" cy="11" r="8" />
				<line x1="21" y1="21" x2="16.65" y2="16.65" />
			</svg>
			<span class="text-sm font-semibold">{t('home.start_search')}</span>
			<span class="text-xs opacity-80">{t('home.start_search_desc')}</span>
		</button>

		<!-- Tasks card -->
		<button
			class="lg-glass-card lg-glass-card-hover squish flex flex-col items-center gap-3 rounded-2xl px-4 py-6 text-center"
			onclick={() => goto('/tasks')}
		>
			<svg
				class="w-7 h-7"
				style="color: var(--color-brand-text)"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
				stroke-linecap="round"
				stroke-linejoin="round"
			>
				<line x1="8" y1="6" x2="21" y2="6" />
				<line x1="8" y1="12" x2="21" y2="12" />
				<line x1="8" y1="18" x2="21" y2="18" />
				<line x1="3" y1="6" x2="3.01" y2="6" />
				<line x1="3" y1="12" x2="3.01" y2="12" />
				<line x1="3" y1="18" x2="3.01" y2="18" />
			</svg>
			<span class="text-sm font-semibold" style="color: var(--color-text-primary)">
				{t('home.open_tasks')}
			</span>
			<span class="text-xs" style="color: var(--color-text-tertiary)">
				{t('home.open_tasks_desc')}
			</span>
		</button>
	</div>

	<!-- Active tasks summary -->
	<section>
		<h2 class="text-lg font-semibold mb-3" style="color: var(--color-text-primary)">
			{t('home.active_tasks')}
			{#if activeTasks.length > 0}
				<span
					class="ml-2 inline-flex items-center justify-center w-6 h-6 rounded-full text-xs font-bold"
					style="background: var(--color-brand-primary); color: var(--color-brand-text)"
				>
					{activeTasks.length}
				</span>
			{/if}
		</h2>

		{#if loading}
			<GlassPanel>
				<Skeleton lines={4} />
			</GlassPanel>
		{:else if error}
			<GlassPanel class="text-center">
				<p class="text-sm" style="color: var(--color-status-error)">{error}</p>
				<button
					class="lg-btn-secondary squish mt-3 rounded-xl px-4 py-2 text-sm"
					onclick={fetchTasks}
				>
					{t('common.retry')}
				</button>
			</GlassPanel>
		{:else if activeTasks.length === 0}
			<GlassPanel class="text-center">
				<p class="text-sm" style="color: var(--color-text-tertiary)">
					{t('home.no_active_tasks')}
				</p>
				<button
					class="lg-btn-secondary squish mt-3 rounded-xl px-4 py-2 text-sm"
					onclick={() => goto('/search')}
				>
					{t('search.go_to_search')}
				</button>
			</GlassPanel>
		{:else}
			<div class="flex flex-col gap-3">
				{#each activeTasks as task, i (task.id)}
					<button
						class="lg-glass-card lg-glass-card-hover squish flex items-center gap-3 rounded-2xl px-4 py-3 text-left page-enter stagger-{Math.min(i + 1, 5)}"
						onclick={() => goto('/tasks')}
					>
						<div class="flex-1 min-w-0">
							<div class="flex items-center gap-2 mb-1">
								<span
									class="shrink-0 inline-flex items-center px-1.5 py-0.5 rounded-md text-[10px] font-bold tracking-wider"
									class:lg-provider-srt={task.provider === 'SRT'}
								class:lg-provider-ktx={task.provider === 'KTX'}
							>
									{task.provider}
								</span>
								<StatusChip
									label={t(taskStatusI18nKey(task.status))}
									variant={statusVariant(task.status)}
								/>
							</div>
							<p class="text-sm font-medium truncate" style="color: var(--color-text-primary)">
								{task.departure_station} → {task.arrival_station}
							</p>
							<p class="text-xs mt-0.5" style="color: var(--color-text-tertiary)">
								{formatDate(task.travel_date)} · {formatTime(task.departure_time)}
							</p>
						</div>

						<svg
							class="w-4 h-4 shrink-0"
							style="color: var(--color-text-tertiary)"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
							stroke-linecap="round"
							stroke-linejoin="round"
						>
							<polyline points="9 18 15 12 9 6" />
						</svg>
					</button>
				{/each}
			</div>
		{/if}
	</section>
</div>
