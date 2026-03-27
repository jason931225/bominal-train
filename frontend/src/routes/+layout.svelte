<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { auth } from '$lib/stores/auth.svelte';
	import { themeStore } from '$lib/stores/theme.svelte';
	import { initLocale } from '$lib/i18n';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import BottomNav from '$lib/components/BottomNav.svelte';
	import '../app.css';

	let { children } = $props();

	const authPaths = ['/auth', '/verify-email', '/reset-password'];
	let isAuthPage = $derived(
		authPaths.some(
			(p) => $page.url.pathname === p || $page.url.pathname.startsWith(p + '/')
		)
	);
	let isRootPage = $derived($page.url.pathname === '/');

	let authReady = $state(false);

	onMount(async () => {
		themeStore.init();
		initLocale();
		await auth.check();
		authReady = true;

		if (
			!auth.isAuthenticated &&
			!isAuthPage &&
			!isRootPage &&
			$page.url.pathname !== '/verify-email' &&
			$page.url.pathname !== '/reset-password'
		) {
			goto('/auth');
		}
	});
</script>

{#if !authReady}
	<!-- Loading spinner while auth check is in-flight — prevents dashboard flash -->
	<div class="flex min-h-screen items-center justify-center">
		<div class="text-center">
			<div class="relative h-12 w-12 mx-auto">
				<div class="absolute inset-0 rounded-full border-[3px]" style="border-color: var(--color-border-default);"></div>
				<div class="absolute inset-0 rounded-full border-[3px] border-transparent animate-spin" style="border-top-color: var(--color-brand-primary);"></div>
			</div>
			<p class="mt-5 text-sm font-medium tracking-tight" style="color: var(--color-text-tertiary);">Bominal</p>
		</div>
	</div>
{:else if isAuthPage || isRootPage || $page.url.pathname === '/verify-email' || $page.url.pathname === '/reset-password'}
	<main class="min-h-screen">
		{@render children()}
	</main>
{:else}
	<div class="flex min-h-screen">
		<div class="hidden md:block">
			<Sidebar />
		</div>
		<main class="flex-1 flex flex-col min-h-screen">
			{@render children()}
		</main>
		<div class="md:hidden">
			<BottomNav />
		</div>
	</div>
{/if}
