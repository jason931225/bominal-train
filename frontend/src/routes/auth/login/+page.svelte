<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { login } from '$lib/api/auth';
	import { post } from '$lib/api/client';
	import { startConditionalPasskeyLogin } from '$lib/interop/passkey';
	import { t } from '$lib/i18n';
	import { auth } from '$lib/stores/auth.svelte';
	import type { AuthResponse } from '$lib/types';
	import GlassPanel from '$lib/components/GlassPanel.svelte';

	let email = $state('');
	let password = $state('');
	let showPassword = $state(false);
	let loading = $state(false);
	let error = $state('');
	let conditionalAvailable = $state(false);

	async function handleLogin(): Promise<void> {
		loading = true;
		error = '';
		try {
			const result = await login(email, password);
			auth.setUser(result);
			goto('/home');
		} catch (err) {
			error = err instanceof Error ? err.message : t('error.login_failed');
		} finally {
			loading = false;
		}
	}

	async function attemptConditionalLogin(): Promise<void> {
		try {
			const available = await PublicKeyCredential.isConditionalMediationAvailable?.();
			if (!available) return;
			conditionalAvailable = true;

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

	function handleKeydown(event: KeyboardEvent): void {
		if (event.key === 'Enter' && !loading) {
			handleLogin();
		}
	}

	onMount(() => {
		attemptConditionalLogin();
	});
</script>

<svelte:head><title>{t('auth.sign_in')} | Bominal</title></svelte:head>

<div class="flex min-h-screen items-center justify-center px-4 pt-12">
	<div class="page-enter w-full max-w-sm">
		<GlassPanel class="flex flex-col gap-5">
			<div class="text-center">
				<h1 class="text-2xl font-bold tracking-tight" style="color: var(--color-text-primary);">
					{t('auth.welcome_back')}
				</h1>
				<p class="mt-1 text-sm" style="color: var(--color-text-tertiary);">
					{t('auth.enter_email_password')}
				</p>
			</div>

			{#if error}
				<p class="text-center text-sm" style="color: var(--color-status-error);">{error}</p>
			{/if}

			<div class="flex flex-col gap-3">
				<input
					type="email"
					bind:value={email}
					placeholder={t('auth.email_placeholder')}
					autocomplete={conditionalAvailable ? 'username webauthn' : 'username'}
					class="lg-glass-card w-full rounded-xl px-4 py-3 text-sm outline-none"
					style="color: var(--color-text-primary); border-color: var(--color-border-default);"
					onkeydown={handleKeydown}
				/>

				<div class="relative">
					<input
						type={showPassword ? 'text' : 'password'}
						bind:value={password}
						placeholder={t('auth.password')}
						autocomplete={conditionalAvailable ? 'current-password webauthn' : 'current-password'}
						class="lg-glass-card w-full rounded-xl px-4 py-3 pr-12 text-sm outline-none"
						style="color: var(--color-text-primary); border-color: var(--color-border-default);"
						onkeydown={handleKeydown}
					/>
					<button
						type="button"
						class="absolute right-3 top-1/2 -translate-y-1/2 p-1 rounded"
						style="color: var(--color-text-tertiary);"
						onclick={() => (showPassword = !showPassword)}
						aria-label={showPassword ? t('auth.hide_password') : t('auth.show_password')}
					>
						{#if showPassword}
							<!-- Phosphor EyeSlash -->
							<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 256 256" fill="currentColor"><path d="M53.92,34.62A8,8,0,1,0,42.08,45.38L61.32,66.55C25,88.84,9.38,123.2,8.69,124.76a8,8,0,0,0,0,6.5c.35.79,8.82,19.57,27.65,38.4C61.43,194.74,93.12,208,128,208a127.11,127.11,0,0,0,52.07-10.83l22,24.21a8,8,0,1,0,11.84-10.76Zm47.33,75.8,41.67,45.85a32,32,0,0,1-41.67-45.85ZM128,192c-30.78,0-57.67-11.19-79.93-33.29A133.47,133.47,0,0,1,25,128c4.69-8.79,19.66-33.39,47.35-49.38l18,19.75a48,48,0,0,0,63.66,70l14.73,16.2A112,112,0,0,1,128,192Zm6-95.43a8,8,0,0,1,3-15.72,48.16,48.16,0,0,1,38.77,42.64,8,8,0,0,1-7.22,8.71,6.39,6.39,0,0,1-.75,0,8,8,0,0,1-8-7.26A32.09,32.09,0,0,0,134,96.57Zm113.28,34.69c-.42.94-10.55,23.37-33.36,43.8a8,8,0,1,1-10.67-11.92A132.77,132.77,0,0,0,231,128a133.15,133.15,0,0,0-23.07-30.71C185.67,75.19,158.78,64,128,64a118.37,118.37,0,0,0-19.36,1.57A8,8,0,1,1,106,49.79,134,134,0,0,1,128,48c34.88,0,66.57,13.26,91.66,38.35,18.83,18.83,27.3,37.62,27.65,38.41A8,8,0,0,1,247.31,131.26Z"/></svg>
						{:else}
							<!-- Phosphor Eye -->
							<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 256 256" fill="currentColor"><path d="M247.31,124.76c-.35-.79-8.82-19.58-27.65-38.41C194.57,61.26,162.88,48,128,48S61.43,61.26,36.34,86.35C17.51,105.18,9,124,8.69,124.76a8,8,0,0,0,0,6.5c.35.79,8.82,19.57,27.65,38.4C61.43,194.74,93.12,208,128,208s66.57-13.26,91.66-38.34c18.83-18.83,27.3-37.61,27.65-38.4A8,8,0,0,0,247.31,124.76ZM128,192c-30.78,0-57.67-11.19-79.93-33.29A133.47,133.47,0,0,1,25,128a133.33,133.33,0,0,1,23.07-30.71C70.33,75.19,97.22,64,128,64s57.67,11.19,79.93,33.29A133.46,133.46,0,0,1,231,128C226.94,135.84,195.17,192,128,192Zm0-112a48,48,0,1,0,48,48A48.05,48.05,0,0,0,128,80Zm0,80a32,32,0,1,1,32-32A32,32,0,0,1,128,160Z"/></svg>
						{/if}
					</button>
				</div>
			</div>

			<button
				class="lg-btn-primary squish w-full rounded-2xl px-6 py-3.5 text-base"
				disabled={loading || !email || !password}
				onclick={handleLogin}
			>
				{#if loading}
					{t('common.loading')}
				{:else}
					{t('auth.sign_in')}
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
				onclick={() => goto('/auth/forgot')}
			>
				{t('auth.forgot_password')}
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
