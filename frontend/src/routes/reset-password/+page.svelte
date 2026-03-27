<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { resetPassword } from '$lib/api/auth';
	import { t } from '$lib/i18n';
	import GlassPanel from '$lib/components/GlassPanel.svelte';

	let token = $state('');
	let password = $state('');
	let confirmPassword = $state('');
	let showPassword = $state(false);
	let loading = $state(false);
	let error = $state('');
	let success = $state(false);
	let missingToken = $state(false);

	const passwordStrength = $derived(computePasswordStrength(password));
	const passwordsMatch = $derived(
		confirmPassword.length > 0 && password === confirmPassword
	);
	const passwordsMismatch = $derived(
		confirmPassword.length > 0 && password !== confirmPassword
	);
	const isFormValid = $derived(
		password.length >= 8 && passwordsMatch && !missingToken
	);

	function computePasswordStrength(pw: string): { score: number; label: string; color: string } {
		if (pw.length === 0) return { score: 0, label: '', color: '' };

		let score = 0;
		if (pw.length >= 8) score += 1;
		if (pw.length >= 12) score += 1;
		if (/[a-z]/.test(pw) && /[A-Z]/.test(pw)) score += 1;
		if (/\d/.test(pw)) score += 1;
		if (/[^a-zA-Z0-9]/.test(pw)) score += 1;

		if (score <= 1) return { score: 20, label: t('auth.pw_weak'), color: 'var(--color-status-error)' };
		if (score === 2) return { score: 40, label: t('auth.pw_fair'), color: 'var(--color-status-warning)' };
		if (score === 3) return { score: 65, label: t('auth.pw_good'), color: 'var(--color-status-warning)' };
		return { score: 100, label: t('auth.pw_strong'), color: 'var(--color-status-success)' };
	}

	async function handleReset(): Promise<void> {
		loading = true;
		error = '';
		try {
			await resetPassword(token, password);
			success = true;
		} catch (err) {
			error = err instanceof Error ? err.message : t('error.unexpected');
		} finally {
			loading = false;
		}
	}

	function handleKeydown(event: KeyboardEvent): void {
		if (event.key === 'Enter' && isFormValid && !loading) {
			handleReset();
		}
	}

	onMount(() => {
		const urlToken = page.url.searchParams.get('token');
		if (!urlToken) {
			missingToken = true;
			error = t('auth.missing_token');
			return;
		}
		token = urlToken;
	});
</script>

<svelte:head><title>{t('auth.reset_password')} | Bominal</title></svelte:head>

<div class="flex min-h-screen items-center justify-center px-4">
	<div class="page-enter w-full max-w-sm">
		<GlassPanel class="flex flex-col gap-5">
			{#if success}
				<div class="flex flex-col items-center gap-4 py-4 text-center">
					<div
						class="flex h-14 w-14 items-center justify-center rounded-full"
						style="background: var(--color-status-success-bg);"
					>
						<svg
							xmlns="http://www.w3.org/2000/svg"
							class="h-7 w-7"
							fill="none"
							viewBox="0 0 24 24"
							stroke="currentColor"
							style="color: var(--color-status-success);"
						>
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
						</svg>
					</div>

					<div>
						<h1 class="text-2xl font-bold tracking-tight" style="color: var(--color-text-primary);">
							{t('auth.reset_password')}
						</h1>
						<p class="mt-2 text-sm" style="color: var(--color-text-tertiary);">
							{t('auth.password_reset_success')}
						</p>
					</div>

					<button
						class="lg-btn-primary squish w-full rounded-2xl px-6 py-3.5 text-base"
						onclick={() => goto('/auth/login')}
					>
						{t('auth.sign_in')}
					</button>
				</div>
			{:else}
				<div class="text-center">
					<h1 class="text-2xl font-bold tracking-tight" style="color: var(--color-text-primary);">
						{t('auth.reset_password')}
					</h1>
				</div>

				{#if error}
					<p class="text-center text-sm" style="color: var(--color-status-error);">{error}</p>
				{/if}

				{#if !missingToken}
					<div class="flex flex-col gap-3">
						<div class="relative">
							<input
								type={showPassword ? 'text' : 'password'}
								bind:value={password}
								placeholder={t('auth.new_password')}
								autocomplete="new-password"
								class="lg-glass-card w-full rounded-xl px-4 py-3 pr-12 text-sm outline-none"
								style="color: var(--color-text-primary); border-color: var(--color-border-default);"
								onkeydown={handleKeydown}
							/>
							<button
								type="button"
								class="absolute right-3 top-1/2 -translate-y-1/2 text-xs"
								style="color: var(--color-text-tertiary);"
								onclick={() => (showPassword = !showPassword)}
								aria-label={showPassword ? t('auth.hide_password') : t('auth.show_password')}
							>
								{showPassword ? t('auth.hide_password') : t('auth.show_password')}
							</button>
						</div>

						{#if password.length > 0}
							<div class="flex flex-col gap-1.5">
								<div class="h-1.5 w-full overflow-hidden rounded-full" style="background: var(--color-bg-sunken);">
									<div
										class="h-full rounded-full transition-all duration-300"
										style="width: {passwordStrength.score}%; background: {passwordStrength.color};"
									></div>
								</div>
								<span class="text-xs font-medium" style="color: {passwordStrength.color};">
									{passwordStrength.label}
								</span>
							</div>
						{/if}

						<input
							type={showPassword ? 'text' : 'password'}
							bind:value={confirmPassword}
							placeholder={t('auth.confirm_password')}
							autocomplete="new-password"
							class="lg-glass-card w-full rounded-xl px-4 py-3 text-sm outline-none"
							style="color: var(--color-text-primary); border-color: var(--color-border-default);"
							onkeydown={handleKeydown}
						/>

						{#if passwordsMatch}
							<p class="text-xs" style="color: var(--color-status-success);">
								{t('auth.passwords_match')}
							</p>
						{:else if passwordsMismatch}
							<p class="text-xs" style="color: var(--color-status-error);">
								{t('auth.passwords_mismatch')}
							</p>
						{/if}
					</div>

					<button
						class="lg-btn-primary squish w-full rounded-2xl px-6 py-3.5 text-base"
						disabled={loading || !isFormValid}
						onclick={handleReset}
					>
						{#if loading}
							{t('common.loading')}
						{:else}
							{t('auth.reset_password')}
						{/if}
					</button>
				{/if}

				<div class="text-center">
					<button
						class="text-sm font-medium"
						style="color: var(--color-brand-text);"
						onclick={() => goto('/auth')}
					>
						{t('auth.back_to_signin')}
					</button>
				</div>
			{/if}
		</GlassPanel>
	</div>
</div>
