<script lang="ts">
	import { goto } from '$app/navigation';
	import { post } from '$lib/api/client';
	import { t } from '$lib/i18n';
	import GlassPanel from '$lib/components/GlassPanel.svelte';

	let loading = $state(false);
	let error = $state('');
	let success = $state(false);

	async function handleAddPasskey(): Promise<void> {
		loading = true;
		error = '';
		try {
			const startResult = await post<{ options: string }>('/api/auth/passkey/register/start');
			const credentialJson = await (window as any).__startPasskeyRegistration(
				startResult.options
			);
			await post('/api/auth/passkey/register/finish', { credential: credentialJson });
			success = true;
			setTimeout(() => goto('/home'), 2000);
		} catch (err) {
			error = err instanceof Error ? err.message : t('error.passkey_register_failed');
		} finally {
			loading = false;
		}
	}
</script>

<svelte:head><title>{t('auth.add_passkey')} | Bominal</title></svelte:head>

<div class="flex min-h-screen items-center justify-center px-4">
	<div class="page-enter w-full max-w-sm">
		<GlassPanel class="flex flex-col items-center gap-5 text-center">
			{#if success}
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

				<h1 class="text-2xl font-bold tracking-tight" style="color: var(--color-text-primary);">
					{t('auth.passkey_signin')}
				</h1>
				<p class="text-sm" style="color: var(--color-text-tertiary);">
					{t('auth.passkey_benefit_1')}
				</p>
			{:else}
				<div
					class="flex h-14 w-14 items-center justify-center rounded-full"
					style="background: var(--color-brand-primary);"
				>
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="h-7 w-7"
						fill="none"
						viewBox="0 0 24 24"
						stroke="currentColor"
						style="color: var(--color-brand-text);"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z"
						/>
					</svg>
				</div>

				<div>
					<h1 class="text-2xl font-bold tracking-tight" style="color: var(--color-text-primary);">
						{t('auth.add_passkey')}
					</h1>
				</div>

				<ul class="flex flex-col gap-2 text-left text-sm" style="color: var(--color-text-secondary);">
					<li class="flex items-start gap-2">
						<span style="color: var(--color-status-success);">&#10003;</span>
						{t('auth.passkey_benefit_1')}
					</li>
					<li class="flex items-start gap-2">
						<span style="color: var(--color-status-success);">&#10003;</span>
						{t('auth.passkey_benefit_2')}
					</li>
					<li class="flex items-start gap-2">
						<span style="color: var(--color-status-success);">&#10003;</span>
						{t('auth.passkey_benefit_3')}
					</li>
				</ul>

				{#if error}
					<p class="text-sm" style="color: var(--color-status-error);">{error}</p>
				{/if}

				<div class="flex w-full flex-col gap-3">
					<button
						class="lg-btn-primary squish w-full rounded-2xl px-6 py-3.5 text-base"
						disabled={loading}
						onclick={handleAddPasskey}
					>
						{#if loading}
							{t('common.loading')}
						{:else}
							{t('auth.add_passkey_now')}
						{/if}
					</button>

					<button
						class="lg-btn-secondary squish w-full rounded-2xl px-6 py-3 text-sm"
						onclick={() => goto('/home')}
					>
						{t('auth.skip_for_now')}
					</button>
				</div>
			{/if}
		</GlassPanel>
	</div>
</div>
