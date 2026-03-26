<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { goto } from '$app/navigation';
	import { t } from '$lib/i18n';
	import { sseStore } from '$lib/stores/sse.svelte';
	import { listTasks, deleteTask } from '$lib/api/tasks';
	import type { TaskInfo, TaskStatus } from '$lib/types';
	import {
		formatTime,
		formatDate,
		formatCost,
		statusVariant,
		taskStatusI18nKey
	} from '$lib/utils';
	import GlassPanel from '$lib/components/GlassPanel.svelte';
	import StatusChip from '$lib/components/StatusChip.svelte';
	import Skeleton from '$lib/components/Skeleton.svelte';

	const ACTIVE_STATUSES: TaskStatus[] = ['queued', 'running', 'idle', 'awaiting_payment'];
	const COMPLETED_STATUSES: TaskStatus[] = ['confirmed', 'failed', 'cancelled'];

	let tasks = $state<TaskInfo[]>([]);
	let loading = $state(true);
	let error = $state('');
	let activeTab = $state<'active' | 'completed'>('active');

	/* ── Pull-to-refresh state ── */
	let pullStartY = $state(0);
	let pullDistance = $state(0);
	let refreshing = $state(false);
	let scrollContainer: HTMLElement | undefined = $state();

	/* ── Swipe-to-cancel state ── */
	let swipeTaskId = $state<string | null>(null);
	let swipeStartX = $state(0);
	let swipeOffset = $state(0);
	let cancellingId = $state<string | null>(null);

	/* ── Expanded task details ── */
	let expandedId = $state<string | null>(null);

	const activeTasks = $derived(tasks.filter((task) => ACTIVE_STATUSES.includes(task.status)));
	const completedTasks = $derived(tasks.filter((task) => COMPLETED_STATUSES.includes(task.status)));
	const displayTasks = $derived(activeTab === 'active' ? activeTasks : completedTasks);

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

	async function handleCancel(id: string): Promise<void> {
		cancellingId = id;
		try {
			await deleteTask(id);
			tasks = tasks.filter((task) => task.id !== id);
		} catch (err) {
			error = err instanceof Error ? err.message : t('error.unexpected');
		} finally {
			cancellingId = null;
			swipeTaskId = null;
			swipeOffset = 0;
		}
	}

	function toggleExpand(id: string): void {
		expandedId = expandedId === id ? null : id;
	}

	/* ── Pull-to-refresh handlers ── */
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

	/* ── Swipe-to-cancel handlers ── */
	function onSwipeStart(e: TouchEvent, taskId: string): void {
		swipeTaskId = taskId;
		swipeStartX = e.touches[0].clientX;
		swipeOffset = 0;
	}

	function onSwipeMove(e: TouchEvent): void {
		if (swipeTaskId === null) return;
		const delta = swipeStartX - e.touches[0].clientX;
		swipeOffset = Math.max(0, Math.min(delta, 100));
	}

	function onSwipeEnd(): void {
		if (swipeOffset > 60 && swipeTaskId !== null) {
			// Keep revealed state for cancel action
		} else {
			swipeTaskId = null;
			swipeOffset = 0;
		}
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

<svelte:head><title>{t('nav.tasks')} | Bominal</title></svelte:head>

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

	<!-- Tabs -->
	<div class="flex rounded-xl overflow-hidden mb-5" style="background: var(--color-bg-sunken)">
		<button
			type="button"
			class="flex-1 py-2.5 text-sm font-medium transition-all rounded-xl squish"
			class:lg-active={activeTab === 'active'}
			style={activeTab === 'active'
				? 'color: var(--color-brand-text)'
				: 'color: var(--color-text-secondary)'}
			onclick={() => { activeTab = 'active'; }}
		>
			{t('task.active')}
			{#if activeTasks.length > 0}
				<span
					class="ml-1 inline-flex items-center justify-center w-5 h-5 rounded-full text-[10px] font-bold"
					style="background: var(--color-brand-primary); color: var(--color-brand-text)"
				>
					{activeTasks.length}
				</span>
			{/if}
		</button>
		<button
			type="button"
			class="flex-1 py-2.5 text-sm font-medium transition-all rounded-xl squish"
			class:lg-active={activeTab === 'completed'}
			style={activeTab === 'completed'
				? 'color: var(--color-brand-text)'
				: 'color: var(--color-text-secondary)'}
			onclick={() => { activeTab = 'completed'; }}
		>
			{t('task.completed')}
		</button>
	</div>

	<!-- Loading -->
	{#if loading}
		<GlassPanel>
			<Skeleton lines={5} />
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
	{:else if displayTasks.length === 0}
		<!-- Empty state -->
		<GlassPanel class="text-center py-10">
			<svg
				class="w-12 h-12 mx-auto mb-4"
				style="color: var(--color-text-disabled)"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="1.5"
				stroke-linecap="round"
				stroke-linejoin="round"
			>
				<rect x="2" y="7" width="20" height="14" rx="2" ry="2" />
				<polyline points="16 3 12 7 8 3" />
			</svg>
			<p class="text-sm mb-3" style="color: var(--color-text-tertiary)">
				{activeTab === 'active' ? t('task.no_active') : t('task.no_completed')}
			</p>
			{#if activeTab === 'active'}
				<button
					class="lg-btn-secondary squish rounded-xl px-4 py-2 text-sm"
					onclick={() => goto('/search')}
				>
					{t('search.go_to_search')}
				</button>
			{/if}
		</GlassPanel>
	{:else}
		<!-- Task list -->
		<div class="flex flex-col gap-3">
			{#each displayTasks as task, i (task.id)}
				<div class="relative overflow-hidden rounded-2xl page-enter stagger-{Math.min(i + 1, 5)}">
					<!-- Swipe-behind cancel action -->
					{#if swipeTaskId === task.id && swipeOffset > 0}
						<div
							class="absolute inset-y-0 right-0 flex items-center justify-center px-5"
							style="background: var(--color-status-error); width: {swipeOffset}px"
						>
							<button
								type="button"
								class="text-white text-xs font-bold"
								onclick={() => handleCancel(task.id)}
								disabled={cancellingId === task.id}
							>
								{cancellingId === task.id ? t('task.cancelling') : t('task.cancel')}
							</button>
						</div>
					{/if}

					<!-- Task card -->
					<div
						class="lg-glass-card rounded-2xl transition-transform"
						style="transform: translateX({swipeTaskId === task.id ? -swipeOffset : 0}px)"
						ontouchstart={(e) => onSwipeStart(e, task.id)}
						ontouchmove={onSwipeMove}
						ontouchend={onSwipeEnd}
					>
						<button
							type="button"
							class="w-full text-left px-4 py-3"
							onclick={() => toggleExpand(task.id)}
						>
							<!-- Header row -->
							<div class="flex items-center gap-2 mb-1.5">
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
								<span class="flex-1"></span>
								<span class="text-xs tabular-nums" style="color: var(--color-text-tertiary)">
									{task.attempt_count} {t('task.attempts')}
								</span>
							</div>

							<!-- Route -->
							<p class="text-sm font-semibold" style="color: var(--color-text-primary)">
								{task.departure_station} → {task.arrival_station}
							</p>

							<!-- Date & time -->
							<p class="text-xs mt-0.5" style="color: var(--color-text-tertiary)">
								{formatDate(task.travel_date)} · {formatTime(task.departure_time)}
								· {task.target_trains.length} {t('task.total')}
							</p>

							<!-- Reservation snapshot -->
							{#if task.reservation}
								<div
									class="mt-2 rounded-xl px-3 py-2"
									style="background: var(--color-status-success-bg)"
								>
									<p class="text-xs font-medium" style="color: var(--color-status-success)">
										{task.reservation.train_number} · {formatTime(task.reservation.dep_time)}
										· {formatCost(task.reservation.total_cost)}
									</p>
								</div>
							{/if}
						</button>

						<!-- Expanded details -->
						{#if expandedId === task.id}
							<div class="border-t px-4 py-3" style="border-color: var(--color-border-subtle)">
								<!-- Target trains -->
								<p class="text-xs font-medium mb-2" style="color: var(--color-text-tertiary)">
									{t('task.schedules_title')}
								</p>
								<div class="flex flex-wrap gap-1.5 mb-3">
									{#each task.target_trains as train}
										<span
											class="inline-flex items-center gap-1 px-2 py-1 rounded-lg text-xs"
											style="background: var(--color-bg-sunken); color: var(--color-text-secondary)"
										>
											{train.provider} {train.train_number} {formatTime(train.dep_time)}
										</span>
									{/each}
								</div>

								<!-- Meta info -->
								<div class="grid grid-cols-2 gap-2 text-xs">
									<div>
										<span style="color: var(--color-text-tertiary)">{t('task.seat_class')}</span>
										<p class="font-medium" style="color: var(--color-text-primary)">
											{task.seat_preference}
										</p>
									</div>
									<div>
										<span style="color: var(--color-text-tertiary)">{t('task.passengers_label')}</span>
										<p class="font-medium" style="color: var(--color-text-primary)">
											{task.passengers.map((p) => `${t('passenger.' + p.type)} ${p.count}`).join(', ')}
										</p>
									</div>
									<div>
										<span style="color: var(--color-text-tertiary)">{t('task.started_at')}</span>
										<p class="font-medium tabular-nums" style="color: var(--color-text-primary)">
											{task.started_at ? formatDate(task.started_at) : t('task.not_started')}
										</p>
									</div>
									<div>
										<span style="color: var(--color-text-tertiary)">{t('task.last_attempt')}</span>
										<p class="font-medium tabular-nums" style="color: var(--color-text-primary)">
											{task.last_attempt_at ? formatTime(task.last_attempt_at) : t('task.no_attempt')}
										</p>
									</div>
								</div>

								<!-- Actions -->
								{#if ACTIVE_STATUSES.includes(task.status)}
									<div class="flex gap-2 mt-3 pt-3" style="border-top: 1px solid var(--color-border-subtle)">
										<button
											type="button"
											class="lg-btn-danger squish flex-1 rounded-xl py-2 text-sm"
											disabled={cancellingId === task.id}
											onclick={() => handleCancel(task.id)}
										>
											{cancellingId === task.id ? t('task.cancelling') : t('task.cancel')}
										</button>
									</div>
								{/if}
							</div>
						{/if}
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>
