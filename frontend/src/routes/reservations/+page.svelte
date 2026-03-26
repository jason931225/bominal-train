<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { t } from '$lib/i18n';
	import {
		listReservations,
		ticketDetail,
		cancelReservation,
		payReservation,
		refundReservation
	} from '$lib/api/reservations';
	import { listCards } from '$lib/api/cards';
	import type { ReservationInfo, TicketInfo, CardInfo, Provider } from '$lib/types';
	import { formatTime, formatDate, formatCost } from '$lib/utils';
	import GlassPanel from '$lib/components/GlassPanel.svelte';
	import CardBrand from '$lib/components/CardBrand.svelte';
	import Skeleton from '$lib/components/Skeleton.svelte';

	let providerFilter = $state<'Both' | 'SRT' | 'KTX'>('Both');
	let reservations = $state<ReservationInfo[]>([]);
	let loading = $state(true);
	let error = $state('');

	/* ── Pull-to-refresh ── */
	let pullStartY = $state(0);
	let pullDistance = $state(0);
	let refreshing = $state(false);
	let scrollContainer: HTMLElement | undefined = $state();

	/* ── Expanded detail state ── */
	let expandedPnr = $state<string | null>(null);
	let tickets = $state<Record<string, TicketInfo[]>>({});
	let ticketsLoading = $state<Record<string, boolean>>({});

	/* ── Action state ── */
	let actionLoading = $state<Record<string, string>>({});
	let showPayCard = $state<string | null>(null);
	let cards = $state<CardInfo[]>([]);
	let selectedPayCardId = $state<string | null>(null);

	/* ── Confirmation dialog ── */
	let confirmDialog = $state<{ pnr: string; provider: string; action: 'cancel' | 'refund' } | null>(null);

	async function fetchReservations(): Promise<void> {
		loading = true;
		error = '';
		try {
			if (providerFilter === 'Both') {
				const [srt, ktx] = await Promise.all([
					listReservations('SRT'),
					listReservations('KTX')
				]);
				reservations = [...srt, ...ktx];
			} else {
				reservations = await listReservations(providerFilter);
			}
		} catch (err) {
			error = err instanceof Error ? err.message : t('error.load_failed');
			reservations = [];
		} finally {
			loading = false;
		}
	}

	async function handleRefresh(): Promise<void> {
		refreshing = true;
		await fetchReservations();
		refreshing = false;
	}

	async function toggleExpand(res: ReservationInfo): Promise<void> {
		const key = `${res.provider}:${res.reservation_number}`;
		if (expandedPnr === key) {
			expandedPnr = null;
			return;
		}
		expandedPnr = key;

		if (!tickets[key]) {
			ticketsLoading = { ...ticketsLoading, [key]: true };
			try {
				const detail = await ticketDetail(res.provider, res.reservation_number);
				tickets = { ...tickets, [key]: detail };
			} catch {
				tickets = { ...tickets, [key]: [] };
			} finally {
				ticketsLoading = { ...ticketsLoading, [key]: false };
			}
		}
	}

	async function handlePay(provider: string, pnr: string): Promise<void> {
		if (!selectedPayCardId) return;
		const key = `${provider}:${pnr}`;
		actionLoading = { ...actionLoading, [key]: 'pay' };
		try {
			await payReservation(provider, pnr, selectedPayCardId);
			showPayCard = null;
			selectedPayCardId = null;
			await fetchReservations();
		} catch (err) {
			error = err instanceof Error ? err.message : t('error.unexpected');
		} finally {
			actionLoading = { ...actionLoading, [key]: '' };
		}
	}

	async function handleConfirmAction(): Promise<void> {
		if (!confirmDialog) return;
		const { pnr, provider, action } = confirmDialog;
		const key = `${provider}:${pnr}`;
		actionLoading = { ...actionLoading, [key]: action };
		try {
			if (action === 'cancel') {
				await cancelReservation(provider, pnr);
			} else {
				await refundReservation(provider, pnr);
			}
			confirmDialog = null;
			await fetchReservations();
		} catch (err) {
			error = err instanceof Error ? err.message : t('error.unexpected');
		} finally {
			actionLoading = { ...actionLoading, [key]: '' };
		}
	}

	async function openPayCard(provider: string, pnr: string): Promise<void> {
		showPayCard = `${provider}:${pnr}`;
		if (cards.length === 0) {
			try {
				cards = await listCards();
				if (cards.length > 0) {
					selectedPayCardId = cards[0].id;
				}
			} catch {
				cards = [];
			}
		}
	}

	function onProviderChange(filter: 'Both' | 'SRT' | 'KTX'): void {
		providerFilter = filter;
		fetchReservations();
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

	onMount(() => {
		fetchReservations();
	});
</script>

<svelte:head><title>{t('reservation.title')} | Bominal</title></svelte:head>

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

	<!-- Title -->
	<h1 class="text-xl font-bold mb-5" style="color: var(--color-text-primary)">
		{t('reservation.title')}
	</h1>

	<!-- Provider filter -->
	<div class="flex rounded-xl overflow-hidden mb-5" style="background: var(--color-bg-sunken)">
		{#each (['SRT', 'KTX', 'Both'] as const) as option}
			<button
				type="button"
				class="flex-1 py-2.5 text-sm font-medium transition-all rounded-xl squish"
				class:lg-active={providerFilter === option}
				style={providerFilter === option
					? 'color: var(--color-brand-text)'
					: 'color: var(--color-text-secondary)'}
				onclick={() => onProviderChange(option)}
			>
				{option === 'Both' ? 'Both' : option}
			</button>
		{/each}
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
				onclick={fetchReservations}
			>
				{t('common.retry')}
			</button>
		</GlassPanel>
	{:else if reservations.length === 0}
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
				<path d="M2 9a3 3 0 0 1 3-3h14a3 3 0 0 1 3 3v9a3 3 0 0 1-3 3H5a3 3 0 0 1-3-3V9Z" />
				<path d="M2 11h20" />
				<path d="M7 3v3" />
				<path d="M17 3v3" />
			</svg>
			<p class="text-sm" style="color: var(--color-text-tertiary)">
				{t('reservation.no_active')}
			</p>
			<button
				class="lg-btn-secondary squish mt-3 rounded-xl px-4 py-2 text-sm"
				onclick={() => goto('/search')}
			>
				{t('search.go_to_search')}
			</button>
		</GlassPanel>
	{:else}
		<!-- Reservation list -->
		<div class="flex flex-col gap-3">
			{#each reservations as res, i (res.provider + res.reservation_number)}
				{@const resKey = `${res.provider}:${res.reservation_number}`}
				<div class="lg-glass-card rounded-2xl overflow-hidden page-enter stagger-{Math.min(i + 1, 5)}">
					<!-- Main card -->
					<button
						type="button"
						class="w-full text-left px-4 py-3"
						onclick={() => toggleExpand(res)}
					>
						<!-- Header -->
						<div class="flex items-center gap-2 mb-1.5">
							<span
								class="shrink-0 inline-flex items-center px-1.5 py-0.5 rounded-md text-[10px] font-bold tracking-wider"
								class:lg-provider-srt={res.provider === 'SRT'}
								class:lg-provider-ktx={res.provider === 'KTX'}
							>
								{res.provider}
							</span>
							<span class="text-xs font-medium tabular-nums" style="color: var(--color-text-tertiary)">
								{res.train_name} {res.train_number}
							</span>
							<span class="flex-1"></span>
							<!-- Payment status badge -->
							{#if res.is_waiting}
								<span
									class="inline-flex items-center px-2 py-0.5 rounded-full text-[10px] font-medium"
									style="background: var(--color-status-warning-bg); color: var(--color-status-warning)"
								>
									{t('reservation.waiting')}
								</span>
							{:else if res.paid}
								<span
									class="inline-flex items-center px-2 py-0.5 rounded-full text-[10px] font-medium"
									style="background: var(--color-status-success-bg); color: var(--color-status-success)"
								>
									{t('reservation.paid')}
								</span>
							{:else}
								<span
									class="inline-flex items-center px-2 py-0.5 rounded-full text-[10px] font-medium"
									style="background: var(--color-status-error-bg); color: var(--color-status-error)"
								>
									{t('reservation.unpaid')}
								</span>
							{/if}
						</div>

						<!-- Route -->
						<p class="text-sm font-semibold" style="color: var(--color-text-primary)">
							{res.dep_station} → {res.arr_station}
						</p>

						<!-- Date/time/cost -->
						<div class="flex items-center gap-2 mt-1">
							<span class="text-xs tabular-nums" style="color: var(--color-text-tertiary)">
								{formatDate(res.dep_date)} · {formatTime(res.dep_time)} → {formatTime(res.arr_time)}
							</span>
							<span class="flex-1"></span>
							<span class="text-sm font-semibold tabular-nums" style="color: var(--color-text-primary)">
								{formatCost(res.total_cost)}
							</span>
						</div>

						<!-- Payment deadline if unpaid -->
						{#if !res.paid && !res.is_waiting}
							<p class="text-[10px] mt-1" style="color: var(--color-status-warning)">
								{formatDate(res.payment_deadline_date)} {formatTime(res.payment_deadline_time)}
							</p>
						{/if}
					</button>

					<!-- Expanded detail -->
					{#if expandedPnr === resKey}
						<div class="border-t px-4 py-3" style="border-color: var(--color-border-subtle)">
							<!-- Ticket details -->
							{#if ticketsLoading[resKey]}
								<Skeleton lines={2} />
							{:else if tickets[resKey] && tickets[resKey].length > 0}
								<p class="text-xs font-medium mb-2" style="color: var(--color-text-tertiary)">
									{t('reservation.view_tickets')}
								</p>
								<div class="flex flex-col gap-1.5 mb-3">
									{#each tickets[resKey] as ticket}
										<div
											class="flex items-center justify-between rounded-xl px-3 py-2"
											style="background: var(--color-bg-sunken)"
										>
											<div>
												<span class="text-xs font-medium" style="color: var(--color-text-primary)">
													{ticket.car} - {ticket.seat}
												</span>
												<span class="text-xs ml-2" style="color: var(--color-text-tertiary)">
													{ticket.seat_type} · {ticket.passenger_type}
												</span>
											</div>
											<span class="text-xs font-semibold tabular-nums" style="color: var(--color-text-primary)">
												{formatCost(String(ticket.price))}
											</span>
										</div>
									{/each}
								</div>
							{/if}

							<!-- Pay card selector -->
							{#if showPayCard === resKey}
								<div class="mb-3">
									{#if cards.length === 0}
										<p class="text-xs" style="color: var(--color-text-tertiary)">
											{t('search.no_cards')}
											<button
												type="button"
												class="font-medium underline"
												style="color: var(--color-brand-text)"
												onclick={() => goto('/settings')}
											>
												{t('search.add_card')}
											</button>
										</p>
									{:else}
										<p class="text-xs mb-2" style="color: var(--color-text-tertiary)">
											{t('search.select_card')}
										</p>
										<div class="flex flex-col gap-1.5 mb-2">
											{#each cards as card (card.id)}
												<button
													type="button"
													class="flex items-center gap-3 rounded-xl px-3 py-2 transition-all squish"
													class:lg-active={selectedPayCardId === card.id}
													style={selectedPayCardId === card.id
														? 'background: var(--color-brand-primary); border: 1px solid var(--color-brand-border)'
														: 'background: var(--color-bg-sunken); border: 1px solid var(--color-border-default)'}
													onclick={() => { selectedPayCardId = card.id; }}
												>
													<CardBrand cardType={card.card_type} lastFour={card.last_four} />
													<span class="text-xs" style="color: var(--color-text-secondary)">
														{card.label}
													</span>
												</button>
											{/each}
										</div>
										<button
											type="button"
											class="lg-btn-primary squish w-full rounded-xl py-2 text-sm"
											disabled={!selectedPayCardId || actionLoading[resKey] === 'pay'}
											onclick={() => handlePay(res.provider, res.reservation_number)}
										>
											{actionLoading[resKey] === 'pay' ? t('payment.paying') : t('payment.pay')}
										</button>
									{/if}
								</div>
							{/if}

							<!-- Action buttons -->
							<div class="flex gap-2 pt-2" style="border-top: 1px solid var(--color-border-subtle)">
								{#if !res.paid && !res.is_waiting}
									<button
										type="button"
										class="lg-btn-primary squish flex-1 rounded-xl py-2 text-sm"
										onclick={() => openPayCard(res.provider, res.reservation_number)}
									>
										{t('payment.pay')}
									</button>
								{/if}

								{#if res.paid}
									<button
										type="button"
										class="lg-btn-secondary squish flex-1 rounded-xl py-2 text-sm"
										disabled={actionLoading[resKey] === 'refund'}
										onclick={() => { confirmDialog = { pnr: res.reservation_number, provider: res.provider, action: 'refund' }; }}
									>
										{t('reservation.refund')}
									</button>
								{/if}

								<button
									type="button"
									class="lg-btn-danger squish flex-1 rounded-xl py-2 text-sm"
									disabled={actionLoading[resKey] === 'cancel'}
									onclick={() => { confirmDialog = { pnr: res.reservation_number, provider: res.provider, action: 'cancel' }; }}
								>
									{t('reservation.cancel')}
								</button>
							</div>
						</div>
					{/if}
				</div>
			{/each}
		</div>
	{/if}
</div>

<!-- Confirmation dialog -->
{#if confirmDialog}
	<button
		type="button"
		class="fixed inset-0 z-40 bg-black/40 backdrop-blur-sm"
		onclick={() => { confirmDialog = null; }}
		aria-label={t('common.close')}
	></button>

	<div class="fixed inset-0 z-50 flex items-center justify-center px-6">
		<div class="lg-glass-panel w-full max-w-sm p-6 modal-enter">
			<h3 class="text-lg font-bold mb-2" style="color: var(--color-text-primary)">
				{confirmDialog.action === 'cancel' ? t('reservation.cancel') : t('reservation.refund')}
			</h3>
			<p class="text-sm mb-5" style="color: var(--color-text-tertiary)">
				{t('task.cancel_description')}
			</p>
			<div class="flex gap-3">
				<button
					type="button"
					class="lg-btn-secondary squish flex-1 rounded-xl py-2.5 text-sm"
					onclick={() => { confirmDialog = null; }}
				>
					{t('common.cancel')}
				</button>
				<button
					type="button"
					class="squish flex-1 rounded-xl py-2.5 text-sm font-semibold"
					style="background: var(--color-status-error-bg); color: var(--color-status-error)"
					onclick={handleConfirmAction}
				>
					{t('common.confirm')}
				</button>
			</div>
		</div>
	</div>
{/if}
