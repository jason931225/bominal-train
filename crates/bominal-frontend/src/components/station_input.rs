//! Station autocomplete input — text input with suggest API dropdown.

use leptos::ev;
use leptos::prelude::*;

use crate::api::search::{StationInfo, suggest_stations};
use crate::i18n::t;

/// An autocomplete text input for station selection.
///
/// Uses the suggest server function for ranked, autocorrected results when the
/// user is typing, and falls back to the full station list when the query is
/// empty (e.g. on focus).
#[component]
pub fn StationInput(
    /// Visible label text (e.g. "FROM" / "TO").
    label: &'static str,
    /// HTML id for the input element (used for label-for association).
    id: &'static str,
    /// The currently selected station name (Korean).
    value: ReadSignal<String>,
    /// Callback when a station is selected.
    set_value: WriteSignal<String>,
    /// Async station list resource (full list, used when query is empty).
    stations: Resource<Result<Vec<StationInfo>, ServerFnError>>,
    /// Current provider ("SRT" / "KTX").
    provider: ReadSignal<String>,
) -> impl IntoView {
    // Local text in the input (may differ from confirmed `value` while typing).
    let (query, set_query) = signal(String::new());
    // Debounced query for suggest API calls.
    let (debounced, set_debounced) = signal(String::new());
    // Whether the dropdown is open.
    let (open, set_open) = signal(false);
    // Index of keyboard-highlighted option (-1 = none).
    let (highlight, set_highlight) = signal(-1i32);

    // Sync input text when the confirmed value changes externally.
    Effect::new(move || {
        let v = value.get();
        set_query.set(v);
    });

    // Debounce: update `debounced` 150ms after `query` changes.
    Effect::new(move || {
        let q = query.get();
        if q.is_empty() {
            set_debounced.set(String::new());
            return;
        }
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use wasm_bindgen::closure::Closure;
            let cb = Closure::once(move || {
                set_debounced.set(q);
            });
            let _ = web_sys::window()
                .unwrap()
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    cb.as_ref().unchecked_ref(),
                    150,
                );
            cb.forget();
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            // SSR: server functions are local calls, no debounce needed.
            set_debounced.set(q);
        }
    });

    // Suggest resource — fires when debounced query or provider changes.
    let suggest = Resource::new(
        move || (provider.get(), debounced.get()),
        |(prov, q)| async move {
            if q.is_empty() {
                return Ok(crate::api::search::SuggestResult {
                    matches: vec![],
                    corrected_query: None,
                    autocorrect_applied: false,
                });
            }
            suggest_stations(prov, q, None).await
        },
    );

    /// Unified dropdown item for display.
    #[derive(Clone)]
    struct DropdownItem {
        name_ko: String,
    }

    // Merged dropdown items: suggest results when typing, full list when empty.
    let dropdown_items = move || -> Vec<DropdownItem> {
        let q = query.get();
        if q.is_empty() {
            // Show full station list.
            let Some(Ok(list)) = stations.get() else {
                return vec![];
            };
            return list
                .into_iter()
                .map(|s| DropdownItem { name_ko: s.name_ko })
                .collect();
        }
        // Show suggest results.
        let Some(Ok(result)) = suggest.get() else {
            return vec![];
        };
        result
            .matches
            .into_iter()
            .map(|m| DropdownItem { name_ko: m.name_ko })
            .collect()
    };

    // Corrected query hint (shown when autocorrect was applied).
    let corrected_hint = move || -> Option<String> {
        let q = query.get();
        if q.is_empty() {
            return None;
        }
        let Some(Ok(result)) = suggest.get() else {
            return None;
        };
        if result.autocorrect_applied {
            result.corrected_query
        } else {
            None
        }
    };

    let listbox_id = format!("{id}-listbox");
    let listbox_id_attr = listbox_id.clone();

    let on_input = move |ev: ev::Event| {
        let text = event_target_value(&ev);
        set_query.set(text);
        set_open.set(true);
        set_highlight.set(-1);
    };

    let on_focus = move |_| {
        set_open.set(true);
    };

    // Close dropdown on blur. Options use on:mousedown with
    // prevent_default(), which prevents the blur from firing when
    // clicking an option, so the select handler runs first.
    let on_blur = move |_| {
        set_open.set(false);
        // Revert to confirmed value if nothing was selected.
        if query.get_untracked() != value.get_untracked() {
            set_query.set(value.get_untracked());
        }
    };

    let select_station = move |name: String| {
        set_value.set(name.clone());
        set_query.set(name);
        set_open.set(false);
        set_highlight.set(-1);
    };

    let on_keydown = move |ev: ev::KeyboardEvent| {
        let key = ev.key();
        let list = dropdown_items();
        let len = list.len() as i32;
        match key.as_str() {
            "ArrowDown" => {
                ev.prevent_default();
                set_open.set(true);
                set_highlight.update(|h| *h = (*h + 1).min(len - 1));
            }
            "ArrowUp" => {
                ev.prevent_default();
                set_highlight.update(|h| *h = (*h - 1).max(-1));
            }
            "Enter" => {
                ev.prevent_default();
                let h = highlight.get_untracked();
                if h >= 0 && (h as usize) < list.len() {
                    select_station(list[h as usize].name_ko.clone());
                }
            }
            "Escape" => {
                set_open.set(false);
                set_query.set(value.get_untracked());
            }
            _ => {}
        }
    };

    view! {
        <div class="relative">
            <label for=id class="block text-xs font-semibold text-[var(--color-text-secondary)] uppercase tracking-wide mb-1.5">
                {label}
            </label>
            <div class="flex items-center bg-[var(--color-bg-sunken)] border border-[var(--color-border-default)] rounded-xl focus-within:border-[var(--color-border-focus)] transition-colors">
                <div class="pl-3">
                    <svg aria-hidden="true" class="w-4 h-4 text-[var(--color-text-tertiary)]" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z" />
                        <path stroke-linecap="round" stroke-linejoin="round" d="M15 11a3 3 0 11-6 0 3 3 0 016 0z" />
                    </svg>
                </div>
                <input
                    id=id
                    type="text"
                    role="combobox"
                    autocomplete="off"
                    aria-expanded=move || if open.get() { "true" } else { "false" }
                    aria-controls=listbox_id_attr.clone()
                    aria-activedescendant=move || {
                        let h = highlight.get();
                        if h >= 0 { format!("{id}-opt-{h}") } else { String::new() }
                    }
                    placeholder=t("search.select_station")
                    prop:value=move || query.get()
                    on:input=on_input
                    on:focus=on_focus
                    on:blur=on_blur
                    on:keydown=on_keydown
                    class="w-full px-2 py-2.5 bg-transparent text-sm text-[var(--color-text-primary)] focus:outline-none"
                />
            </div>

            // Autocorrect hint
            {move || corrected_hint().map(|hint| view! {
                <div class="mt-1 px-2 text-xs text-[var(--color-text-tertiary)] italic">
                    {format!("\"{}\" →", hint)}
                </div>
            })}

            // Dropdown
            <Show when=move || open.get() && !dropdown_items().is_empty()>
                <ul
                    id=listbox_id.clone()
                    role="listbox"
                    class="absolute z-50 left-0 right-0 mt-1 max-h-60 overflow-y-auto rounded-xl glass-panel border border-[var(--color-border-default)] shadow-lg no-scrollbar"
                >
                    {move || dropdown_items().into_iter().enumerate().map(|(idx, item)| {
                        let name = item.name_ko.clone();
                        let name_click = name.clone();
                        let is_selected = value.get() == name;
                        let option_id = format!("{id}-opt-{idx}");
                        view! {
                            <li
                                id=option_id
                                role="option"
                                aria-selected=if is_selected { "true" } else { "false" }
                                class=move || {
                                    let h = highlight.get() == idx as i32;
                                    if h {
                                        "px-3 py-2.5 text-sm cursor-pointer bg-[var(--color-interactive-hover)] text-[var(--color-text-primary)]"
                                    } else if is_selected {
                                        "px-3 py-2.5 text-sm cursor-pointer text-[var(--color-brand-text)] font-medium"
                                    } else {
                                        "px-3 py-2.5 text-sm cursor-pointer text-[var(--color-text-primary)] hover:bg-[var(--color-interactive-hover)]"
                                    }
                                }
                                on:mousedown=move |ev| {
                                    ev.prevent_default();
                                    select_station(name_click.clone());
                                }
                                on:mouseenter=move |_| set_highlight.set(idx as i32)
                            >
                                {name}
                            </li>
                        }
                    }).collect::<Vec<_>>()}
                </ul>
            </Show>
        </div>
    }.into_any()
}
