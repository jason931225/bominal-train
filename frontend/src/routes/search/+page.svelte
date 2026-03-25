<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { t } from '$lib/i18n';
	import { searchTrains } from '$lib/api/search';
	import { listCards } from '$lib/api/cards';
	import { createTask } from '$lib/api/tasks';
	import type {
		TrainInfo,
		CardInfo,
		PassengerCount,
		PassengerKind,
		SeatPreference,
		Provider,
		TargetTrain,
		CreateTaskInput
	} from '$lib/types';
	import { formatTime, slotToTimeString, formatTimeSlot } from '$lib/utils';
	import GlassPanel from '$lib/components/GlassPanel.svelte';
	import TimeSlider from '$lib/components/TimeSlider.svelte';
	import SortableList from '$lib/components/SortableList.svelte';
	import CardBrand from '$lib/components/CardBrand.svelte';
	import Skeleton from '$lib/components/Skeleton.svelte';
	import StationInput from '$lib/components/StationInput.svelte';

	/* ── Types ── */
	interface SelectedTrain {
		provider: Provider;
		train_number: string;
		dep_time: string;
	}

	/* ── Search form state ── */
	let departure = $state('');
	let arrival = $state('');
	let date = $state('');
	let timeSlot = $state(12);
	let providerFilter = $state<'Both' | 'SRT' | 'KTX'>('Both');

	/* ── Passengers ── */
	const passengerKinds: { type: PassengerKind; labelKey: string }[] = [
		{ type: 'adult', labelKey: 'passenger.adult' },
		{ type: 'child', labelKey: 'passenger.child' },
		{ type: 'senior', labelKey: 'passenger.senior' },
		{ type: 'severe', labelKey: 'passenger.severe' },
		{ type: 'mild', labelKey: 'passenger.mild' },
		{ type: 'infant', labelKey: 'passenger.infant' }
	];

	let passengers = $state<PassengerCount[]>([{ type: 'adult', count: 1 }]);

	const totalPassengers = $derived(passengers.reduce((sum, p) => sum + p.count, 0));

	/* ── Results & selection ── */
	let results = $state<TrainInfo[]>([]);
	let searching = $state(false);
	let searchError = $state('');
	let hasSearched = $state(false);

	let selected = $state<SelectedTrain[]>([]);

	/* ── Review modal ── */
	let showReview = $state(false);
	let seatPref = $state<SeatPreference>('GeneralFirst');
	let autoPay = $state(false);
	let selectedCardId = $state<string | null>(null);
	let cards = $state<CardInfo[]>([]);
	let creatingTask = $state(false);

	const seatOptions: { value: SeatPreference; labelKey: string }[] = [
		{ value: 'GeneralFirst', labelKey: 'search.seat_general_first' },
		{ value: 'SpecialFirst', labelKey: 'search.seat_special_first' },
		{ value: 'GeneralOnly', labelKey: 'search.seat_general_only' },
		{ value: 'SpecialOnly', labelKey: 'search.seat_special_only' }
	];

	/* ── Circled number badges ── */
	const circledNumbers = ['\u2460', '\u2461', '\u2462', '\u2463', '\u2464', '\u2465', '\u2466', '\u2467', '\u2468', '\u2469'];

	function selectionIndex(train: TrainInfo): number {
		return selected.findIndex(
			(s) =>
				s.provider === train.provider &&
				s.train_number === train.train_number &&
				s.dep_time === train.dep_time
		);
	}

	function swapStations(): void {
		const temp = departure;
		departure = arrival;
		arrival = temp;
	}

	/* ── Passenger controls ── */
	function adjustPassenger(kind: PassengerKind, delta: number): void {
		const existing = passengers.find((p) => p.type === kind);
		if (existing) {
			const newCount = existing.count + delta;
			if (newCount <= 0) {
				passengers = passengers.filter((p) => p.type !== kind);
			} else if (totalPassengers + delta <= 9) {
				passengers = passengers.map((p) =>
					p.type === kind ? { ...p, count: newCount } : p
				);
			}
		} else if (delta > 0 && totalPassengers < 9) {
			passengers = [...passengers, { type: kind, count: 1 }];
		}
	}

	function getPassengerCount(kind: PassengerKind): number {
		return passengers.find((p) => p.type === kind)?.count ?? 0;
	}

	/* ── Search ── */
	async function handleSearch(): Promise<void> {
		if (!departure || !arrival) return;
		searching = true;
		searchError = '';
		hasSearched = true;
		selected = [];

		const timeStr = slotToTimeString(timeSlot);

		try {
			if (providerFilter === 'Both') {
				const [srtResults, ktxResults] = await Promise.all([
					searchTrains('SRT', departure, arrival, date || undefined, timeStr),
					searchTrains('KTX', departure, arrival, date || undefined, timeStr)
				]);
				const merged = [...srtResults, ...ktxResults];
				results = merged.sort((a, b) => a.dep_time.localeCompare(b.dep_time));
			} else {
				results = await searchTrains(
					providerFilter,
					departure,
					arrival,
					date || undefined,
					timeStr
				);
			}
		} catch (err) {
			searchError = err instanceof Error ? err.message : t('error.load_failed');
			results = [];
		} finally {
			searching = false;
		}
	}

	/* ── Selection ── */
	function toggleTrain(train: TrainInfo): void {
		const idx = selectionIndex(train);
		if (idx >= 0) {
			selected = selected.filter((_, i) => i !== idx);
		} else {
			selected = [
				...selected,
				{
					provider: train.provider as Provider,
					train_number: train.train_number,
					dep_time: train.dep_time
				}
			];
		}
	}

	/* ── Review modal ── */
	async function openReview(): Promise<void> {
		showReview = true;
		try {
			cards = await listCards();
			if (cards.length > 0 && selectedCardId === null) {
				selectedCardId = cards[0].id;
			}
		} catch {
			cards = [];
		}
	}

	function closeReview(): void {
		showReview = false;
	}

	function handleReorder(items: { id: string; label: string }[]): void {
		selected = items.map((item) => {
			const parts = item.id.split('::');
			return {
				provider: parts[0] as Provider,
				train_number: parts[1],
				dep_time: parts[2]
			};
		});
	}

	function handleRemove(id: string): void {
		const parts = id.split('::');
		selected = selected.filter(
			(s) =>
				!(s.provider === parts[0] && s.train_number === parts[1] && s.dep_time === parts[2])
		);
		if (selected.length === 0) {
			showReview = false;
		}
	}

	const sortableItems = $derived(
		selected.map((s) => ({
			id: `${s.provider}::${s.train_number}::${s.dep_time}`,
			label: `${s.provider} ${s.train_number} ${formatTime(s.dep_time)}`
		}))
	);

	async function handleCreateTask(): Promise<void> {
		if (selected.length === 0) return;
		creatingTask = true;

		const targetTrains: TargetTrain[] = selected.map((s) => ({
			provider: s.provider,
			train_number: s.train_number,
			dep_time: s.dep_time
		}));

		const input: CreateTaskInput = {
			provider: providerFilter === 'Both' ? undefined : providerFilter,
			departure_station: departure,
			arrival_station: arrival,
			travel_date: date,
			departure_time: slotToTimeString(timeSlot),
			passengers,
			seat_preference: seatPref,
			target_trains: targetTrains,
			auto_pay: autoPay,
			payment_card_id: autoPay ? selectedCardId : null,
			notify_enabled: true,
			auto_retry: true
		};

		try {
			await createTask(input);
			goto('/tasks');
		} catch (err) {
			searchError = err instanceof Error ? err.message : t('error.unexpected');
		} finally {
			creatingTask = false;
		}
	}

	/* ── Date default ── */
	onMount(() => {
		const now = new Date();
		const y = now.getFullYear();
		const m = String(now.getMonth() + 1).padStart(2, '0');
		const d = String(now.getDate()).padStart(2, '0');
		date = `${y}${m}${d}`;
	});
</script>

<div class="page-container">
	<!-- Title -->
	<h1 class="text-xl font-bold mb-5 page-enter" style="color: var(--color-text-primary)">
		{t('search.title')}
	</h1>

	<!-- Station inputs -->
	<GlassPanel class="mb-4 page-enter stagger-1">
		<div class="flex items-center gap-2">
			<StationInput
				bind:value={departure}
				label={t('search.from')}
				placeholder={t('search.select_station')}
				provider={providerFilter === 'Both' ? 'SRT' : providerFilter}
				name="dep"
			/>

			<!-- Swap button -->
			<button
				type="button"
				class="shrink-0 mt-4 p-2 rounded-xl squish"
				style="background: var(--color-bg-sunken)"
				onclick={swapStations}
				aria-label={t('search.swap_stations')}
			>
				<svg
					class="w-5 h-5"
					style="color: var(--color-brand-text)"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
				>
					<polyline points="7 16 3 12 7 8" />
					<line x1="21" y1="12" x2="3" y2="12" />
					<polyline points="17 8 21 12 17 16" />
				</svg>
			</button>

			<StationInput
				bind:value={arrival}
				label={t('search.to')}
				placeholder={t('search.select_station')}
				provider={providerFilter === 'Both' ? 'SRT' : providerFilter}
				name="arr"
			/>
		</div>
	</GlassPanel>

	<!-- Date & Time -->
	<GlassPanel class="mb-4 page-enter stagger-2">
		<div class="flex items-center gap-3 mb-4">
			<div class="flex-1">
				<label class="text-xs font-medium mb-1 block" style="color: var(--color-text-tertiary)">
					{t('search.date')}
				</label>
				<input
					type="text"
					class="w-full rounded-xl px-3 py-2.5 text-sm font-medium outline-none tabular-nums"
					style="background: var(--color-bg-sunken); color: var(--color-text-primary); border: 1px solid var(--color-border-default)"
					placeholder="YYYYMMDD"
					maxlength={8}
					bind:value={date}
				/>
			</div>
		</div>

		<div>
			<label class="text-xs font-medium mb-2 block" style="color: var(--color-text-tertiary)">
				{t('search.time')} — {formatTimeSlot(timeSlot)}
			</label>
			<TimeSlider value={timeSlot} onchange={(v) => { timeSlot = v; }} />
		</div>
	</GlassPanel>

	<!-- Passengers -->
	<GlassPanel class="mb-4 page-enter stagger-3">
		<div class="flex items-center justify-between mb-3">
			<label class="text-xs font-medium" style="color: var(--color-text-tertiary)">
				{t('search.passengers')}
			</label>
			<span class="text-xs font-semibold tabular-nums" style="color: var(--color-brand-text)">
				{totalPassengers} {t('search.passenger_count')}
			</span>
		</div>
		<div class="grid grid-cols-2 gap-2">
			{#each passengerKinds as kind}
				{@const count = getPassengerCount(kind.type)}
				<div
					class="flex items-center justify-between rounded-xl px-3 py-2"
					style="background: var(--color-bg-sunken)"
				>
					<span class="text-sm" style="color: var(--color-text-primary)">{t(kind.labelKey)}</span>
					<div class="flex items-center gap-2">
						<button
							type="button"
							class="w-7 h-7 flex items-center justify-center rounded-lg text-sm font-bold squish disabled:opacity-30"
							style="background: var(--color-brand-primary); color: var(--color-brand-text)"
							disabled={count === 0}
							onclick={() => adjustPassenger(kind.type, -1)}
							aria-label={t('passenger.decrease')}
						>
							-
						</button>
						<span class="text-sm font-semibold tabular-nums w-4 text-center" style="color: var(--color-text-primary)">
							{count}
						</span>
						<button
							type="button"
							class="w-7 h-7 flex items-center justify-center rounded-lg text-sm font-bold squish disabled:opacity-30"
							style="background: var(--color-brand-primary); color: var(--color-brand-text)"
							disabled={totalPassengers >= 9}
							onclick={() => adjustPassenger(kind.type, 1)}
							aria-label={t('passenger.increase')}
						>
							+
						</button>
					</div>
				</div>
			{/each}
		</div>
	</GlassPanel>

	<!-- Provider filter -->
	<GlassPanel class="mb-4 page-enter stagger-4">
		<label class="text-xs font-medium mb-2 block" style="color: var(--color-text-tertiary)">
			{t('search.provider')}
		</label>
		<div class="flex rounded-xl overflow-hidden" style="background: var(--color-bg-sunken)">
			{#each (['Both', 'SRT', 'KTX'] as const) as option}
				<button
					type="button"
					class="flex-1 py-2.5 text-sm font-medium transition-all rounded-xl squish"
					class:glass-active={providerFilter === option}
					style={providerFilter === option
						? `color: var(--color-brand-text)`
						: `color: var(--color-text-secondary)`}
					onclick={() => { providerFilter = option; }}
				>
					{option === 'Both' ? 'Both' : option}
				</button>
			{/each}
		</div>
	</GlassPanel>

	<!-- Search button -->
	<button
		class="btn-primary squish w-full rounded-2xl px-6 py-3.5 text-base mb-6 page-enter stagger-5"
		disabled={searching || !departure || !arrival || !date}
		onclick={handleSearch}
	>
		{#if searching}
			{t('search.searching')}
		{:else}
			{t('search.search_btn')}
		{/if}
	</button>

	<!-- Search error -->
	{#if searchError}
		<GlassPanel class="mb-4 text-center">
			<p class="text-sm" style="color: var(--color-status-error)">{searchError}</p>
		</GlassPanel>
	{/if}

	<!-- Results list -->
	{#if searching}
		<GlassPanel>
			<Skeleton lines={6} />
		</GlassPanel>
	{:else if hasSearched && results.length === 0 && !searchError}
		<GlassPanel class="text-center">
			<p class="text-sm" style="color: var(--color-text-tertiary)">{t('search.no_results')}</p>
		</GlassPanel>
	{:else if results.length > 0}
		<div class="flex flex-col gap-2 mb-24">
			{#if hasSearched && !searching}
				<p class="text-xs mb-1" style="color: var(--color-text-tertiary)">
					{t('search.tap_to_select')}
				</p>
			{/if}
			{#each results as train, i (train.provider + train.train_number + train.dep_time)}
				{@const idx = selectionIndex(train)}
				<button
					type="button"
					class="glass-card squish flex items-center gap-3 rounded-2xl px-4 py-3 text-left transition-all page-enter stagger-{Math.min(i + 1, 5)}"
					class:glass-active={idx >= 0}
					onclick={() => toggleTrain(train)}
				>
					<!-- Priority badge -->
					{#if idx >= 0}
						<span
							class="shrink-0 w-7 h-7 flex items-center justify-center rounded-full text-sm font-bold"
							style="background: var(--color-brand-primary); color: var(--color-brand-text)"
						>
							{circledNumbers[idx] ?? idx + 1}
						</span>
					{/if}

					<!-- Provider badge -->
					<span
						class="shrink-0 inline-flex items-center px-1.5 py-0.5 rounded-md text-[10px] font-bold tracking-wider"
						class:bg-[rgba(255,42,85,0.12)]={train.provider === 'SRT'}
						class:text-[#FF2A54]={train.provider === 'SRT'}
						class:bg-[rgba(0,122,255,0.12)]={train.provider === 'KTX'}
						class:text-[#007AFF]={train.provider === 'KTX'}
					>
						{train.provider}
					</span>

					<!-- Train number -->
					<span class="text-sm font-semibold tabular-nums" style="color: var(--color-text-primary)">
						{train.train_number}
					</span>

					<!-- Times -->
					<span class="flex-1 text-sm tabular-nums" style="color: var(--color-text-secondary)">
						{formatTime(train.dep_time)} → {formatTime(train.arr_time)}
					</span>

					<!-- Availability -->
					<div class="flex items-center gap-1.5 shrink-0">
						{#if train.general_available}
							<span class="text-[10px] font-medium px-1.5 py-0.5 rounded-md" style="background: var(--color-status-success-bg); color: var(--color-status-success)">
								{t('seat.general')}
							</span>
						{/if}
						{#if train.special_available}
							<span class="text-[10px] font-medium px-1.5 py-0.5 rounded-md" style="background: var(--color-status-success-bg); color: var(--color-status-success)">
								{t('seat.special')}
							</span>
						{/if}
					</div>
				</button>
			{/each}
		</div>
	{/if}

	<!-- Selection floating bar -->
	{#if selected.length > 0 && !showReview}
		<div class="fixed bottom-20 left-0 right-0 z-30 px-4 md:bottom-4">
			<div class="glass-panel mx-auto flex items-center justify-between gap-3 px-5 py-3.5 sheet-enter" style="max-width: 48rem">
				<span class="text-sm font-semibold" style="color: var(--color-text-primary)">
					{selected.length} {t('selection.selected_count')}
				</span>
				<div class="flex items-center gap-2">
					<button
						type="button"
						class="btn-glass squish rounded-xl px-4 py-2 text-sm"
						onclick={() => { selected = []; }}
					>
						{t('selection.clear')}
					</button>
					<button
						type="button"
						class="btn-primary squish rounded-xl px-5 py-2 text-sm"
						onclick={openReview}
					>
						{t('selection.review')}
					</button>
				</div>
			</div>
		</div>
	{/if}

	<!-- Review bottom sheet -->
	{#if showReview}
		<!-- Backdrop -->
		<button
			type="button"
			class="fixed inset-0 z-40 bg-black/40 backdrop-blur-sm"
			onclick={closeReview}
			aria-label={t('common.close')}
		></button>

		<div class="fixed inset-x-0 bottom-0 z-50 sheet-enter">
			<div class="glass-panel mx-auto rounded-t-3xl px-5 pt-5 pb-8 safe-area-pb" style="max-width: 48rem">
				<!-- Handle -->
				<div class="flex justify-center mb-4">
					<div class="w-10 h-1 rounded-full" style="background: var(--color-border-default)"></div>
				</div>

				<h2 class="text-lg font-bold mb-4" style="color: var(--color-text-primary)">
					{t('review.title')}
				</h2>

				<!-- Priority order -->
				<div class="mb-5">
					<p class="text-xs font-medium mb-2" style="color: var(--color-text-tertiary)">
						{t('review.priority_order')} — {t('review.reorder_hint')}
					</p>
					<SortableList
						items={sortableItems}
						onReorder={handleReorder}
						onRemove={handleRemove}
					/>
				</div>

				<!-- Seat preference -->
				<div class="mb-5">
					<label class="text-xs font-medium mb-2 block" style="color: var(--color-text-tertiary)">
						{t('search.seat_preference')}
					</label>
					<div class="grid grid-cols-2 gap-2">
						{#each seatOptions as option}
							<button
								type="button"
								class="rounded-xl px-3 py-2.5 text-sm font-medium transition-all squish"
								class:glass-active={seatPref === option.value}
								style={seatPref === option.value
									? 'background: var(--color-brand-primary); color: var(--color-brand-text); border: 1px solid var(--color-brand-border)'
									: 'background: var(--color-bg-sunken); color: var(--color-text-secondary); border: 1px solid transparent'}
								onclick={() => { seatPref = option.value; }}
							>
								{t(option.labelKey)}
							</button>
						{/each}
					</div>
				</div>

				<!-- Auto-pay + card selector -->
				<div class="mb-6">
					<div class="flex items-center justify-between mb-2">
						<label class="text-sm font-medium" style="color: var(--color-text-primary)">
							{t('search.auto_pay')}
						</label>
						<button
							type="button"
							class="relative w-11 h-6 rounded-full transition-colors"
							style={autoPay
								? 'background: var(--color-brand-text)'
								: 'background: var(--color-bg-sunken); border: 1px solid var(--color-border-default)'}
							onclick={() => { autoPay = !autoPay; }}
							role="switch"
							aria-checked={autoPay}
						>
							<span
								class="absolute top-0.5 left-0.5 w-5 h-5 rounded-full bg-white shadow transition-transform"
								style={autoPay ? 'transform: translateX(1.25rem)' : ''}
							></span>
						</button>
					</div>

					{#if autoPay}
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
							<div class="flex flex-col gap-2">
								{#each cards as card (card.id)}
									<button
										type="button"
										class="flex items-center gap-3 rounded-xl px-3 py-2.5 transition-all squish"
										class:glass-active={selectedCardId === card.id}
										style={selectedCardId === card.id
											? 'background: var(--color-brand-primary); border: 1px solid var(--color-brand-border)'
											: 'background: var(--color-bg-sunken); border: 1px solid var(--color-border-default)'}
										onclick={() => { selectedCardId = card.id; }}
									>
										<CardBrand cardType={card.card_type} lastFour={card.last_four} />
										<span class="text-sm" style="color: var(--color-text-secondary)">
											{card.label}
										</span>
									</button>
								{/each}
							</div>
						{/if}
					{/if}

					{#if autoPay && cards.length > 0 && selectedCardId === null}
						<p class="text-xs mt-1" style="color: var(--color-status-warning)">
							{t('search.auto_pay_card_required')}
						</p>
					{/if}
				</div>

				<!-- Start task button -->
				<button
					class="btn-primary squish w-full rounded-2xl px-6 py-3.5 text-base"
					disabled={creatingTask || selected.length === 0 || (autoPay && !selectedCardId)}
					onclick={handleCreateTask}
				>
					{#if creatingTask}
						{t('search.creating_task')}
					{:else}
						{t('search.create_task')}
					{/if}
				</button>
			</div>
		</div>
	{/if}
</div>
