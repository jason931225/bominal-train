//! Date picker modal component — pure WASM calendar grid.

use chrono::{Datelike, Local, NaiveDate};
use leptos::prelude::*;

/// A calendar date picker modal.
#[component]
pub fn DatePicker(
    /// Whether the modal is open.
    open: ReadSignal<bool>,
    /// Callback when a date is selected.
    on_select: Callback<NaiveDate>,
    /// Callback to close the modal.
    on_close: Callback<()>,
    /// Currently selected date (if any).
    #[prop(optional)]
    selected: Option<NaiveDate>,
) -> impl IntoView {
    let today = Local::now().date_naive();
    let (year, set_year) = signal(selected.unwrap_or(today).year());
    let (month, set_month) = signal(selected.unwrap_or(today).month());

    let prev_month = move |_| {
        if month.get() == 1 {
            set_month.set(12);
            set_year.set(year.get() - 1);
        } else {
            set_month.set(month.get() - 1);
        }
    };

    let next_month = move |_| {
        if month.get() == 12 {
            set_month.set(1);
            set_year.set(year.get() + 1);
        } else {
            set_month.set(month.get() + 1);
        }
    };

    let calendar_days = move || {
        let y = year.get();
        let m = month.get();
        let first = NaiveDate::from_ymd_opt(y, m, 1).unwrap();
        let start_weekday = first.weekday().num_days_from_sunday();
        let days_in_month = if m == 12 {
            NaiveDate::from_ymd_opt(y + 1, 1, 1)
        } else {
            NaiveDate::from_ymd_opt(y, m + 1, 1)
        }
        .unwrap()
        .signed_duration_since(first)
        .num_days() as u32;

        let mut cells: Vec<Option<NaiveDate>> = Vec::new();
        for _ in 0..start_weekday {
            cells.push(None);
        }
        for d in 1..=days_in_month {
            cells.push(NaiveDate::from_ymd_opt(y, m, d));
        }
        cells
    };

    let month_names = [
        "", "January", "February", "March", "April", "May", "June",
        "July", "August", "September", "October", "November", "December",
    ];

    view! {
        <Show when=move || open.get()>
            <div class="fixed inset-0 z-50 flex items-center justify-center page-enter"
                 on:click=move |_| on_close.run(())>
                <div class="absolute inset-0 bg-black/40 backdrop-blur-sm"></div>
                <div class="glass-panel rounded-3xl p-4 w-80 relative z-10 modal-enter"
                     on:click=move |e| e.stop_propagation()>
                    // Header
                    <div class="flex items-center justify-between mb-4">
                        <button class="p-2 rounded-lg hover:bg-[var(--theme-surface-hover)] text-[var(--theme-text-primary)]"
                                on:click=prev_month>
                            "\u{2190}"
                        </button>
                        <span class="font-semibold text-[var(--theme-text-strong)]">
                            {move || format!("{} {}", month_names[month.get() as usize], year.get())}
                        </span>
                        <button class="p-2 rounded-lg hover:bg-[var(--theme-surface-hover)] text-[var(--theme-text-primary)]"
                                on:click=next_month>
                            "\u{2192}"
                        </button>
                    </div>
                    // Day headers
                    <div class="grid grid-cols-7 gap-1 mb-1">
                        {["S","M","T","W","T","F","S"].iter().map(|d| view! {
                            <div class="text-center text-xs text-[var(--theme-text-subtle)] font-medium py-1">{*d}</div>
                        }).collect::<Vec<_>>()}
                    </div>
                    // Calendar grid
                    <div class="grid grid-cols-7 gap-1">
                        {move || calendar_days().into_iter().map(|cell| {
                            match cell {
                                None => view! { <div class="h-9"></div> }.into_any(),
                                Some(date) => {
                                    let is_today = date == today;
                                    let is_selected = selected == Some(date);
                                    let is_past = date < today;
                                    let cls = if is_selected {
                                        "h-9 w-full rounded-lg flex items-center justify-center text-sm font-medium bg-[var(--theme-accent-solid)] text-white cursor-pointer"
                                    } else if is_today {
                                        "h-9 w-full rounded-lg flex items-center justify-center text-sm font-medium ring-1 ring-[var(--theme-accent-solid)] text-[var(--theme-accent-text)] cursor-pointer hover:bg-[var(--theme-accent-soft)]"
                                    } else if is_past {
                                        "h-9 w-full rounded-lg flex items-center justify-center text-sm text-[var(--theme-text-subtle)] opacity-40"
                                    } else {
                                        "h-9 w-full rounded-lg flex items-center justify-center text-sm text-[var(--theme-text-primary)] cursor-pointer hover:bg-[var(--theme-surface-hover)]"
                                    };
                                    let on_click = move |_| {
                                        if !is_past {
                                            on_select.run(date);
                                        }
                                    };
                                    view! {
                                        <button class=cls on:click=on_click disabled=is_past>
                                            {date.day()}
                                        </button>
                                    }.into_any()
                                }
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>
            </div>
        </Show>
    }
}
