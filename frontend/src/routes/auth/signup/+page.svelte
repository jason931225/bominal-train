<script lang="ts">
	import { goto } from '$app/navigation';
	import { register } from '$lib/api/auth';
	import { t } from '$lib/i18n';
	import { auth } from '$lib/stores/auth.svelte';
	import GlassPanel from '$lib/components/GlassPanel.svelte';

	let email = $state('');
	let password = $state('');
	let displayName = $state('');
	let showPassword = $state(false);
	let loading = $state(false);
	let error = $state('');

	const passwordStrength = $derived(computePasswordStrength(password));

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

	const isEmailValid = $derived(/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email));
	const isFormValid = $derived(isEmailValid && password.length >= 8 && displayName.trim().length > 0);

	async function handleSignup(): Promise<void> {
		loading = true;
		error = '';
		try {
			const result = await register(email, password, displayName.trim());
			auth.setUser(result);
			goto('/auth/verify');
		} catch (err) {
			error = err instanceof Error ? err.message : t('error.unexpected');
		} finally {
			loading = false;
		}
	}

	function handleKeydown(event: KeyboardEvent): void {
		if (event.key === 'Enter' && isFormValid && !loading) {
			handleSignup();
		}
	}
</script>

<svelte:head><title>{t('auth.create_account')} | Bominal</title></svelte:head>

<div class="flex min-h-screen items-center justify-center px-4">
	<div class="page-enter w-full max-w-sm">
		<GlassPanel class="flex flex-col gap-5">
			<div class="text-center">
				<h1 class="text-2xl font-bold tracking-tight" style="color: var(--color-text-primary);">
					{t('auth.create_account')}
				</h1>
				<p class="mt-1 text-sm" style="color: var(--color-text-tertiary);">
					{t('auth.get_started')}
				</p>
			</div>

			{#if error}
				<p class="text-center text-sm" style="color: var(--color-status-error);">{error}</p>
			{/if}

			<div class="flex flex-col gap-3">
				<input
					type="text"
					bind:value={displayName}
					placeholder={t('auth.display_name')}
					autocomplete="name"
					class="lg-glass-card w-full rounded-xl px-4 py-3 text-sm outline-none"
					style="color: var(--color-text-primary); border-color: var(--color-border-default);"
					onkeydown={handleKeydown}
				/>

				<input
					type="email"
					bind:value={email}
					placeholder={t('auth.email_placeholder')}
					autocomplete="email"
					class="lg-glass-card w-full rounded-xl px-4 py-3 text-sm outline-none"
					style="color: var(--color-text-primary); border-color: var(--color-border-default);"
					onkeydown={handleKeydown}
				/>

				<div class="relative">
					<input
						type={showPassword ? 'text' : 'password'}
						bind:value={password}
						placeholder={t('auth.password')}
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
			</div>

			<button
				class="lg-btn-primary squish w-full rounded-2xl px-6 py-3.5 text-base"
				disabled={loading || !isFormValid}
				onclick={handleSignup}
			>
				{#if loading}
					{t('common.loading')}
				{:else}
					{t('auth.create_account')}
				{/if}
			</button>

			<div class="text-center text-sm" style="color: var(--color-text-tertiary);">
				{t('auth.has_account')}
				<button
					class="font-medium underline"
					style="color: var(--color-brand-text);"
					onclick={() => goto('/auth/login')}
				>
					{t('auth.signin_link')}
				</button>
			</div>
		</GlassPanel>
	</div>
</div>
