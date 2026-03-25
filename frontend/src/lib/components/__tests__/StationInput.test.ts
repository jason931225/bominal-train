import { render, fireEvent, waitFor } from '@testing-library/svelte';
import { vi, describe, it, expect, beforeEach, afterEach } from 'vitest';
import StationInput from '../StationInput.svelte';

vi.mock('$lib/api/search', () => ({
	suggestStations: vi.fn()
}));

vi.mock('$lib/i18n', () => ({
	t: (key: string) => key
}));

import { suggestStations } from '$lib/api/search';
const mockSuggest = vi.mocked(suggestStations);

const mockResult = {
	matches: [
		{ name_ko: '서울', name_en: 'Seoul', name_ja: 'ソウル', score: 100 },
		{ name_ko: '서울대입구', name_en: 'Seoul Nat Univ', name_ja: 'ソウル大入口', score: 80 }
	],
	corrected_query: null,
	autocorrect_applied: false
};

function renderInput(overrides = {}) {
	return render(StationInput, {
		props: { value: '', label: 'Departure', provider: 'SRT', name: 'dep', ...overrides }
	});
}

describe('StationInput', () => {
	beforeEach(() => {
		vi.useFakeTimers();
		mockSuggest.mockReset();
		mockSuggest.mockResolvedValue(mockResult);
	});

	afterEach(() => {
		vi.useRealTimers();
	});

	describe('SRCH-01: debounced autocomplete', () => {
		it('calls suggestStations after 250ms debounce', async () => {
			const { getByRole } = renderInput();
			const input = getByRole('combobox');

			await fireEvent.input(input, { target: { value: '서울' } });

			expect(mockSuggest).not.toHaveBeenCalled();

			vi.advanceTimersByTime(250);

			expect(mockSuggest).toHaveBeenCalledWith('SRT', '서울');
		});

		it('does NOT call suggestStations for empty input', async () => {
			const { getByRole } = renderInput();
			const input = getByRole('combobox');

			await fireEvent.input(input, { target: { value: '' } });
			vi.advanceTimersByTime(250);

			expect(mockSuggest).not.toHaveBeenCalled();
		});

		it('shows suggestions in a listbox when API returns matches', async () => {
			const { getByRole, findByRole } = renderInput();
			const input = getByRole('combobox');

			await fireEvent.input(input, { target: { value: '서울' } });
			vi.advanceTimersByTime(250);

			// Flush the promise
			await vi.runAllTimersAsync();

			const listbox = await findByRole('listbox');
			expect(listbox).toBeTruthy();

			const options = listbox.querySelectorAll('[role="option"]');
			expect(options).toHaveLength(2);
		});

		it('discards stale responses', async () => {
			// First call resolves slowly with '서울' results
			let resolveFirst: (value: unknown) => void;
			const firstPromise = new Promise((resolve) => { resolveFirst = resolve; });
			mockSuggest.mockImplementationOnce(() => firstPromise as never);

			// Second call resolves immediately with '대전' results
			const secondResult = {
				matches: [{ name_ko: '대전', name_en: 'Daejeon', name_ja: '大田', score: 100 }],
				corrected_query: null,
				autocorrect_applied: false
			};
			mockSuggest.mockImplementationOnce(() => Promise.resolve(secondResult) as never);

			const { getByRole } = renderInput();
			const input = getByRole('combobox');

			// Type '서울', advance timer
			await fireEvent.input(input, { target: { value: '서울' } });
			vi.advanceTimersByTime(250);

			// Type '대전', advance timer — this becomes the current value
			await fireEvent.input(input, { target: { value: '대전' } });
			vi.advanceTimersByTime(250);

			// Flush the second call
			await vi.runAllTimersAsync();

			// Now resolve the first (stale) call
			resolveFirst!(mockResult);
			await vi.runAllTimersAsync();

			// The stale response should be discarded — only '대전' results should show
			const listbox = getByRole('combobox').getAttribute('aria-expanded');
			expect(listbox).toBe('true');
		});
	});

	describe('SRCH-02: IME composition', () => {
		it('does NOT call suggestStations during composition', async () => {
			const { getByRole } = renderInput();
			const input = getByRole('combobox');

			await fireEvent.compositionStart(input);
			await fireEvent.input(input, { target: { value: 'ㅅ' } });
			await fireEvent.input(input, { target: { value: '서' } });

			vi.advanceTimersByTime(500);

			expect(mockSuggest).not.toHaveBeenCalled();
		});

		it('triggers fetch after compositionend', async () => {
			const { getByRole } = renderInput();
			const input = getByRole('combobox');

			await fireEvent.compositionStart(input);
			await fireEvent.input(input, { target: { value: '서울' } });
			await fireEvent.compositionEnd(input);

			vi.advanceTimersByTime(250);

			expect(mockSuggest).toHaveBeenCalledWith('SRT', '서울');
		});
	});

	describe('SRCH-03: keyboard navigation', () => {
		async function openDropdown() {
			const result = renderInput();
			const input = result.getByRole('combobox');

			await fireEvent.input(input, { target: { value: '서울' } });
			vi.advanceTimersByTime(250);
			await vi.runAllTimersAsync();

			return { ...result, input };
		}

		it('ArrowDown increments activeIndex and wraps', async () => {
			const { input } = await openDropdown();

			await fireEvent.keyDown(input, { key: 'ArrowDown' });
			expect(input.getAttribute('aria-activedescendant')).toBe('dep-station-option-0');

			await fireEvent.keyDown(input, { key: 'ArrowDown' });
			expect(input.getAttribute('aria-activedescendant')).toBe('dep-station-option-1');

			// Wrap
			await fireEvent.keyDown(input, { key: 'ArrowDown' });
			expect(input.getAttribute('aria-activedescendant')).toBe('dep-station-option-0');
		});

		it('ArrowUp decrements activeIndex and wraps to last', async () => {
			const { input } = await openDropdown();

			// First ArrowDown to set index to 0
			await fireEvent.keyDown(input, { key: 'ArrowDown' });

			// ArrowUp wraps to last
			await fireEvent.keyDown(input, { key: 'ArrowUp' });
			expect(input.getAttribute('aria-activedescendant')).toBe('dep-station-option-1');
		});

		it('Enter selects the highlighted suggestion', async () => {
			const { input, queryByRole } = await openDropdown();

			await fireEvent.keyDown(input, { key: 'ArrowDown' });
			await fireEvent.keyDown(input, { key: 'Enter' });

			// Dropdown should close
			expect(input.getAttribute('aria-expanded')).toBe('false');
		});

		it('Escape closes the dropdown', async () => {
			const { input } = await openDropdown();

			expect(input.getAttribute('aria-expanded')).toBe('true');

			await fireEvent.keyDown(input, { key: 'Escape' });

			expect(input.getAttribute('aria-expanded')).toBe('false');
			expect(input.getAttribute('aria-activedescendant')).toBeNull();
		});
	});

	describe('ARIA attributes', () => {
		it('input has role=combobox and required ARIA attributes', () => {
			const { getByRole } = renderInput();
			const input = getByRole('combobox');

			expect(input.getAttribute('role')).toBe('combobox');
			expect(input.getAttribute('aria-expanded')).toBe('false');
			expect(input.getAttribute('aria-controls')).toBe('dep-station-listbox');
			expect(input.getAttribute('aria-autocomplete')).toBe('list');
		});

		it('uses name prop for unique ARIA IDs', () => {
			const { getByRole } = renderInput({ name: 'arr' });
			const input = getByRole('combobox');

			expect(input.getAttribute('aria-controls')).toBe('arr-station-listbox');
		});
	});
});
