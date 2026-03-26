<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { post } from '$lib/api/client';
	import { startPasskeyLogin, startConditionalPasskeyLogin } from '$lib/interop/passkey';
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

<div class="flex min-h-screen items-center justify-center px-4">
	<div class="page-enter w-full max-w-sm">
		<GlassPanel class="flex flex-col items-center gap-6 text-center">
			<div
				class="flex h-16 w-16 items-center justify-center rounded-2xl"
				style="background-color: var(--color-brand-primary); color: white;"
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					width="36"
					height="36"
					viewBox="0 0 256 256"
					fill="currentColor"
					aria-hidden="true"
				>
					<path d="M184,24H72A32,32,0,0,0,40,56V184a32,32,0,0,0,32,32h8L64,232a8,8,0,0,0,16,0l16-16h64l16,16a8,8,0,0,0,16,0l-16-16h8a32,32,0,0,0,32-32V56A32,32,0,0,0,184,24ZM72,40H184a16,16,0,0,1,16,16v64H56V56A16,16,0,0,1,72,40ZM184,200H72a16,16,0,0,1-16-16V136H200v48A16,16,0,0,1,184,200ZM96,172a12,12,0,1,1-12-12A12,12,0,0,1,96,172Zm88,0a12,12,0,1,1-12-12A12,12,0,0,1,184,172Z" />
				</svg>
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
				<!-- Phosphor Fingerprint -->
				<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 256 256" fill="currentColor"><path d="M72,128a134.63,134.63,0,0,1-14.16,60.47,8,8,0,1,1-14.32-7.12A118.8,118.8,0,0,0,56,128,72.08,72.08,0,0,1,128,56a71.56,71.56,0,0,1,44.6,15.49,8,8,0,0,1-10,12.49A55.56,55.56,0,0,0,128,72,56.06,56.06,0,0,0,72,128Zm56-104A104.11,104.11,0,0,0,24,128a87.53,87.53,0,0,1-5.33,30.15,8,8,0,1,0,14.86,5.94A103.62,103.62,0,0,0,40,128a88,88,0,0,1,176,0,281.6,281.6,0,0,1-7.11,56.37,8,8,0,0,0,5.67,9.79,8.11,8.11,0,0,0,2.06.27,8,8,0,0,0,7.73-5.93A297.88,297.88,0,0,0,232,128,104.12,104.12,0,0,0,128,24Zm0,32a72.08,72.08,0,0,0-72,72,8,8,0,0,0,16,0,56.06,56.06,0,0,1,112,0,245.22,245.22,0,0,1-6.21,49.19,8,8,0,0,0,5.65,9.81,8.13,8.13,0,0,0,2.08.27,8,8,0,0,0,7.73-5.93A261.42,261.42,0,0,0,200,128,72.08,72.08,0,0,0,128,56Zm0,40a32,32,0,0,0-32,32,167.43,167.43,0,0,1-8.51,53.06,8,8,0,0,0,14.86,5.94A183.33,183.33,0,0,0,112,128a16,16,0,0,1,32,0,214.67,214.67,0,0,1-20.51,92.34,8,8,0,1,0,14.28,7.22A230.69,230.69,0,0,0,160,128,32,32,0,0,0,128,96Zm0-64A96.11,96.11,0,0,0,32,128a55.8,55.8,0,0,1-4.28,21.57,8,8,0,0,0,14.86,5.94A71.87,71.87,0,0,0,48,128a80,80,0,0,1,160,0,317.35,317.35,0,0,1-7.78,62.57,8,8,0,1,0,15.56,3.88A332.91,332.91,0,0,0,224,128,96.11,96.11,0,0,0,128,32Z"/></svg>
				{#if loading}
					{t('common.loading')}
				{:else}
					{t('auth.passkey_signin')}
				{/if}
			</button>

			<!-- Separator -->
			<div class="relative">
				<div class="absolute inset-0 flex items-center">
					<div class="w-full" style="border-top: 1px solid var(--color-border-default);"></div>
				</div>
			</div>

			<button
				class="lg-btn-secondary squish w-full rounded-2xl px-6 py-3 text-sm"
				onclick={() => goto('/auth/login')}
			>
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
