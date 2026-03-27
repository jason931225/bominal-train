<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { verifyEmail } from '$lib/api/auth';
	import { t } from '$lib/i18n';
	import GlassPanel from '$lib/components/GlassPanel.svelte';

	let loading = $state(true);
	let error = $state('');
	let verified = $state(false);

	onMount(async () => {
		const token = page.url.searchParams.get('token');
		if (!token) {
			error = t('auth.missing_token');
			loading = false;
			return;
		}

		try {
			await verifyEmail(token);
			verified = true;
		} catch (err) {
			error = err instanceof Error ? err.message : t('auth.verify_failed');
		} finally {
			loading = false;
		}
	});
</script>

<svelte:head><title>{t('auth.email_verified')} | Bominal</title></svelte:head>

<div class="flex min-h-screen items-center justify-center px-4">
	<div class="page-enter w-full max-w-sm">
		<GlassPanel class="flex flex-col items-center gap-5 text-center">
			{#if loading}
				<div
					class="flex h-14 w-14 items-center justify-center rounded-full"
					style="background: var(--color-brand-primary);"
				>
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="h-7 w-7 animate-spin"
						fill="none"
						viewBox="0 0 24 24"
						stroke="currentColor"
						style="color: var(--color-brand-text);"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
						/>
					</svg>
				</div>
				<p class="text-sm font-medium" style="color: var(--color-text-secondary);">
					{t('auth.verifying')}
				</p>
			{:else if verified}
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
						{t('auth.email_verified')}
					</h1>
					<p class="mt-2 text-sm" style="color: var(--color-text-tertiary);">
						{t('auth.email_verified_desc')}
					</p>
				</div>

				<button
					class="lg-btn-primary squish w-full rounded-2xl px-6 py-3.5 text-base"
					onclick={() => goto('/home')}
				>
					{t('common.next')}
				</button>
			{:else}
				<div
					class="flex h-14 w-14 items-center justify-center rounded-full"
					style="background: var(--color-status-error-bg);"
				>
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="h-7 w-7"
						fill="none"
						viewBox="0 0 24 24"
						stroke="currentColor"
						style="color: var(--color-status-error);"
					>
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
					</svg>
				</div>

				<div>
					<h1 class="text-2xl font-bold tracking-tight" style="color: var(--color-text-primary);">
						{t('auth.verify_failed')}
					</h1>
					<p class="mt-2 text-sm" style="color: var(--color-status-error);">
						{error}
					</p>
				</div>

				<button
					class="lg-btn-secondary squish w-full rounded-2xl px-6 py-3 text-sm"
					onclick={() => goto('/auth')}
				>
					{t('auth.go_to_login')}
				</button>
			{/if}
		</GlassPanel>
	</div>
</div>
