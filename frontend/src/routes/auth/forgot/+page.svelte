<script lang="ts">
	import { goto } from '$app/navigation';
	import { forgotPassword } from '$lib/api/auth';
	import { t } from '$lib/i18n';
	import GlassPanel from '$lib/components/GlassPanel.svelte';

	let email = $state('');
	let loading = $state(false);
	let error = $state('');
	let sent = $state(false);

	async function handleSendReset(): Promise<void> {
		loading = true;
		error = '';
		try {
			await forgotPassword(email);
			sent = true;
		} catch (err) {
			error = err instanceof Error ? err.message : t('error.unexpected');
		} finally {
			loading = false;
		}
	}

	function handleKeydown(event: KeyboardEvent): void {
		if (event.key === 'Enter' && email && !loading && !sent) {
			handleSendReset();
		}
	}
</script>

<svelte:head><title>{t('auth.reset_password')} | Bominal</title></svelte:head>

<div class="flex min-h-screen items-center justify-center px-4">
	<div class="page-enter w-full max-w-sm">
		<GlassPanel class="flex flex-col gap-5">
			<div class="text-center">
				<h1 class="text-2xl font-bold tracking-tight" style="color: var(--color-text-primary);">
					{t('auth.reset_password')}
				</h1>
				<p class="mt-1 text-sm" style="color: var(--color-text-tertiary);">
					{t('auth.reset_subtitle')}
				</p>
			</div>

			{#if error}
				<p class="text-center text-sm" style="color: var(--color-status-error);">{error}</p>
			{/if}

			{#if sent}
				<div class="flex flex-col items-center gap-3 py-4 text-center">
					<div
						class="flex h-12 w-12 items-center justify-center rounded-full"
						style="background: var(--color-status-success-bg);"
					>
						<svg
							xmlns="http://www.w3.org/2000/svg"
							class="h-6 w-6"
							fill="none"
							viewBox="0 0 24 24"
							stroke="currentColor"
							style="color: var(--color-status-success);"
						>
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
						</svg>
					</div>
					<p class="text-sm font-medium" style="color: var(--color-text-primary);">
						{t('auth.reset_link_sent')}
					</p>
				</div>
			{:else}
				<input
					type="email"
					bind:value={email}
					placeholder={t('auth.email_placeholder')}
					autocomplete="email"
					class="lg-glass-card w-full rounded-xl px-4 py-3 text-sm outline-none"
					style="color: var(--color-text-primary); border-color: var(--color-border-default);"
					onkeydown={handleKeydown}
				/>

				<button
					class="lg-btn-primary squish w-full rounded-2xl px-6 py-3.5 text-base"
					disabled={loading || !email}
					onclick={handleSendReset}
				>
					{#if loading}
						{t('common.loading')}
					{:else}
						{t('auth.send_reset_link')}
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
		</GlassPanel>
	</div>
</div>
