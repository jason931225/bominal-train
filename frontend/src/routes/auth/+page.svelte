<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { post } from '$lib/api/client';
	import { startPasskeyLogin, startConditionalPasskeyLogin } from '$lib/interop/passkey';
	import { t } from '$lib/i18n';
	import { auth } from '$lib/stores/auth.svelte';
	import type { AuthResponse } from '$lib/types';
	import GlassPanel from '$lib/components/GlassPanel.svelte';
	import Icon from '$lib/components/Icon.svelte';

	let loading = $state(false);
	let error = $state('');

	async function handlePasskeyLogin(): Promise<void> {
		loading = true;
		error = '';
		try {
			const startResult = await post<{ options: string }>('/api/auth/passkey/login/start');
			const credentialJson = await startPasskeyLogin(startResult.options);
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
			const credentialJson = await startConditionalPasskeyLogin(
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

<div class="flex min-h-screen items-center justify-center px-4 pt-12">
	<div class="page-enter w-full max-w-sm">
		<GlassPanel class="flex flex-col items-center gap-6 text-center">
			<div
				class="flex h-16 w-16 items-center justify-center rounded-2xl"
				style="background-color: var(--color-brand-primary); color: white;"
			>
				<Icon name="train" size={36} />
			</div>
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
				class="lg-btn-primary squish flex w-full items-center justify-center gap-3 rounded-2xl px-6 py-3.5 text-base"
				disabled={loading}
				onclick={handlePasskeyLogin}
			>
				<Icon name="fingerprint" size={24} />
				{#if loading}
					{t('common.loading')}
				{:else}
					{t('auth.passkey_signin')}
				{/if}
			</button>

			<!-- Separator -->
			<div class="relative w-full">
				<div class="absolute inset-0 flex items-center">
					<div class="w-full" style="border-top: 1px solid var(--color-border-default);"></div>
				</div>
			</div>

			<button
				class="lg-btn-secondary squish flex w-full items-center justify-center gap-3 rounded-2xl px-6 py-3 text-sm"
				onclick={() => goto('/auth/login')}
			>
				<Icon name="envelope" size={20} />
				{t('auth.continue_email')}
			</button>

			<button
				class="lg-btn-secondary squish w-full rounded-2xl px-6 py-3 text-sm"
				onclick={() => goto('/auth/signup')}
			>
				{t('auth.signup_link')}
			</button>
		</GlassPanel>
	</div>
</div>
