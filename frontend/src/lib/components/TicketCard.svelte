<script lang="ts">
	import type { ReservationInfo } from '$lib/types';
	import { t } from '$lib/i18n';
	import { formatTime, formatDate, formatCost } from '$lib/utils';

	interface Props {
		reservation: ReservationInfo;
	}

	const { reservation }: Props = $props();

	let detailOpen = $state(false);

	const providerClass = $derived(
		reservation.provider.toUpperCase() === 'SRT' ? 'lg-provider-srt' : 'lg-provider-ktx'
	);

	function paymentStatus(): { label: string; variant: string } {
		if (reservation.is_waiting) {
			return { label: t('reservation.waiting'), variant: 'warning' };
		}
		if (reservation.paid) {
			return { label: t('reservation.paid'), variant: 'success' };
		}
		return { label: t('reservation.unpaid'), variant: 'error' };
	}

	const status = $derived(paymentStatus());
</script>

<div class="lg-glass-card overflow-hidden">
	<button
		type="button"
		class="w-full text-left p-4"
		onclick={() => detailOpen = !detailOpen}
		aria-expanded={detailOpen}
	>
		<!-- Top row: provider badge + train number + payment status -->
		<div class="flex items-center justify-between mb-2">
			<div class="flex items-center gap-2">
				<span
					class="inline-flex items-center px-2 py-0.5 rounded-lg text-[11px] font-bold tracking-tight {providerClass}"
				>
					{reservation.provider}
				</span>
				<span class="text-xs font-medium" style="color: var(--color-text-secondary)">
					{reservation.train_name} {reservation.train_number}
				</span>
			</div>
			<span
				class="inline-flex items-center px-2 py-0.5 rounded-full text-[10px] font-medium lg-badge-{status.variant}"
			>
				{status.label}
			</span>
		</div>

		<!-- Route -->
		<div class="flex items-center gap-2 mb-1">
			<span class="text-base font-semibold" style="color: var(--color-text-primary)">
				{reservation.dep_station}
			</span>
			<svg class="w-4 h-4 shrink-0" style="color: var(--color-text-disabled)" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
				<line x1="5" y1="12" x2="19" y2="12" />
				<polyline points="12 5 19 12 12 19" />
			</svg>
			<span class="text-base font-semibold" style="color: var(--color-text-primary)">
				{reservation.arr_station}
			</span>
		</div>

		<!-- Date + Time -->
		<div class="flex items-center gap-3 text-xs" style="color: var(--color-text-tertiary)">
			<span>{formatDate(reservation.dep_date)}</span>
			<span>{formatTime(reservation.dep_time)} &rarr; {formatTime(reservation.arr_time)}</span>
		</div>
	</button>

	<!-- Collapsible details -->
	{#if detailOpen}
		<div class="border-t px-4 pb-4 pt-3 page-enter" style="border-color: var(--color-border-subtle)">
			<dl class="grid grid-cols-2 gap-y-2 gap-x-4 text-xs">
				<div>
					<dt style="color: var(--color-text-tertiary)">{t('reservation.title')}</dt>
					<dd class="font-medium" style="color: var(--color-text-primary)">
						{reservation.reservation_number}
					</dd>
				</div>

				<div>
					<dt style="color: var(--color-text-tertiary)">{t('task.total')}</dt>
					<dd class="font-semibold" style="color: var(--color-text-primary)">
						&#8361;{formatCost(reservation.total_cost)}
					</dd>
				</div>

				<div>
					<dt style="color: var(--color-text-tertiary)">{t('search.passengers')}</dt>
					<dd class="font-medium" style="color: var(--color-text-primary)">
						{reservation.seat_count}
					</dd>
				</div>

				{#if !reservation.paid && !reservation.is_waiting}
					<div>
						<dt style="color: var(--color-text-tertiary)">{t('task.pay_fare')}</dt>
						<dd class="font-medium" style="color: var(--color-status-warning)">
							{formatDate(reservation.payment_deadline_date)} {formatTime(reservation.payment_deadline_time)}
						</dd>
					</div>
				{/if}
			</dl>
		</div>
	{/if}
</div>
