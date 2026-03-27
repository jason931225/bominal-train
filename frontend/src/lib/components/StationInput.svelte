<script lang="ts">
	import { suggestStations } from '$lib/api/search';
	import { t } from '$lib/i18n';
	import type { SuggestMatch } from '$lib/types';

	interface Props {
		value: string;
		label: string;
		placeholder?: string;
		provider: string;
		name: string;
		onselect?: (match: SuggestMatch) => void;
	}

	let {
		value = $bindable(),
		label,
		placeholder = '',
		provider,
		name,
		onselect
	}: Props = $props();

	// Suggest state
	let suggestions = $state<SuggestMatch[]>([]);
	let expanded = $state(false);
	let activeIndex = $state(-1);
	let loading = $state(false);

	// IME composition state
	let isComposing = $state(false);
	let debounceTimer: ReturnType<typeof setTimeout> | undefined;

	// Cache for re-focus behavior (D-06)
	let lastQuery = $state('');
	let cachedSuggestions = $state<SuggestMatch[]>([]);

	// ARIA IDs (unique per instance via name prop)
	const listboxId = $derived(`${name}-station-listbox`);
	const activeDescendant = $derived(
		activeIndex >= 0 ? `${name}-station-option-${activeIndex}` : undefined
	);

	// Clear cache when provider changes
	$effect(() => {
		void provider;
		cachedSuggestions = [];
		suggestions = [];
		expanded = false;
		activeIndex = -1;
	});

	function scheduleFetch(query: string): void {
		clearTimeout(debounceTimer);
		if (query.length < 1) {
			suggestions = [];
			expanded = false;
			loading = false;
			return;
		}
		loading = true;
		debounceTimer = setTimeout(() => fetchSuggestions(query), 250);
	}

	async function fetchSuggestions(query: string): Promise<void> {
		try {
			const result = await suggestStations(provider, query);
			if (query !== value) return; // Discard stale responses
			suggestions = result.matches;
			cachedSuggestions = result.matches;
			lastQuery = query;
			expanded = suggestions.length > 0;
			activeIndex = -1;
		} catch {
			suggestions = [];
		} finally {
			loading = false;
		}
	}

	function handleInput(e: Event): void {
		const query = (e.target as HTMLInputElement).value;
		value = query;
		if (isComposing) return;
		scheduleFetch(query);
	}

	function handleCompositionStart(): void {
		isComposing = true;
	}

	function handleCompositionEnd(): void {
		isComposing = false;
		scheduleFetch(value);
	}

	function handleFocus(): void {
		if (cachedSuggestions.length > 0 && value === lastQuery) {
			suggestions = cachedSuggestions;
			expanded = true;
		}
	}

	function handleBlur(): void {
		setTimeout(() => {
			expanded = false;
			activeIndex = -1;
		}, 200);
	}

	function selectStation(match: SuggestMatch): void {
		value = match.name_ko;
		expanded = false;
		activeIndex = -1;
		onselect?.(match);
	}

	function handleKeydown(e: KeyboardEvent): void {
		if (!expanded || suggestions.length === 0) {
			if (e.key === 'ArrowDown' && suggestions.length > 0) {
				expanded = true;
				activeIndex = 0;
				e.preventDefault();
			}
			return;
		}
		switch (e.key) {
			case 'ArrowDown':
				activeIndex = (activeIndex + 1) % suggestions.length;
				scrollActiveIntoView();
				e.preventDefault();
				break;
			case 'ArrowUp':
				activeIndex = activeIndex <= 0 ? suggestions.length - 1 : activeIndex - 1;
				scrollActiveIntoView();
				e.preventDefault();
				break;
			case 'Enter':
				if (activeIndex >= 0 && activeIndex < suggestions.length) {
					selectStation(suggestions[activeIndex]);
					e.preventDefault();
				}
				break;
			case 'Escape':
				expanded = false;
				activeIndex = -1;
				e.preventDefault();
				break;
		}
	}

	function scrollActiveIntoView(): void {
		requestAnimationFrame(() => {
			const el = document.getElementById(`${name}-station-option-${activeIndex}`);
			el?.scrollIntoView({ block: 'nearest' });
		});
	}
</script>

<div class="flex-1 relative">
	<label
		class="text-xs font-medium mb-1 block"
		style="color: var(--color-text-tertiary)"
		for="{name}-station-input"
	>
		{label}
	</label>
	<div class="relative">
		<input
			id="{name}-station-input"
			type="text"
			role="combobox"
			aria-expanded={expanded}
			aria-controls={listboxId}
			aria-activedescendant={activeDescendant}
			aria-autocomplete="list"
			class="w-full rounded-xl px-3 py-2.5 text-sm font-medium outline-none"
			style="background: var(--color-bg-sunken); color: var(--color-text-primary); border: 1px solid var(--color-border-default)"
			{placeholder}
			{value}
			oninput={handleInput}
			onkeydown={handleKeydown}
			onfocus={handleFocus}
			onblur={handleBlur}
			oncompositionstart={handleCompositionStart}
			oncompositionend={handleCompositionEnd}
		/>
		{#if loading}
			<div class="absolute right-3 top-1/2 -translate-y-1/2" aria-label={t('search.loading_stations')}>
				<svg class="animate-spin w-4 h-4" viewBox="0 0 24 24" fill="none">
					<circle cx="12" cy="12" r="10" stroke="var(--color-brand-text)" stroke-width="2" opacity="0.25" />
					<path d="M12 2a10 10 0 0 1 10 10" stroke="var(--color-brand-text)" stroke-width="2" stroke-linecap="round" />
				</svg>
			</div>
		{/if}
	</div>
	{#if expanded}
		<ul
			id={listboxId}
			role="listbox"
			class="absolute z-20 left-0 right-0 mt-1 glass-panel rounded-xl overflow-hidden max-h-48 overflow-y-auto"
		>
			{#if suggestions.length === 0}
				<li class="px-3 py-2.5 text-sm" style="color: var(--color-text-tertiary)">
					{t('search.no_station_match')}
				</li>
			{:else}
				{#each suggestions as match, i}
					<li
						id="{name}-station-option-{i}"
						role="option"
						aria-selected={i === activeIndex}
					>
						<button
							type="button"
							class="w-full text-left px-3 py-2.5 text-sm transition-colors"
							class:bg-[var(--color-interactive-hover)]={i === activeIndex}
							style="color: var(--color-text-primary)"
							onmousedown={() => selectStation(match)}
						>
							{match.name_ko}
							<span class="text-xs ml-1" style="color: var(--color-text-tertiary)">
								{match.name_en}
							</span>
						</button>
					</li>
				{/each}
			{/if}
		</ul>
	{/if}
</div>
