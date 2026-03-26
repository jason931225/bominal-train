<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { post } from '$lib/api/client';
	import { t } from '$lib/i18n';
	import { auth } from '$lib/stores/auth.svelte';
	import type { AuthResponse } from '$lib/types';
	import GlassPanel from '$lib/components/GlassPanel.svelte';

	let loading = $state(false);
	let error = $state('');

	async function handlePasskeyLogin(): Promise<void> {
		loading = true;
		error = '';
		try {
			const startResult = await post<{ options: string }>('/api/auth/passkey/login/start');
			const credentialJson = await (window as any).__startPasskeyLogin(startResult.options);
			const authResponse = await post<AuthResponse>('/api/auth/passkey/login/finish', {
				credential: credentialJson
			});
			auth.setUser(authResponse);
			goto('/home');
		} catch (err) {
			error = err instanceof Error ? err.message : t('error.passkey_failed');
		} finally {
			loading = false;
		}
	}

	async function attemptConditionalLogin(): Promise<void> {
		try {
			const available = await PublicKeyCredential.isConditionalMediationAvailable?.();
			if (!available) return;

			const startResult = await post<{ options: string }>('/api/auth/passkey/login/start');
			const credentialJson = await (window as any).__startConditionalPasskeyLogin(
				startResult.options
			);
			const authResponse = await post<AuthResponse>('/api/auth/passkey/login/finish', {
				credential: credentialJson
			});
			auth.setUser(authResponse);
			goto('/home');
		} catch {
			// Conditional login is best-effort; ignore failures silently
		}
	}

	onMount(() => {
		attemptConditionalLogin();
	});
</script>

<svelte:head><title>{t('auth.get_started')} | Bominal</title></svelte:head>

<div class="flex min-h-screen items-center justify-center px-4">
	<div class="page-enter w-full max-w-sm">
		<GlassPanel class="flex flex-col items-center gap-6 text-center">
			<h1
				class="app-brand-wordmark bg-clip-text text-4xl font-bold tracking-tight text-transparent"
			>
				Bominal
			</h1>
			<p class="text-sm" style="color: var(--color-text-tertiary);">
				{t('auth.get_started')}
			</p>

			{#if error}
				<p class="text-sm" style="color: var(--color-status-error);">{error}</p>
			{/if}

			<button
				class="lg-btn-primary squish w-full rounded-2xl px-6 py-3.5 text-base"
				disabled={loading}
				onclick={handlePasskeyLogin}
			>
				{#if loading}
					{t('common.loading')}
				{:else}
					{t('auth.passkey_signin')}
				{/if}
			</button>

			<button
				class="lg-btn-secondary squish w-full rounded-2xl px-6 py-3 text-sm"
				onclick={() => goto('/auth/login')}
			>
				{t('auth.continue_email')}
			</button>

			<p class="text-sm" style="color: var(--color-text-tertiary);">
				{t('auth.no_account')}
				<button
					class="font-medium underline"
					style="color: var(--color-brand-text);"
					onclick={() => goto('/auth/signup')}
				>
					{t('auth.signup_link')}
				</button>
			</p>
		</GlassPanel>
	</div>
</div>
