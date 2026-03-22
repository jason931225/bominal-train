//! Station autocomplete input — text input with filterable dropdown.

use leptos::ev;
use leptos::prelude::*;

use crate::i18n::t;

/// An autocomplete text input for station selection.
///
/// Fetches the full station list via a `Resource`, then filters client-side
/// using Korean substring matching.  Supports keyboard navigation (arrow
/// keys, Enter, Escape) and the ARIA combobox pattern.
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
    /// Async station list resource.
    stations: Resource<Result<Vec<crate::api::search::StationInfo>, ServerFnError>>,
) -> impl IntoView {
    // Local text in the input (may differ from confirmed `value` while typing).
    let (query, set_query) = signal(String::new());
    // Whether the dropdown is open.
    let (open, set_open) = signal(false);
    // Index of keyboard-highlighted option (-1 = none).
    let (highlight, set_highlight) = signal(-1i32);

    // Sync input text when the confirmed value changes externally.
    Effect::new(move || {
        let v = value.get();
        set_query.set(v);
    });

    // Filtered stations list.
    let filtered = move || {
        let q = query.get();
        let Some(Ok(list)) = stations.get() else {
            return vec![];
        };
        if q.is_empty() {
            return list;
        }
        list.into_iter()
            .filter(|s| s.name_ko.contains(&q))
            .collect::<Vec<_>>()
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
        let list = filtered();
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

            // Dropdown
            <Show when=move || open.get() && !filtered().is_empty()>
                <ul
                    id=listbox_id.clone()
                    role="listbox"
                    class="absolute z-50 left-0 right-0 mt-1 max-h-60 overflow-y-auto rounded-xl glass-panel border border-[var(--color-border-default)] shadow-lg no-scrollbar"
                >
                    {move || filtered().into_iter().enumerate().map(|(idx, station)| {
                        let name = station.name_ko.clone();
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
