//! Search panel — station selection, calendar/time modal, passenger modal,
//! toggle chips, train results, selection prompt, and review modal.

use std::str::FromStr;

use chrono::NaiveDate;
use leptos::prelude::*;

use crate::api::cards::list_cards;
use crate::api::search::{TrainInfo, list_stations, search_trains};
use crate::api::tasks::{
    CreateTaskInput, PassengerCount as TaskPassengerCount, PassengerKind, PassengerList, Provider,
    SeatPreference, TargetTrain, TargetTrainList, create_task,
};
use crate::components::bottom_sheet::BottomSheet;
use crate::components::date_picker::DatePicker;
use crate::components::glass_panel::GlassPanel;
use crate::components::passenger_selector::{PassengerCount, PassengerSelector};
use crate::components::review_modal::ReviewModal;
use crate::components::selection_prompt::SelectionPrompt;
use crate::components::station_input::StationInput;
use crate::components::sortable_list::SortableItem;
use crate::i18n::t;
use crate::utils::{format_time, format_time_slot, slot_to_time_string};

// ── Types ────────────────────────────────────────────────────────────

/// Minimal train info for tracking selection state.
#[derive(Clone, Debug, PartialEq)]
struct SelectedTrain {
    train_number: String,
    dep_time: String,
}

/// Build default passenger counts (7 types).
/// `infant` and `merit` are greyed out — not yet supported end-to-end for KTX.
fn default_passengers() -> Vec<PassengerCount> {
    vec![
        PassengerCount {
            ptype: "adult".into(),
            label: t("passenger.adult").into(),
            description: t("passenger.adult_desc").into(),
            count: 1,
            min: 0,
            max: 9,
            disabled: false,
        },
        PassengerCount {
            ptype: "child".into(),
            label: t("passenger.child").into(),
            description: t("passenger.child_desc").into(),
            count: 0,
            min: 0,
            max: 9,
            disabled: false,
        },
        PassengerCount {
            ptype: "senior".into(),
            label: t("passenger.senior").into(),
            description: t("passenger.senior_desc").into(),
            count: 0,
            min: 0,
            max: 9,
            disabled: false,
        },
        PassengerCount {
            ptype: "severe".into(),
            label: t("passenger.severe").into(),
            description: t("passenger.severe_desc").into(),
            count: 0,
            min: 0,
            max: 9,
            disabled: false,
        },
        PassengerCount {
            ptype: "mild".into(),
            label: t("passenger.mild").into(),
            description: t("passenger.mild_desc").into(),
            count: 0,
            min: 0,
            max: 9,
            disabled: false,
        },
        // Not yet supported end-to-end for KTX — greyed out until runner parity is complete.
        PassengerCount {
            ptype: "infant".into(),
            label: t("passenger.infant").into(),
            description: t("passenger.infant_desc").into(),
            count: 0,
            min: 0,
            max: 9,
            disabled: true,
        },
    ]
}

fn total_passengers(passengers: &[PassengerCount]) -> u8 {
    passengers.iter().map(|p| p.count).sum()
}

fn format_passenger_summary(passengers: &[PassengerCount]) -> String {
    let active: Vec<_> = passengers.iter().filter(|p| p.count > 0).collect();
    if active.is_empty() {
        return "0".into();
    }
    let total: u8 = active.iter().map(|p| p.count).sum();
    if active.len() == 1 {
        format!("{} {}", active[0].count, active[0].label)
    } else {
        format!("{total} {}", t("search.passengers"))
    }
}

fn parse_provider(value: &str) -> Provider {
    Provider::from_str(value).unwrap_or(Provider::Srt)
}

fn parse_seat_preference(value: &str) -> SeatPreference {
    SeatPreference::from_str(value).unwrap_or(SeatPreference::GeneralFirst)
}

fn parse_passenger_kind(value: &str) -> Option<PassengerKind> {
    match value {
        "adult" => Some(PassengerKind::Adult),
        "child" => Some(PassengerKind::Child),
        "senior" => Some(PassengerKind::Senior),
        "severe" => Some(PassengerKind::Severe),
        "mild" => Some(PassengerKind::Mild),
        "infant" => Some(PassengerKind::Infant),
        "merit" => Some(PassengerKind::Merit),
        _ => None,
    }
}

// ── Main component ───────────────────────────────────────────────────

/// Train search form with station selection, calendar/time modal, passenger
/// modal, toggle chips, train results, selection prompt, and review modal.
#[component]
pub fn SearchPanel() -> impl IntoView {
    // ── Form state ─────────────────────────────────────────────────
    let (provider, set_provider) = signal("SRT".to_string());
    let (departure, set_departure) = signal(String::new());
    let (arrival, set_arrival) = signal(String::new());
    let (selected_date, set_selected_date) = signal::<Option<NaiveDate>>(None);
    let (time_slot, set_time_slot) = signal(16u32); // default 08:00
    let (passengers, set_passengers) = signal(default_passengers());
    let (temp_passengers, set_temp_passengers) = signal(default_passengers());

    // ── Toggle chips ───────────────────────────────────────────────
    let (auto_pay, set_auto_pay) = signal(false);
    let (notify, set_notify) = signal(true);
    let (auto_retry, set_auto_retry) = signal(false);

    // ── Modal state ────────────────────────────────────────────────
    let (expanded, set_expanded) = signal(true);
    let (date_modal_open, set_date_modal_open) = signal(false);
    let (passenger_modal_open, set_passenger_modal_open) = signal(false);
    let (review_modal_open, set_review_modal_open) = signal(false);

    // ── Task creation state ────────────────────────────────────────
    let (selected_trains, set_selected_trains) = signal(Vec::<SelectedTrain>::new());
    let (seat_preference, set_seat_preference) = signal("GeneralFirst".to_string());
    let (selected_card_id, set_selected_card_id) = signal(String::new());
    let (review_items, set_review_items) = signal(Vec::<SortableItem>::new());

    // ── Resources ──────────────────────────────────────────────────
    let stations = Resource::new(move || provider.get(), list_stations);
    let cards = Resource::new(|| (), |_| list_cards());

    // ── Search action ──────────────────────────────────────────────
    let search_action = Action::new(move |_: &()| {
        let prov = provider.get_untracked();
        let dep = departure.get_untracked();
        let arr = arrival.get_untracked();
        let d = selected_date
            .get_untracked()
            .map(|d| d.format("%Y%m%d").to_string());
        let t = {
            let slot = time_slot.get_untracked();
            if selected_date.get_untracked().is_some() {
                Some(slot_to_time_string(slot))
            } else {
                None
            }
        };
        async move { search_trains(prov, dep, arr, d, t).await }
    });

    // Clear selection when new search results arrive
    Effect::new(move || {
        if search_action.value().get().is_some() {
            set_selected_trains.set(Vec::new());
            set_expanded.set(false); // collapse form after search
        }
    });

    // ── Task creation action ───────────────────────────────────────
    let create_action = Action::new(move |_: &()| {
        let prov = parse_provider(&provider.get_untracked());
        let dep = departure.get_untracked();
        let arr = arrival.get_untracked();
        let d = selected_date
            .get_untracked()
            .map(|d| d.format("%Y%m%d").to_string())
            .unwrap_or_default();
        let items = review_items.get_untracked();
        let seat_pref = parse_seat_preference(&seat_preference.get_untracked());
        let pay = auto_pay.get_untracked();
        let card = selected_card_id.get_untracked();
        let pax = passengers.get_untracked();
        let notify_on = notify.get_untracked();
        let auto_retry_val = auto_retry.get_untracked();

        let target_trains = TargetTrainList(
            items
                .iter()
                .map(|item| {
                    let parts: Vec<&str> = item.id.splitn(2, ':').collect();
                    TargetTrain {
                        train_number: parts.first().unwrap_or(&"").to_string(),
                        dep_time: parts.get(1).unwrap_or(&"").to_string(),
                    }
                })
                .collect(),
        );

        let passengers = PassengerList(
            pax.iter()
                .filter(|p| p.count > 0)
                .filter_map(|p| {
                    parse_passenger_kind(&p.ptype)
                        .map(|kind| TaskPassengerCount::new(kind, p.count))
                })
                .collect(),
        );

        let dep_time = items
            .first()
            .map(|item| {
                let parts: Vec<&str> = item.id.splitn(2, ':').collect();
                parts.get(1).unwrap_or(&"").to_string()
            })
            .unwrap_or_default();

        async move {
            create_task(CreateTaskInput {
                provider: prov,
                departure_station: dep,
                arrival_station: arr,
                travel_date: d,
                departure_time: dep_time,
                passengers,
                seat_preference: seat_pref,
                target_trains,
                auto_pay: pay,
                payment_card_id: if card.is_empty() {
                    None
                } else {
                    uuid::Uuid::parse_str(&card).ok()
                },
                notify_enabled: notify_on,
                auto_retry: auto_retry_val,
            })
            .await
        }
    });

    let results = search_action.value();
    let searching = search_action.pending();
    let creating = create_action.pending();

    // ── Derived signals ────────────────────────────────────────────
    let selection_count = Memo::new(move |_| selected_trains.get().len());

    // Sync review items when opening review modal
    let open_review = move || {
        let trains = selected_trains.get();
        set_review_items.set(
            trains
                .iter()
                .map(|t| SortableItem {
                    id: format!("{}:{}", t.train_number, t.dep_time),
                    label: format!("{} ({})", t.train_number, format_time(&t.dep_time)),
                })
                .collect(),
        );
        set_review_modal_open.set(true);
    };

    // Open passenger modal with temp state sync
    let open_passenger_modal = move || {
        set_temp_passengers.set(passengers.get());
        set_passenger_modal_open.set(true);
    };

    let confirm_passengers = move || {
        set_passengers.set(temp_passengers.get());
        set_passenger_modal_open.set(false);
    };

    let cancel_passengers = move || {
        set_temp_passengers.set(passengers.get());
        set_passenger_modal_open.set(false);
    };

    view! {
        <div class="px-4 pt-6 pb-4 space-y-4 max-w-xl lg:max-w-2xl mx-auto page-enter">
            // ── Header ─────────────────────────────────────────────
            <div class="flex items-center justify-between">
                <h1 class="text-xl font-bold text-[var(--color-text-primary)]">{t("search.title")}</h1>
                <a href="/settings" class="p-2 rounded-lg hover:bg-[var(--color-interactive-hover)] transition-colors">
                    <svg class="w-5 h-5 text-[var(--color-text-secondary)]" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                        <path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                    </svg>
                </a>
            </div>

            // ── Collapsed summary bar ──────────────────────────────
            <Show when=move || !expanded.get()>
                <CollapsedBar
                    departure=departure
                    arrival=arrival
                    selected_date=selected_date
                    time_slot=time_slot
                    passengers=passengers
                    on_edit=move || set_expanded.set(true)
                />
            </Show>

            // ── Expanded search form ───────────────────────────────
            <Show when=move || expanded.get()>
                <GlassPanel>
                    <div class="p-4 space-y-4">
                        // Provider toggle
                        <ProviderToggle provider=provider set_provider=set_provider />

                        // Station inputs
                        <div class="space-y-3">
                            <StationInput
                                label=t("search.from")
                                id="station-from"
                                value=departure
                                set_value=set_departure
                                stations=stations
                            />
                            <div class="flex justify-center">
                                <button
                                    aria-label=t("search.swap_stations")
                                    class="p-2 min-h-11 min-w-11 flex items-center justify-center rounded-full bg-[var(--color-bg-sunken)] hover:bg-[var(--color-interactive-hover)] transition-colors"
                                    on:click=move |_| {
                                        let d = departure.get_untracked();
                                        let a = arrival.get_untracked();
                                        set_departure.set(a);
                                        set_arrival.set(d);
                                    }
                                >
                                    <svg aria-hidden="true" class="w-4 h-4 text-[var(--color-text-secondary)]" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                                        <path stroke-linecap="round" stroke-linejoin="round" d="M7 16V4m0 0L3 8m4-4l4 4m6 0v12m0 0l4-4m-4 4l-4-4" />
                                    </svg>
                                </button>
                            </div>
                            <StationInput
                                label=t("search.to")
                                id="station-to"
                                value=arrival
                                set_value=set_arrival
                                stations=stations
                            />
                        </div>

                        // Date & Passengers cards
                        <div class="grid grid-cols-1 min-[360px]:grid-cols-2 gap-3">
                            // Date card — opens calendar modal
                            <button
                                class="p-3 bg-[var(--color-bg-sunken)] border border-[var(--color-border-default)] rounded-xl text-left hover:border-[var(--color-border-focus)] transition-colors"
                                on:click=move |_| set_date_modal_open.set(true)
                            >
                                <div class="text-xs font-medium text-[var(--color-text-secondary)] mb-1 flex items-center gap-1.5">
                                    <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                                        <path stroke-linecap="round" stroke-linejoin="round" d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
                                    </svg>
                                    {t("search.date")}
                                </div>
                                <div class="text-sm font-medium text-[var(--color-text-primary)]">
                                    {move || match selected_date.get() {
                                        Some(d) => format!("{} {}", d.format("%b %d"), format_time_slot(time_slot.get())),
                                        None => t("search.date").to_string(),
                                    }}
                                </div>
                            </button>

                            // Passenger card — opens passenger modal
                            <button
                                class="p-3 bg-[var(--color-bg-sunken)] border border-[var(--color-border-default)] rounded-xl text-left hover:border-[var(--color-border-focus)] transition-colors"
                                on:click=move |_| open_passenger_modal()
                            >
                                <div class="text-xs font-medium text-[var(--color-text-secondary)] mb-1 flex items-center gap-1.5">
                                    <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                                        <path stroke-linecap="round" stroke-linejoin="round" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0z" />
                                    </svg>
                                    {t("search.passengers")}
                                </div>
                                <div class="text-sm font-medium text-[var(--color-text-primary)]">
                                    {move || format_passenger_summary(&passengers.get())}
                                </div>
                            </button>
                        </div>

                        // Toggle chips
                        <div class="flex gap-2">
                            <ToggleChip
                                label=t("search.auto_pay")
                                active=auto_pay
                                on_toggle=move || set_auto_pay.update(|v| *v = !*v)
                            />
                            <ToggleChip
                                label=t("search.notify")
                                active=notify
                                on_toggle=move || set_notify.update(|v| *v = !*v)
                            />
                            <ToggleChip
                                label=t("search.auto_retry")
                                active=auto_retry
                                on_toggle=move || set_auto_retry.update(|v| *v = !*v)
                            />
                        </div>

                        // Search button
                        <button
                            class="w-full py-3 btn-glass font-medium rounded-xl text-sm disabled:opacity-50 transition-all"
                            disabled=searching
                            on:click=move |_| { search_action.dispatch(()); }
                        >
                            {move || if searching.get() { t("search.searching") } else { t("search.search_btn") }}
                        </button>
                    </div>
                </GlassPanel>
            </Show>

            // ── Train results ──────────────────────────────────────
            {move || results.get().map(|result| match result {
                Ok(trains) if trains.is_empty() => view! {
                    <GlassPanel>
                        <div class="p-4 text-center py-8">
                            <p class="text-[var(--color-text-tertiary)] text-sm">{t("search.no_results")}</p>
                        </div>
                    </GlassPanel>
                }.into_any(),
                Ok(trains) => view! {
                    <TrainResults
                        trains=trains
                        provider=provider.get_untracked()
                        selected=selected_trains
                        set_selected=set_selected_trains
                    />
                }.into_any(),
                Err(e) => view! {
                    <GlassPanel>
                        <div class="p-4 text-center py-8">
                            <p class="text-[var(--color-status-error)] text-sm">{format!("{e}")}</p>
                        </div>
                    </GlassPanel>
                }.into_any(),
            })}

            // ── Task creation success ──────────────────────────────
            {move || create_action.value().get().and_then(|r| r.ok()).map(|_| view! {
                <GlassPanel>
                    <div class="p-4">
                        <div class="px-3 py-2 bg-[var(--color-status-success)]/10 border border-[var(--color-status-success)]/30 rounded-xl">
                            <p class="text-sm text-[var(--color-status-success)]">{t("task.created")}</p>
                            <a href="/tasks" class="text-xs text-[var(--color-brand-text)] hover:underline mt-1 inline-block">
                                {t("search.view_tasks")}
                            </a>
                        </div>
                    </div>
                </GlassPanel>
            })}
        </div>

        // ── Floating selection prompt ──────────────────────────────
        <SelectionPrompt
            count=Signal::from(selection_count)
            on_confirm=Callback::new(move |_| open_review())
            on_clear=Callback::new(move |_| set_selected_trains.set(Vec::new()))
        />

        // ── Date & Time modal ──────────────────────────────────────
        <DatePicker
            open=date_modal_open
            selected=selected_date.get_untracked().unwrap_or_else(|| {
                let kst = chrono::Utc::now() + chrono::Duration::hours(9);
                kst.date_naive()
            })
            selected_time_slot=time_slot.get_untracked()
            on_select=Callback::new(move |(date, slot)| {
                set_selected_date.set(Some(date));
                set_time_slot.set(slot);
                set_date_modal_open.set(false);
            })
            on_close=Callback::new(move |_| set_date_modal_open.set(false))
        />

        // ── Passenger modal ────────────────────────────────────────
        <BottomSheet
            open=passenger_modal_open
            on_close=Callback::new(move |_| cancel_passengers())
            title=t("passenger.title").to_string()
        >
            <div class="space-y-4">
                // Total badge
                <div class="flex justify-end">
                    <span class="text-xs font-medium bg-[var(--color-brand-primary)]/20 text-[var(--color-brand-text)] px-2 py-1 rounded-full">
                        {move || format!("{}: {} / 9", t("passenger.total"), total_passengers(&temp_passengers.get()))}
                    </span>
                </div>
                <PassengerSelector
                    passengers=temp_passengers
                    on_change=Callback::new(move |v| set_temp_passengers.set(v))
                />
                // Footer buttons
                <div class="flex gap-3 pt-2">
                    <button
                        class="flex-1 py-2.5 rounded-xl text-sm font-medium bg-[var(--color-bg-sunken)] text-[var(--color-text-tertiary)] hover:bg-[var(--color-interactive-hover)] transition-colors"
                        on:click=move |_| cancel_passengers()
                    >{t("common.cancel")}</button>
                    <button
                        class="flex-1 py-2.5 rounded-xl text-sm font-medium btn-glass transition-all"
                        on:click=move |_| confirm_passengers()
                    >{t("common.confirm")}</button>
                </div>
            </div>
        </BottomSheet>

        // ── Review modal ───────────────────────────────────────────
        <ReviewModal
            open=review_modal_open
            items=review_items
            on_items_change=Callback::new(move |v| set_review_items.set(v))
            seat_preference=seat_preference
            set_seat_preference=set_seat_preference
            auto_pay=auto_pay
            selected_card_id=selected_card_id
            set_selected_card_id=set_selected_card_id
            cards=cards
            creating=Signal::from(creating)
            on_confirm=Callback::new(move |_| { create_action.dispatch(()); })
            on_cancel=Callback::new(move |_| set_review_modal_open.set(false))
        />
    }
}

// ── Sub-components ───────────────────────────────────────────────────

/// Collapsed summary bar — click to expand.
#[component]
fn CollapsedBar(
    departure: ReadSignal<String>,
    arrival: ReadSignal<String>,
    selected_date: ReadSignal<Option<NaiveDate>>,
    time_slot: ReadSignal<u32>,
    passengers: ReadSignal<Vec<PassengerCount>>,
    on_edit: impl Fn() + Send + Sync + 'static,
) -> impl IntoView {
    view! {
        <div class="cursor-pointer" on:click=move |_| on_edit()>
            <GlassPanel>
                <div class="p-3 flex items-center justify-between">
                    <div class="flex items-center gap-2 overflow-hidden">
                        <span class="font-semibold text-sm text-[var(--color-text-primary)] truncate">
                            {move || {
                                let d = departure.get();
                                let a = arrival.get();
                                if d.is_empty() && a.is_empty() {
                                    t("search.title").to_string()
                                } else {
                                    format!("{d} \u{2192} {a}")
                                }
                            }}
                        </span>
                        <div class="h-4 w-px bg-[var(--color-border-default)] shrink-0"></div>
                        <span class="text-sm text-[var(--color-text-tertiary)] shrink-0">
                            {move || match selected_date.get() {
                                Some(d) => format!("{} {}", d.format("%b %d"), format_time_slot(time_slot.get())),
                                None => "\u{2014}".to_string(),
                            }}
                        </span>
                        <div class="h-4 w-px bg-[var(--color-border-default)] shrink-0"></div>
                        <span class="text-sm text-[var(--color-text-tertiary)] shrink-0">
                            {move || format!("{} {}", total_passengers(&passengers.get()), t("search.passenger_count"))}
                        </span>
                    </div>
                    <span class="text-[var(--color-brand-text)] text-sm font-semibold shrink-0 pl-2">
                        {t("search.edit")}
                    </span>
                </div>
            </GlassPanel>
        </div>
    }
}

/// Provider toggle (SRT / KTX).
#[component]
fn ProviderToggle(
    provider: ReadSignal<String>,
    set_provider: WriteSignal<String>,
) -> impl IntoView {
    let btn_class = |is_active: bool| {
        if is_active {
            "flex-1 py-2 text-sm font-medium rounded-lg bg-[var(--color-bg-elevated)] text-[var(--color-text-primary)] shadow-sm transition-all"
        } else {
            "flex-1 py-2 text-sm font-medium rounded-lg text-[var(--color-text-tertiary)] transition-all"
        }
    };
    view! {
        <div role="radiogroup" aria-label=t("search.provider") class="flex bg-[var(--color-bg-sunken)] rounded-xl p-1">
            <button
                role="radio"
                aria-checked=move || if provider.get() == "SRT" { "true" } else { "false" }
                class=move || btn_class(provider.get() == "SRT")
                on:click=move |_| set_provider.set("SRT".to_string())
            >"SRT"</button>
            <button
                role="radio"
                aria-checked=move || if provider.get() == "KTX" { "true" } else { "false" }
                class=move || btn_class(provider.get() == "KTX")
                on:click=move |_| set_provider.set("KTX".to_string())
            >"KTX"</button>
        </div>
    }
}

/// Toggle chip button.
#[component]
fn ToggleChip(
    label: &'static str,
    active: ReadSignal<bool>,
    on_toggle: impl Fn() + Send + Sync + 'static,
) -> impl IntoView {
    view! {
        <button
            aria-pressed=move || if active.get() { "true" } else { "false" }
            class=move || if active.get() {
                "flex-1 py-2 px-3 text-xs font-semibold rounded-xl border transition-all flex justify-center items-center gap-1.5 bg-[var(--color-brand-primary)]/20 text-[var(--color-brand-text)] border-[var(--color-brand-primary)]/30 shadow-sm"
            } else {
                "flex-1 py-2 px-3 text-xs font-semibold rounded-xl border transition-all flex justify-center items-center gap-1.5 bg-[var(--color-bg-sunken)] text-[var(--color-text-tertiary)] border-[var(--color-border-default)]"
            }
            on:click=move |_| on_toggle()
        >
            {label}
        </button>
    }
}

/// Train search results with selectable rows.
#[component]
fn TrainResults(
    trains: Vec<TrainInfo>,
    provider: String,
    selected: ReadSignal<Vec<SelectedTrain>>,
    set_selected: WriteSignal<Vec<SelectedTrain>>,
) -> impl IntoView {
    view! {
        <GlassPanel>
            <div class="p-4">
                <div class="flex items-center justify-between mb-3">
                    <h2 class="text-sm font-semibold text-[var(--color-text-secondary)] uppercase tracking-[0.18em]">
                        {format!("{} ({trains_len})", provider, trains_len = trains.len())}
                    </h2>
                    <p class="text-[10px] text-[var(--color-text-tertiary)]">{t("search.tap_to_select")}</p>
                </div>
                <div class="space-y-1">
                    {trains.into_iter().map(|train| {
                        let num = train.train_number.clone();
                        let dep = train.dep_time.clone();
                        let sel_train = SelectedTrain {
                            train_number: num.clone(),
                            dep_time: dep.clone(),
                        };
                        let is_selected = {
                            let n = num.clone();
                            Memo::new(move |_| selected.get().iter().any(|s| s.train_number == n))
                        };
                        let toggle = {
                            let st = sel_train.clone();
                            move |_| {
                                set_selected.update(|list| {
                                    if let Some(idx) = list.iter().position(|s| s.train_number == st.train_number) {
                                        list.remove(idx);
                                    } else {
                                        list.push(st.clone());
                                    }
                                });
                            }
                        };
                        view! {
                            <button
                                class=move || if is_selected.get() {
                                    "w-full flex items-center justify-between py-3 px-3 rounded-xl bg-[var(--color-brand-primary)]/10 border border-[var(--color-brand-primary)]/30 transition-all"
                                } else {
                                    "w-full flex items-center justify-between py-3 px-3 rounded-xl border border-transparent hover:bg-[var(--color-interactive-hover)] transition-all"
                                }
                                on:click=toggle
                            >
                                <div class="flex items-center gap-3">
                                    <div class=move || if is_selected.get() {
                                        "w-5 h-5 rounded-full bg-[var(--color-brand-primary)] flex items-center justify-center shrink-0"
                                    } else {
                                        "w-5 h-5 rounded-full border-2 border-[var(--color-border-default)] shrink-0"
                                    }>
                                        {move || is_selected.get().then(|| view! {
                                            <svg class="w-3 h-3 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="3">
                                                <path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
                                            </svg>
                                        })}
                                    </div>
                                    <div class="text-left">
                                        <div class="flex items-center gap-2">
                                            <span class="text-xs px-1.5 py-0.5 rounded bg-[var(--color-bg-sunken)] text-[var(--color-text-secondary)] font-mono">
                                                {train.train_number.clone()}
                                            </span>
                                            <span class="text-xs text-[var(--color-text-tertiary)]">{train.train_type_name.clone()}</span>
                                        </div>
                                        <div class="flex items-center gap-2 mt-1">
                                            <span class="text-sm font-medium text-[var(--color-text-primary)]">
                                                {format_time(&train.dep_time)}
                                            </span>
                                            <span class="text-xs text-[var(--color-text-tertiary)]">"\u{2192}"</span>
                                            <span class="text-sm font-medium text-[var(--color-text-primary)]">
                                                {format_time(&train.arr_time)}
                                            </span>
                                        </div>
                                    </div>
                                </div>
                                <div class="flex gap-1.5">
                                    <SeatBadge label=t("seat.general") available=train.general_available />
                                    <SeatBadge label=t("seat.special") available=train.special_available />
                                </div>
                            </button>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </div>
        </GlassPanel>
    }
}

/// Seat availability badge.
#[component]
fn SeatBadge(label: &'static str, available: bool) -> impl IntoView {
    let class = if available {
        "text-[10px] px-1.5 py-0.5 rounded bg-[var(--color-status-success)]/20 text-[var(--color-status-success)]"
    } else {
        "text-[10px] px-1.5 py-0.5 rounded bg-[var(--color-bg-sunken)] text-[var(--color-text-disabled)]"
    };
    view! { <span class=class>{label}</span> }
}
