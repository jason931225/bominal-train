//! Date & time picker modal — calendar grid with time slider.

use chrono::{Datelike, Duration, NaiveDate, Utc};
use leptos::prelude::*;

use crate::components::time_slider::TimeSlider;
use crate::i18n::t;

/// Get today's date in KST (UTC+9).
fn today_kst() -> NaiveDate {
    (Utc::now() + Duration::hours(9)).date_naive()
}

/// Get current hour in KST.
fn kst_hour() -> u32 {
    let kst = Utc::now() + Duration::hours(9);
    kst.time().hour()
}

/// Max selectable date: 1 month from today KST.
fn max_selectable_date() -> NaiveDate {
    let today = today_kst();
    // Add 1 month: same day next month (clamped to valid)
    if today.month() == 12 {
        NaiveDate::from_ymd_opt(today.year() + 1, 1, today.day().min(28))
    } else {
        let next_month = today.month() + 1;
        let max_day = days_in_month(today.year(), next_month);
        NaiveDate::from_ymd_opt(today.year(), next_month, today.day().min(max_day))
    }
    .unwrap_or(today)
}

fn days_in_month(year: i32, month: u32) -> u32 {
    if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    }
    .unwrap()
    .pred_opt()
    .unwrap()
    .day()
}

/// Check if a date is selectable (not past, not beyond 1 month, 7AM KST rule).
fn is_selectable(date: NaiveDate) -> bool {
    let today = today_kst();
    if date < today {
        return false;
    }
    let max_date = max_selectable_date();
    if date > max_date {
        return false;
    }
    // Dates exactly at the max boundary require 7AM KST
    if date == max_date {
        return kst_hour() >= 7;
    }
    true
}

use chrono::Timelike;

/// A combined date & time picker modal with calendar grid and time slider.
#[component]
pub fn DatePicker(
    /// Whether the modal is open.
    open: ReadSignal<bool>,
    /// Callback when date+time is confirmed: (date, time_slot 0-47).
    on_select: Callback<(NaiveDate, u32)>,
    /// Callback to close/cancel the modal.
    on_close: Callback<()>,
    /// Currently selected date.
    selected: NaiveDate,
    /// Currently selected time slot (0-47).
    #[prop(default = 16)]
    selected_time_slot: u32,
) -> impl IntoView {
    let initial_date = selected;

    // Temp state for edit-cancel-apply pattern
    let (temp_date, set_temp_date) = signal(initial_date);
    let (temp_time, set_temp_time) = signal(selected_time_slot);
    let (cal_year, set_cal_year) = signal(initial_date.year());
    let (cal_month, set_cal_month) = signal(initial_date.month());

    // Reset temp state when modal opens
    Effect::new(move || {
        if open.get() {
            set_temp_date.set(selected);
            set_temp_time.set(selected_time_slot);
            set_cal_year.set(selected.year());
            set_cal_month.set(selected.month());
        }
    });

    let prev_month_disabled = move || {
        let now = today_kst();
        cal_year.get() < now.year()
            || (cal_year.get() == now.year() && cal_month.get() <= now.month())
    };

    let next_month_disabled = move || {
        let max = max_selectable_date();
        let cal = NaiveDate::from_ymd_opt(cal_year.get(), cal_month.get(), 1).unwrap();
        let max_first = NaiveDate::from_ymd_opt(max.year(), max.month(), 1).unwrap();
        cal >= max_first
    };

    let prev_month = move |_| {
        if !prev_month_disabled() {
            if cal_month.get() == 1 {
                set_cal_month.set(12);
                set_cal_year.set(cal_year.get() - 1);
            } else {
                set_cal_month.set(cal_month.get() - 1);
            }
        }
    };

    let next_month = move |_| {
        if !next_month_disabled() {
            if cal_month.get() == 12 {
                set_cal_month.set(1);
                set_cal_year.set(cal_year.get() + 1);
            } else {
                set_cal_month.set(cal_month.get() + 1);
            }
        }
    };

    let calendar_days = move || {
        let y = cal_year.get();
        let m = cal_month.get();
        let first = NaiveDate::from_ymd_opt(y, m, 1).unwrap();
        let start_weekday = first.weekday().num_days_from_sunday();
        let dim = days_in_month(y, m);

        let mut cells: Vec<Option<NaiveDate>> = Vec::new();
        for _ in 0..start_weekday {
            cells.push(None);
        }
        for d in 1..=dim {
            cells.push(NaiveDate::from_ymd_opt(y, m, d));
        }
        cells
    };

    let month_names = [
        "",
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];

    let on_apply = move |_| {
        on_select.run((temp_date.get(), temp_time.get()));
    };

    let on_cancel = move |_| {
        on_close.run(());
    };

    view! {
        <Show when=move || open.get()>
            <div class="fixed inset-0 z-[160] flex items-center justify-center p-4 page-enter"
                 on:click=on_cancel>
                <div class="absolute inset-0 bg-black/40 backdrop-blur-sm"></div>
                <div class="glass-panel rounded-3xl w-full max-w-sm relative z-10 modal-enter flex flex-col overflow-hidden"
                     on:click=move |e| e.stop_propagation()>
                    // Header
                    <div class="p-4 border-b border-[var(--color-border-default)] flex items-center justify-between">
                        <h3 class="font-semibold text-[var(--color-text-primary)]">{t("calendar.title")}</h3>
                        <button
                            class="p-1.5 rounded-lg hover:bg-[var(--color-interactive-hover)] text-[var(--color-text-tertiary)]"
                            on:click=on_cancel
                        >
                            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
                            </svg>
                        </button>
                    </div>

                    // Calendar body
                    <div class="p-4">
                        // Month navigator
                        <div class="flex items-center justify-between mb-4">
                            <button
                                class="p-2 rounded-lg hover:bg-[var(--color-interactive-hover)] text-[var(--color-text-tertiary)] disabled:opacity-30 transition-colors"
                                on:click=prev_month
                                disabled=prev_month_disabled
                            >
                                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                                    <path stroke-linecap="round" stroke-linejoin="round" d="M15 19l-7-7 7-7" />
                                </svg>
                            </button>
                            <span class="font-bold text-[var(--color-text-primary)]">
                                {move || format!("{} {}", month_names[cal_month.get() as usize], cal_year.get())}
                            </span>
                            <button
                                class="p-2 rounded-lg hover:bg-[var(--color-interactive-hover)] text-[var(--color-text-tertiary)] disabled:opacity-30 transition-colors"
                                on:click=next_month
                                disabled=next_month_disabled
                            >
                                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                                    <path stroke-linecap="round" stroke-linejoin="round" d="M9 5l7 7-7 7" />
                                </svg>
                            </button>
                        </div>

                        // Day headers
                        <div class="grid grid-cols-7 gap-1 mb-1">
                            {["S","M","T","W","T","F","S"].iter().map(|d| view! {
                                <div class="text-center text-xs text-[var(--color-text-disabled)] font-medium py-1">{*d}</div>
                            }).collect::<Vec<_>>()}
                        </div>

                        // Calendar grid
                        <div class="grid grid-cols-7 gap-1 mb-4">
                            {move || calendar_days().into_iter().map(|cell| {
                                match cell {
                                    None => view! { <div class="h-11"></div> }.into_any(),
                                    Some(date) => {
                                        let selectable = is_selectable(date);
                                        let cls = move || {
                                            let sel = temp_date.get() == date;
                                            let is_today = date == today_kst();
                                            if sel {
                                                "h-11 w-full rounded-full flex items-center justify-center text-sm font-medium bg-[var(--color-brand-primary)] text-white shadow-md cursor-pointer"
                                            } else if is_today && selectable {
                                                "h-11 w-full rounded-full flex items-center justify-center text-sm font-medium ring-1 ring-[var(--color-brand-primary)] text-[var(--color-brand-text)] cursor-pointer hover:bg-[var(--color-interactive-hover)]"
                                            } else if !selectable {
                                                "h-11 w-full rounded-full flex items-center justify-center text-sm text-[var(--color-text-disabled)] cursor-not-allowed"
                                            } else {
                                                "h-11 w-full rounded-full flex items-center justify-center text-sm text-[var(--color-text-primary)] cursor-pointer hover:bg-[var(--color-interactive-hover)]"
                                            }
                                        };
                                        view! {
                                            <button
                                                class=cls
                                                disabled=!selectable
                                                on:click=move |_| {
                                                    if selectable {
                                                        set_temp_date.set(date);
                                                    }
                                                }
                                            >
                                                {date.day()}
                                            </button>
                                        }.into_any()
                                    }
                                }
                            }).collect::<Vec<_>>()}
                        </div>

                        // Divider
                        <div class="w-full h-px bg-[var(--color-border-default)] mb-4"></div>

                        // Time slider
                        <TimeSlider
                            value=temp_time
                            on_change=Callback::new(move |v| set_temp_time.set(v))
                            label=t("search.time").to_string()
                        />
                    </div>

                    // Footer — Cancel / Apply
                    <div class="p-4 border-t border-[var(--color-border-default)] grid grid-cols-2 gap-3">
                        <button
                            class="py-3 min-h-11 rounded-xl text-sm font-medium bg-[var(--color-bg-sunken)] text-[var(--color-text-tertiary)] hover:bg-[var(--color-interactive-hover)] transition-colors"
                            on:click=on_cancel
                        >
                            {t("common.cancel")}
                        </button>
                        <button
                            class="py-3 min-h-11 rounded-xl text-sm font-medium btn-glass transition-all"
                            on:click=on_apply
                        >
                            {t("calendar.apply")}
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }.into_any()
}
