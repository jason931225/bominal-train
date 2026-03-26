<script lang="ts">
	import { page } from '$app/stores';
	import { t } from '$lib/i18n';

	interface NavItem {
		href: string;
		label: string;
		icon: string;
	}

	const navItems: NavItem[] = [
		{
			href: '/home',
			label: 'nav.home',
			icon: 'M3 12.5l9-9 9 9M5 11v8a1 1 0 001 1h3.5v-5a1 1 0 011-1h3a1 1 0 011 1v5H18a1 1 0 001-1v-8'
		},
		{
			href: '/search',
			label: 'nav.search',
			icon: 'M21 21l-4.35-4.35M11 19a8 8 0 100-16 8 8 0 000 16z'
		},
		{
			href: '/tasks',
			label: 'nav.tasks',
			icon: 'M9 5h7a2 2 0 012 2v12a2 2 0 01-2 2H8a2 2 0 01-2-2V7a2 2 0 012-2h1m1-2h4a1 1 0 011 1v1H9V4a1 1 0 011-1zM9 12h6M9 16h4'
		},
		{
			href: '/reservations',
			label: 'nav.reservations',
			icon: 'M15 5v2m0 10v2M4 7h16a1 1 0 011 1v8a1 1 0 01-1 1H4a1 1 0 01-1-1V8a1 1 0 011-1zm4 4h.01M8 13h.01M12 11h.01M12 13h.01M16 11h.01M16 13h.01'
		},
		{
			href: '/settings',
			label: 'nav.settings',
			icon: 'M12 15a3 3 0 100-6 3 3 0 000 6zm7.94-2.06a1.06 1.06 0 00.21-.67V11.73a1.06 1.06 0 00-.21-.67l-1.48-1.28a7.84 7.84 0 00-.65-1.58l.27-1.91a1.06 1.06 0 00-.32-.62l-1.09-1.1a1.06 1.06 0 00-.62-.32l-1.91.27a7.84 7.84 0 00-1.58-.65L11.27 2.4a1.06 1.06 0 00-.67-.21H10.06a1.06 1.06 0 00-.67.21L8.11 3.88a7.84 7.84 0 00-1.58.65l-1.91-.27a1.06 1.06 0 00-.62.32L2.9 5.67a1.06 1.06 0 00-.32.62l.27 1.91a7.84 7.84 0 00-.65 1.58L.72 11.06a1.06 1.06 0 00-.21.67v.54a1.06 1.06 0 00.21.67l1.48 1.28c.15.55.37 1.08.65 1.58l-.27 1.91c-.02.23.08.45.32.62l1.09 1.1c.17.24.39.34.62.32l1.91-.27c.5.28 1.03.5 1.58.65l1.28 1.48c.18.14.42.21.67.21h.54'
		}
	];

	const authPaths = ['/auth', '/verify-email', '/reset-password'];
	let isHidden = $derived(
		authPaths.some(
			(p) => $page.url.pathname === p || $page.url.pathname.startsWith(p + '/')
		) || $page.url.pathname === '/'
	);

	function isActive(href: string): boolean {
		const pathname = $page.url.pathname;
		if (href === '/home') return pathname === '/home' || pathname === '/';
		return pathname === href || pathname.startsWith(href + '/');
	}
</script>

{#if !isHidden}
	<nav
		class="lg-bottom-nav fixed bottom-0 left-0 right-0 flex items-center justify-around px-2 pt-2 safe-area-pb"
		style="z-index: 50; border-radius: 0;"
	>
		{#each navItems as item}
			{@const active = isActive(item.href)}
			<a
				href={item.href}
				class="flex flex-col items-center justify-center flex-1 py-2 squish"
				aria-current={active ? 'page' : undefined}
			>
				<svg
					width="24"
					height="24"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="1.8"
					stroke-linecap="round"
					stroke-linejoin="round"
					class="transition-colors duration-200"
					style="color: {active
						? 'var(--color-brand-text)'
						: 'var(--color-text-tertiary)'};"
				>
					<path d={item.icon} />
				</svg>
				<span
					class="text-[10px] mt-0.5 font-medium leading-tight transition-colors duration-200"
					style="color: {active
						? 'var(--color-brand-text)'
						: 'var(--color-text-tertiary)'};"
				>
					{t(item.label)}
				</span>
			</a>
		{/each}
	</nav>
{/if}
