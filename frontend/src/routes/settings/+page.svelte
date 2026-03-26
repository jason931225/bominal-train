<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { t, localeOptions, switchLocale, getCurrentLocale } from '$lib/i18n';
	import type { Locale } from '$lib/i18n';
	import { themeStore } from '$lib/stores/theme.svelte';
	import type { ThemeName, ThemeMode } from '$lib/stores/theme.svelte';
	import { auth } from '$lib/stores/auth.svelte';
	import { logout } from '$lib/api/auth';
	import { listProviders, addProvider, deleteProvider } from '$lib/api/providers';
	import { listCards, deleteCard } from '$lib/api/cards';
	import type { ProviderInfo, CardInfo } from '$lib/types';
	import GlassPanel from '$lib/components/GlassPanel.svelte';
	import CardBrand from '$lib/components/CardBrand.svelte';
	import StatusChip from '$lib/components/StatusChip.svelte';
	import Skeleton from '$lib/components/Skeleton.svelte';

	/* ── Provider state ── */
	let providers = $state<ProviderInfo[]>([]);
	let providersLoading = $state(true);
	let providerError = $state('');

	let showAddProvider = $state(false);
	let addProviderType = $state('SRT');
	let addLoginId = $state('');
	let addPassword = $state('');
	let addingProvider = $state(false);

	let deleteProviderTarget = $state<string | null>(null);
	let deletingProvider = $state(false);

	/* ── Cards state ── */
	let cards = $state<CardInfo[]>([]);
	let cardsLoading = $state(true);
	let cardError = $state('');

	let showAddCard = $state(false);
	let cardLabel = $state('');
	let cardNumber = $state('');
	let cardPassword = $state('');
	let cardBirthday = $state('');
	let cardExpiry = $state('');
	let cardType = $state<'credit' | 'debit'>('credit');
	let addingCard = $state(false);

	let deleteCardTarget = $state<string | null>(null);
	let deletingCard = $state(false);

	/* ── Appearance ── */
	let currentLocale = $state<Locale>(getCurrentLocale());

	/* ── Logout ── */
	let loggingOut = $state(false);

	/* ── Provider status mapping ── */
	function providerStatusVariant(status: string): 'success' | 'error' | 'warning' | 'neutral' {
		switch (status) {
			case 'valid':
				return 'success';
			case 'invalid':
				return 'error';
			case 'unverified':
				return 'warning';
			default:
				return 'neutral';
		}
	}

	function providerStatusLabel(status: string): string {
		switch (status) {
			case 'valid':
				return t('provider.status_valid');
			case 'invalid':
				return t('provider.status_invalid');
			case 'unverified':
				return t('provider.status_unverified');
			default:
				return t('provider.status_disabled');
		}
	}

	/* ── Data fetching ── */
	async function fetchProviders(): Promise<void> {
		try {
			providers = await listProviders();
			providerError = '';
		} catch (err) {
			providerError = err instanceof Error ? err.message : t('error.load_failed');
		} finally {
			providersLoading = false;
		}
	}

	async function fetchCards(): Promise<void> {
		try {
			cards = await listCards();
			cardError = '';
		} catch (err) {
			cardError = err instanceof Error ? err.message : t('error.load_failed');
		} finally {
			cardsLoading = false;
		}
	}

	/* ── Provider actions ── */
	async function handleAddProvider(): Promise<void> {
		if (!addLoginId || !addPassword) return;
		addingProvider = true;
		try {
			await addProvider(addProviderType, addLoginId, addPassword);
			showAddProvider = false;
			addLoginId = '';
			addPassword = '';
			await fetchProviders();
		} catch (err) {
			providerError = err instanceof Error ? err.message : t('error.unexpected');
		} finally {
			addingProvider = false;
		}
	}

	async function handleDeleteProvider(): Promise<void> {
		if (!deleteProviderTarget) return;
		deletingProvider = true;
		try {
			await deleteProvider(deleteProviderTarget);
			deleteProviderTarget = null;
			await fetchProviders();
		} catch (err) {
			providerError = err instanceof Error ? err.message : t('error.unexpected');
		} finally {
			deletingProvider = false;
		}
	}

	/* ── Card actions ── */
	async function handleAddCard(): Promise<void> {
		if (!cardNumber || !cardPassword || !cardBirthday || !cardExpiry) return;
		addingCard = true;
		try {
			// Call interop's submitCard via global
			const submitCard = (window as any).BominalInterop?.submitCard;
			if (submitCard) {
				await submitCard({
					label: cardLabel,
					number: cardNumber,
					password: cardPassword,
					birthday: cardBirthday,
					expiry: cardExpiry,
					type: cardType
				});
			}
			showAddCard = false;
			cardLabel = '';
			cardNumber = '';
			cardPassword = '';
			cardBirthday = '';
			cardExpiry = '';
			await fetchCards();
		} catch (err) {
			cardError = err instanceof Error ? err.message : t('error.unexpected');
		} finally {
			addingCard = false;
		}
	}

	async function handleDeleteCard(): Promise<void> {
		if (!deleteCardTarget) return;
		deletingCard = true;
		try {
			await deleteCard(deleteCardTarget);
			deleteCardTarget = null;
			await fetchCards();
		} catch (err) {
			cardError = err instanceof Error ? err.message : t('error.unexpected');
		} finally {
			deletingCard = false;
		}
	}

	/* ── Appearance ── */
	function handleThemeChange(themeName: ThemeName): void {
		themeStore.setTheme(themeName);
	}

	function handleModeChange(mode: ThemeMode): void {
		themeStore.setMode(mode);
	}

	function handleLocaleChange(locale: Locale): void {
		currentLocale = locale;
		switchLocale(locale);
	}

	/* ── Logout ── */
	async function handleLogout(): Promise<void> {
		loggingOut = true;
		try {
			await logout();
			auth.clear();
			goto('/auth');
		} catch {
			// Best-effort: clear local state and redirect
			auth.clear();
			goto('/auth');
		}
	}

	onMount(() => {
		fetchProviders();
		fetchCards();
	});
</script>

<svelte:head><title>{t('settings.title')} | Bominal</title></svelte:head>

<div class="page-container page-enter">
	<h1 class="text-xl font-bold mb-6" style="color: var(--color-text-primary)">
		{t('settings.title')}
	</h1>

	<!-- Provider Credentials -->
	<section class="mb-6 page-enter stagger-1">
		<h2 class="text-sm font-semibold mb-3" style="color: var(--color-text-tertiary)">
			{t('settings.section_provider')}
		</h2>

		{#if providersLoading}
			<GlassPanel>
				<Skeleton lines={3} />
			</GlassPanel>
		{:else if providerError}
			<GlassPanel class="text-center">
				<p class="text-sm" style="color: var(--color-status-error)">{providerError}</p>
				<button class="lg-btn-secondary squish mt-2 rounded-xl px-4 py-2 text-sm" onclick={fetchProviders}>
					{t('common.retry')}
				</button>
			</GlassPanel>
		{:else}
			<GlassPanel>
				{#if providers.length > 0}
					<div class="flex flex-col gap-3">
						{#each providers as prov (prov.provider)}
							<div class="flex items-center justify-between">
								<div class="flex items-center gap-2">
									<span class="text-sm font-medium" style="color: var(--color-text-primary)">
										{prov.provider === 'SRT' ? t('provider.srt') : t('provider.ktx')}
									</span>
									<StatusChip
										label={providerStatusLabel(prov.status)}
										variant={providerStatusVariant(prov.status)}
									/>
								</div>
								<div class="flex items-center gap-2">
									<span class="text-xs" style="color: var(--color-text-tertiary)">
										{prov.login_id}
									</span>
									<button
										type="button"
										class="p-1.5 rounded-lg transition-colors hover:bg-[var(--color-status-error-bg)]"
										style="color: var(--color-status-error)"
										onclick={() => { deleteProviderTarget = prov.provider; }}
										aria-label={t('provider.remove')}
									>
										<svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
											<polyline points="3 6 5 6 21 6" />
											<path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
										</svg>
									</button>
								</div>
							</div>
						{/each}
					</div>
				{:else}
					<p class="text-sm text-center" style="color: var(--color-text-tertiary)">
						{t('provider.not_configured')}
					</p>
				{/if}

				<!-- Add provider toggle -->
				{#if !showAddProvider}
					<button
						type="button"
						class="lg-btn-secondary squish w-full mt-4 rounded-xl px-4 py-2.5 text-sm"
						onclick={() => { showAddProvider = true; }}
					>
						+ {t('provider.setup')}
					</button>
				{/if}

				<!-- Add provider form -->
				{#if showAddProvider}
					<div class="mt-4 pt-4" style="border-top: 1px solid var(--color-border-subtle)">
						<!-- Provider type toggle -->
						<div class="flex rounded-xl overflow-hidden mb-3" style="background: var(--color-bg-sunken)">
							{#each (['SRT', 'KTX'] as const) as pType}
								<button
									type="button"
									class="flex-1 py-2 text-sm font-medium transition-all rounded-xl squish"
									class:lg-active={addProviderType === pType}
									style={addProviderType === pType
										? 'color: var(--color-brand-text)'
										: 'color: var(--color-text-secondary)'}
									onclick={() => { addProviderType = pType; }}
								>
									{pType}
								</button>
							{/each}
						</div>

						<input
							type="text"
							class="w-full rounded-xl px-3 py-2.5 text-sm outline-none mb-2"
							style="background: var(--color-bg-sunken); color: var(--color-text-primary); border: 1px solid var(--color-border-default)"
							placeholder={t('provider.login_id')}
							bind:value={addLoginId}
						/>
						<input
							type="password"
							class="w-full rounded-xl px-3 py-2.5 text-sm outline-none mb-3"
							style="background: var(--color-bg-sunken); color: var(--color-text-primary); border: 1px solid var(--color-border-default)"
							placeholder={t('provider.password')}
							bind:value={addPassword}
						/>

						<div class="flex gap-2">
							<button
								type="button"
								class="lg-btn-secondary squish flex-1 rounded-xl py-2.5 text-sm"
								onclick={() => { showAddProvider = false; addLoginId = ''; addPassword = ''; }}
							>
								{t('common.cancel')}
							</button>
							<button
								type="button"
								class="lg-btn-primary squish flex-1 rounded-xl py-2.5 text-sm"
								disabled={addingProvider || !addLoginId || !addPassword}
								onclick={handleAddProvider}
							>
								{addingProvider ? t('provider.verifying') : t('provider.verify_save')}
							</button>
						</div>
					</div>
				{/if}
			</GlassPanel>
		{/if}
	</section>

	<!-- Payment Cards -->
	<section class="mb-6 page-enter stagger-2">
		<h2 class="text-sm font-semibold mb-3" style="color: var(--color-text-tertiary)">
			{t('settings.section_payment')}
		</h2>

		{#if cardsLoading}
			<GlassPanel>
				<Skeleton lines={3} />
			</GlassPanel>
		{:else if cardError}
			<GlassPanel class="text-center">
				<p class="text-sm" style="color: var(--color-status-error)">{cardError}</p>
				<button class="lg-btn-secondary squish mt-2 rounded-xl px-4 py-2 text-sm" onclick={fetchCards}>
					{t('common.retry')}
				</button>
			</GlassPanel>
		{:else}
			<GlassPanel>
				{#if cards.length > 0}
					<div class="flex flex-col gap-3">
						{#each cards as card (card.id)}
							<div class="flex items-center justify-between">
								<div class="flex items-center gap-2">
									<CardBrand cardType={card.card_type} lastFour={card.last_four} />
									<span class="text-sm" style="color: var(--color-text-secondary)">
										{card.label}
									</span>
								</div>
								<button
									type="button"
									class="p-1.5 rounded-lg transition-colors hover:bg-[var(--color-status-error-bg)]"
									style="color: var(--color-status-error)"
									onclick={() => { deleteCardTarget = card.id; }}
									aria-label={t('common.delete')}
								>
									<svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
										<polyline points="3 6 5 6 21 6" />
										<path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
									</svg>
								</button>
							</div>
						{/each}
					</div>
				{:else}
					<p class="text-sm text-center" style="color: var(--color-text-tertiary)">
						{t('payment.no_cards')}
					</p>
				{/if}

				{#if !showAddCard}
					<button
						type="button"
						class="lg-btn-secondary squish w-full mt-4 rounded-xl px-4 py-2.5 text-sm"
						onclick={() => { showAddCard = true; }}
					>
						+ {t('payment.add_card')}
					</button>
				{/if}

				{#if showAddCard}
					<div class="mt-4 pt-4" style="border-top: 1px solid var(--color-border-subtle)">
						<input
							type="text"
							class="w-full rounded-xl px-3 py-2.5 text-sm outline-none mb-2"
							style="background: var(--color-bg-sunken); color: var(--color-text-primary); border: 1px solid var(--color-border-default)"
							placeholder={t('payment.card_label')}
							bind:value={cardLabel}
						/>
						<input
							type="text"
							class="w-full rounded-xl px-3 py-2.5 text-sm outline-none mb-2 tabular-nums"
							style="background: var(--color-bg-sunken); color: var(--color-text-primary); border: 1px solid var(--color-border-default)"
							placeholder={t('payment.card_number')}
							inputmode="numeric"
							maxlength={19}
							bind:value={cardNumber}
						/>

						<div class="grid grid-cols-2 gap-2 mb-2">
							<input
								type="text"
								class="w-full rounded-xl px-3 py-2.5 text-sm outline-none tabular-nums"
								style="background: var(--color-bg-sunken); color: var(--color-text-primary); border: 1px solid var(--color-border-default)"
								placeholder={t('payment.expiry')}
								maxlength={5}
								bind:value={cardExpiry}
							/>
							<input
								type="password"
								class="w-full rounded-xl px-3 py-2.5 text-sm outline-none"
								style="background: var(--color-bg-sunken); color: var(--color-text-primary); border: 1px solid var(--color-border-default)"
								placeholder={t('payment.card_password')}
								maxlength={2}
								bind:value={cardPassword}
							/>
						</div>

						<input
							type="text"
							class="w-full rounded-xl px-3 py-2.5 text-sm outline-none mb-3 tabular-nums"
							style="background: var(--color-bg-sunken); color: var(--color-text-primary); border: 1px solid var(--color-border-default)"
							placeholder={t('payment.birthday')}
							inputmode="numeric"
							maxlength={6}
							bind:value={cardBirthday}
						/>

						<!-- Card type toggle -->
						<div class="flex rounded-xl overflow-hidden mb-3" style="background: var(--color-bg-sunken)">
							<button
								type="button"
								class="flex-1 py-2 text-sm font-medium transition-all rounded-xl squish"
								class:lg-active={cardType === 'credit'}
								style={cardType === 'credit'
									? 'color: var(--color-brand-text)'
									: 'color: var(--color-text-secondary)'}
								onclick={() => { cardType = 'credit'; }}
							>
								{t('payment.credit_card')}
							</button>
							<button
								type="button"
								class="flex-1 py-2 text-sm font-medium transition-all rounded-xl squish"
								class:lg-active={cardType === 'debit'}
								style={cardType === 'debit'
									? 'color: var(--color-brand-text)'
									: 'color: var(--color-text-secondary)'}
								onclick={() => { cardType = 'debit'; }}
							>
								{t('payment.debit_card')}
							</button>
						</div>

						<div class="flex gap-2">
							<button
								type="button"
								class="lg-btn-secondary squish flex-1 rounded-xl py-2.5 text-sm"
								onclick={() => { showAddCard = false; cardLabel = ''; cardNumber = ''; cardPassword = ''; cardBirthday = ''; cardExpiry = ''; }}
							>
								{t('common.cancel')}
							</button>
							<button
								type="button"
								class="lg-btn-primary squish flex-1 rounded-xl py-2.5 text-sm"
								disabled={addingCard || !cardNumber || !cardPassword || !cardBirthday || !cardExpiry}
								onclick={handleAddCard}
							>
								{addingCard ? t('common.loading') : t('common.save')}
							</button>
						</div>
					</div>
				{/if}
			</GlassPanel>
		{/if}
	</section>

	<!-- Appearance -->
	<section class="mb-6 page-enter stagger-3">
		<h2 class="text-sm font-semibold mb-3" style="color: var(--color-text-tertiary)">
			{t('settings.section_appearance')}
		</h2>

		<GlassPanel>
			<!-- Theme -->
			<div class="mb-4">
				<label class="text-xs font-medium mb-2 block" style="color: var(--color-text-tertiary)">
					{t('settings.theme')}
				</label>
				<div class="flex rounded-xl overflow-hidden" style="background: var(--color-bg-sunken)">
					{#each (['glass', 'clear-sky'] as const) as themeName}
						<button
							type="button"
							class="flex-1 py-2.5 text-sm font-medium transition-all rounded-xl squish"
							class:lg-active={themeStore.theme === themeName}
							style={themeStore.theme === themeName
								? 'color: var(--color-brand-text)'
								: 'color: var(--color-text-secondary)'}
							onclick={() => handleThemeChange(themeName)}
						>
							{themeName === 'glass' ? 'Glass' : 'Clear Sky'}
						</button>
					{/each}
				</div>
			</div>

			<!-- Mode -->
			<div>
				<label class="text-xs font-medium mb-2 block" style="color: var(--color-text-tertiary)">
					{t('settings.dark_mode')}
				</label>
				<div class="flex rounded-xl overflow-hidden" style="background: var(--color-bg-sunken)">
					{#each (['light', 'dark'] as const) as modeName}
						<button
							type="button"
							class="flex-1 py-2.5 text-sm font-medium transition-all rounded-xl squish"
							class:lg-active={themeStore.mode === modeName}
							style={themeStore.mode === modeName
								? 'color: var(--color-brand-text)'
								: 'color: var(--color-text-secondary)'}
							onclick={() => handleModeChange(modeName)}
						>
							{modeName === 'light' ? t('settings.light_mode') : t('settings.dark_mode')}
						</button>
					{/each}
				</div>
			</div>
		</GlassPanel>
	</section>

	<!-- Language -->
	<section class="mb-6 page-enter stagger-4">
		<h2 class="text-sm font-semibold mb-3" style="color: var(--color-text-tertiary)">
			{t('settings.language')}
		</h2>

		<GlassPanel>
			<div class="flex rounded-xl overflow-hidden" style="background: var(--color-bg-sunken)">
				{#each localeOptions as option}
					<button
						type="button"
						class="flex-1 py-2.5 text-sm font-medium transition-all rounded-xl squish"
						class:lg-active={currentLocale === option.code}
						style={currentLocale === option.code
							? 'color: var(--color-brand-text)'
							: 'color: var(--color-text-secondary)'}
						onclick={() => handleLocaleChange(option.code)}
					>
						{option.label}
					</button>
				{/each}
			</div>
		</GlassPanel>
	</section>

	<!-- Logout -->
	<section class="mb-6 page-enter stagger-5">
		<button
			type="button"
			class="lg-btn-danger squish w-full rounded-2xl px-6 py-3.5 text-base font-semibold"
			disabled={loggingOut}
			onclick={handleLogout}
		>
			{loggingOut ? t('common.loading') : t('auth.logout')}
		</button>
	</section>
</div>

<!-- Delete provider confirmation dialog -->
{#if deleteProviderTarget}
	<button
		type="button"
		class="fixed inset-0 z-40 bg-black/40 backdrop-blur-sm"
		onclick={() => { deleteProviderTarget = null; }}
		aria-label={t('common.close')}
	></button>

	<div class="fixed inset-0 z-50 flex items-center justify-center px-6">
		<div class="lg-glass-panel w-full max-w-sm p-6 modal-enter">
			<h3 class="text-lg font-bold mb-2" style="color: var(--color-text-primary)">
				{t('provider.remove')}
			</h3>
			<p class="text-sm mb-5" style="color: var(--color-text-tertiary)">
				{deleteProviderTarget === 'SRT' ? t('provider.srt') : t('provider.ktx')}
			</p>
			<div class="flex gap-3">
				<button
					type="button"
					class="lg-btn-secondary squish flex-1 rounded-xl py-2.5 text-sm"
					onclick={() => { deleteProviderTarget = null; }}
				>
					{t('common.cancel')}
				</button>
				<button
					type="button"
					class="squish flex-1 rounded-xl py-2.5 text-sm font-semibold"
					style="background: var(--color-status-error-bg); color: var(--color-status-error)"
					disabled={deletingProvider}
					onclick={handleDeleteProvider}
				>
					{deletingProvider ? t('common.loading') : t('common.delete')}
				</button>
			</div>
		</div>
	</div>
{/if}

<!-- Delete card confirmation dialog -->
{#if deleteCardTarget}
	<button
		type="button"
		class="fixed inset-0 z-40 bg-black/40 backdrop-blur-sm"
		onclick={() => { deleteCardTarget = null; }}
		aria-label={t('common.close')}
	></button>

	<div class="fixed inset-0 z-50 flex items-center justify-center px-6">
		<div class="lg-glass-panel w-full max-w-sm p-6 modal-enter">
			<h3 class="text-lg font-bold mb-2" style="color: var(--color-text-primary)">
				{t('common.delete')}
			</h3>
			<p class="text-sm mb-5" style="color: var(--color-text-tertiary)">
				{cards.find((c) => c.id === deleteCardTarget)?.label ?? ''}
			</p>
			<div class="flex gap-3">
				<button
					type="button"
					class="lg-btn-secondary squish flex-1 rounded-xl py-2.5 text-sm"
					onclick={() => { deleteCardTarget = null; }}
				>
					{t('common.cancel')}
				</button>
				<button
					type="button"
					class="squish flex-1 rounded-xl py-2.5 text-sm font-semibold"
					style="background: var(--color-status-error-bg); color: var(--color-status-error)"
					disabled={deletingCard}
					onclick={handleDeleteCard}
				>
					{deletingCard ? t('common.loading') : t('common.delete')}
				</button>
			</div>
		</div>
	</div>
{/if}
