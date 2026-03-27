<script lang="ts">
	import type { Snippet } from 'svelte';
	import { t } from '$lib/i18n';

	interface Props {
		open: boolean;
		onclose: () => void;
		title?: string;
		children: Snippet;
	}

	const { open, onclose, title, children }: Props = $props();

	let dragStartY = $state(0);
	let dragDeltaY = $state(0);
	let isDragging = $state(false);
	let dialogEl: HTMLElement | undefined = $state();

	const DISMISS_THRESHOLD = 100;

	function handleKeydown(e: KeyboardEvent): void {
		if (e.key === 'Escape') {
			onclose();
		}
		// Focus trap: cycle focus within the dialog
		if (e.key === 'Tab' && dialogEl) {
			const focusable = dialogEl.querySelectorAll<HTMLElement>(
				'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
			);
			if (focusable.length === 0) return;
			const first = focusable[0];
			const last = focusable[focusable.length - 1];
			if (e.shiftKey && document.activeElement === first) {
				e.preventDefault();
				last.focus();
			} else if (!e.shiftKey && document.activeElement === last) {
				e.preventDefault();
				first.focus();
			}
		}
	}

	function handleBackdropClick(e: MouseEvent): void {
		if (e.target === e.currentTarget) {
			onclose();
		}
	}

	function handleTouchStart(e: TouchEvent): void {
		dragStartY = e.touches[0].clientY;
		dragDeltaY = 0;
		isDragging = true;
	}

	function handleTouchMove(e: TouchEvent): void {
		if (!isDragging) return;
		const delta = e.touches[0].clientY - dragStartY;
		dragDeltaY = Math.max(0, delta);
	}

	function handleTouchEnd(): void {
		if (dragDeltaY > DISMISS_THRESHOLD) {
			onclose();
		}
		dragDeltaY = 0;
		isDragging = false;
	}

	const sheetTransform = $derived(
		isDragging && dragDeltaY > 0
			? `translateY(${dragDeltaY}px)`
			: 'translateY(0)'
	);
</script>

{#if open}
	<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
	<div
		class="fixed inset-0 z-50 flex items-end md:items-center md:justify-center"
		style="background: rgba(0,0,0,0.4); backdrop-filter: blur(4px)"
		role="dialog"
		aria-modal="true"
		aria-label={title ?? ''}
		onclick={handleBackdropClick}
		onkeydown={handleKeydown}
		bind:this={dialogEl}
	>
		<!-- Mobile: bottom sheet -->
		<div
			class="lg-glass-sheet w-full md:hidden max-h-[85vh] flex flex-col overflow-hidden sheet-enter"
			style="
				border-bottom: none;
				transform: {sheetTransform};
				transition: {isDragging ? 'none' : 'transform 0.3s var(--lg-ease-out)'};
			"
			onclick={(e) => e.stopPropagation()}
			onkeydown={handleKeydown}
			role="document"
		>
			<!-- Drag handle -->
			<div
				class="flex justify-center pt-3 pb-1 cursor-grab"
				ontouchstart={handleTouchStart}
				ontouchmove={handleTouchMove}
				ontouchend={handleTouchEnd}
				role="presentation"
			>
				<div
					class="w-10 h-[5px] rounded-full"
					style="background: var(--lg-light-catch-subtle); backdrop-filter: blur(4px);"
				></div>
			</div>

			<!-- Header -->
			{#if title}
				<div class="flex items-center justify-between px-5 py-3">
					<h2 class="text-base font-semibold" style="color: var(--color-text-primary)">{title}</h2>
					<button
						type="button"
						class="p-1.5 rounded-full transition-colors hover:bg-[var(--color-interactive-hover)]"
						onclick={onclose}
						aria-label={t('common.close')}
					>
						<svg class="w-5 h-5" style="color: var(--color-text-tertiary)" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
							<line x1="18" y1="6" x2="6" y2="18" />
							<line x1="6" y1="6" x2="18" y2="18" />
						</svg>
					</button>
				</div>
			{/if}

			<!-- Content -->
			<div class="flex-1 overflow-y-auto px-5 pb-6 {title ? '' : 'pt-2'}">
				{@render children()}
			</div>
		</div>

		<!-- Desktop: centered modal -->
		<div
			class="lg-glass-sheet hidden md:flex flex-col w-full max-w-lg max-h-[80vh] rounded-2xl overflow-hidden modal-enter"
			onclick={(e) => e.stopPropagation()}
			onkeydown={handleKeydown}
			role="document"
		>
			<!-- Header -->
			<div class="flex items-center justify-between px-6 py-4">
				{#if title}
					<h2 class="text-base font-semibold" style="color: var(--color-text-primary)">{title}</h2>
				{:else}
					<div></div>
				{/if}
				<button
					type="button"
					class="p-1.5 rounded-full transition-colors hover:bg-[var(--color-interactive-hover)]"
					onclick={onclose}
					aria-label={t('common.close')}
				>
					<svg class="w-5 h-5" style="color: var(--color-text-tertiary)" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
						<line x1="18" y1="6" x2="6" y2="18" />
						<line x1="6" y1="6" x2="18" y2="18" />
					</svg>
				</button>
			</div>

			<!-- Content -->
			<div class="flex-1 overflow-y-auto px-6 pb-6">
				{@render children()}
			</div>
		</div>
	</div>
{/if}
