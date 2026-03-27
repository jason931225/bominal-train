<script lang="ts">
	import { goto } from '$app/navigation';
	import { resendVerification } from '$lib/api/auth';
	import { t } from '$lib/i18n';
	import GlassPanel from '$lib/components/GlassPanel.svelte';

	let resending = $state(false);
	let resent = $state(false);
	let error = $state('');

	async function handleResend(): Promise<void> {
		resending = true;
		error = '';
		try {
			await resendVerification();
			resent = true;
		} catch (err) {
			error = err instanceof Error ? err.message : t('error.unexpected');
		} finally {
			resending = false;
		}
	}
</script>

<svelte:head><title>{t('auth.check_email')} | Bominal</title></svelte:head>

<div class="flex min-h-screen items-center justify-center px-4">
	<div class="page-enter w-full max-w-sm">
		<GlassPanel class="flex flex-col items-center gap-5 text-center">
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
						d="M3 8l7.89 5.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"
					/>
				</svg>
			</div>

			<div>
				<h1 class="text-2xl font-bold tracking-tight" style="color: var(--color-text-primary);">
					{t('auth.check_email')}
				</h1>
				<p class="mt-2 text-sm" style="color: var(--color-text-tertiary);">
					{t('auth.verify_click_link')}
				</p>
			</div>

			{#if error}
				<p class="text-sm" style="color: var(--color-status-error);">{error}</p>
			{/if}

			<p class="text-sm" style="color: var(--color-text-tertiary);">
				{t('auth.resend_prompt')}
				<button
					class="font-medium underline"
					style="color: var(--color-brand-text);"
					disabled={resending}
					onclick={handleResend}
				>
					{#if resending}
						{t('common.loading')}
					{:else if resent}
						{t('auth.resend_link')} ✓
					{:else}
						{t('auth.resend_link')}
					{/if}
				</button>
			</p>

			<div class="flex w-full flex-col gap-3">
				<button
					class="lg-btn-primary squish w-full rounded-2xl px-6 py-3.5 text-base"
					onclick={() => goto('/auth/add-passkey')}
				>
					{t('auth.add_passkey_now')}
				</button>

				<button
					class="lg-btn-secondary squish w-full rounded-2xl px-6 py-3 text-sm"
					onclick={() => goto('/home')}
				>
					{t('auth.skip_for_now')}
				</button>
			</div>
		</GlassPanel>
	</div>
</div>
