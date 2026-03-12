//! Search panel — departure, arrival, date, passengers, search button, train
//! selection, and task creation.

use leptos::prelude::*;

use crate::api::cards::list_cards;
use crate::api::search::{list_stations, search_trains, StationInfo, TrainInfo};
use crate::api::tasks::create_task;
use crate::components::glass_panel::GlassPanel;
use crate::i18n::t;

/// Train search form with station selection, date picker, passenger count,
/// train selection, and task creation.
#[component]
pub fn SearchPanel() -> impl IntoView {
    let (provider, set_provider) = signal("SRT".to_string());
    let (departure, set_departure) = signal(String::new());
    let (arrival, set_arrival) = signal(String::new());
    let (date, set_date) = signal(String::new());
    let (time, set_time) = signal(String::new());
    let (adult_count, set_adult_count) = signal(1u8);

    // Selected train numbers for task creation
    let (selected_trains, set_selected_trains) = signal(Vec::<SelectedTrain>::new());

    // Task creation options
    let (seat_preference, set_seat_preference) = signal("GeneralFirst".to_string());
    let (auto_pay, set_auto_pay) = signal(false);
    let (selected_card_id, set_selected_card_id) = signal(String::new());

    // Fetch stations for the selected provider
    let stations = Resource::new(move || provider.get(), |prov| list_stations(prov));

    // Fetch cards for auto-pay selection
    let cards = Resource::new(|| (), |_| list_cards());

    // Search action
    let search_action = Action::new(move |_: &()| {
        let prov = provider.get_untracked();
        let dep = departure.get_untracked();
        let arr = arrival.get_untracked();
        let d = date.get_untracked();
        let t = time.get_untracked();
        async move {
            search_trains(
                prov,
                dep,
                arr,
                if d.is_empty() { None } else { Some(d) },
                if t.is_empty() { None } else { Some(t) },
            )
            .await
        }
    });

    // Clear selection when new search results arrive
    Effect::new(move || {
        if search_action.value().get().is_some() {
            set_selected_trains.set(Vec::new());
        }
    });

    // Task creation action
    let create_action = Action::new(move |_: &()| {
        let prov = provider.get_untracked();
        let dep = departure.get_untracked();
        let arr = arrival.get_untracked();
        let d = date.get_untracked();
        let adults = adult_count.get_untracked();
        let trains = selected_trains.get_untracked();
        let seat_pref = seat_preference.get_untracked();
        let pay = auto_pay.get_untracked();
        let card = selected_card_id.get_untracked();

        // Build target_trains JSON
        let target_trains_json = serde_json::json!(
            trains.iter().map(|t| {
                serde_json::json!({
                    "train_number": t.train_number,
                    "dep_time": t.dep_time,
                })
            }).collect::<Vec<_>>()
        )
        .to_string();

        // Build passengers JSON
        let passengers_json =
            serde_json::json!([{"type": "adult", "count": adults}]).to_string();

        // First selected train's dep_time as the task departure_time
        let dep_time = trains
            .first()
            .map(|t| t.dep_time.clone())
            .unwrap_or_default();

        async move {
            create_task(
                prov,
                dep,
                arr,
                d,
                dep_time,
                passengers_json,
                seat_pref,
                target_trains_json,
                Some(pay),
                if card.is_empty() { None } else { Some(card) },
                Some(true),
            )
            .await
        }
    });

    let results = search_action.value();
    let searching = search_action.pending();
    let creating = create_action.pending();
    let create_result = create_action.value();

    view! {
        <div class="px-4 pt-6 pb-4 space-y-4">
            <div class="flex items-center justify-between">
                <h1 class="text-xl font-bold text-[var(--color-text-primary)]">{t("search.title")}</h1>
                <a href="/settings" class="p-2 rounded-lg hover:bg-[var(--color-interactive-hover)] transition-colors">
                    <svg class="w-5 h-5 text-[var(--color-text-secondary)]" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                        <path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                    </svg>
                </a>
            </div>

            <GlassPanel>
                <div class="p-4 space-y-4">
                    // Provider toggle
                    <div class="flex bg-[var(--color-bg-sunken)] rounded-xl p-1">
                        <button
                            class=move || if provider.get() == "SRT" {
                                "flex-1 py-2 text-sm font-medium rounded-lg bg-[var(--color-bg-elevated)] text-[var(--color-text-primary)] shadow-sm transition-all"
                            } else {
                                "flex-1 py-2 text-sm font-medium rounded-lg text-[var(--color-text-tertiary)] transition-all"
                            }
                            on:click=move |_| set_provider.set("SRT".to_string())
                        >
                            "SRT"
                        </button>
                        <button
                            class=move || if provider.get() == "KTX" {
                                "flex-1 py-2 text-sm font-medium rounded-lg bg-[var(--color-bg-elevated)] text-[var(--color-text-primary)] shadow-sm transition-all"
                            } else {
                                "flex-1 py-2 text-sm font-medium rounded-lg text-[var(--color-text-tertiary)] transition-all"
                            }
                            on:click=move |_| set_provider.set("KTX".to_string())
                        >
                            "KTX"
                        </button>
                    </div>

                    // Station inputs
                    <div class="space-y-3">
                        <StationSelect
                            label=t("search.departure")
                            value=departure
                            set_value=set_departure
                            stations=stations
                        />

                        // Swap button
                        <div class="flex justify-center">
                            <button
                                class="p-2 rounded-full bg-[var(--color-bg-sunken)] hover:bg-[var(--color-interactive-hover)] transition-colors"
                                on:click=move |_| {
                                    let d = departure.get_untracked();
                                    let a = arrival.get_untracked();
                                    set_departure.set(a);
                                    set_arrival.set(d);
                                }
                            >
                                <svg class="w-4 h-4 text-[var(--color-text-secondary)]" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                                    <path stroke-linecap="round" stroke-linejoin="round" d="M7 16V4m0 0L3 8m4-4l4 4m6 0v12m0 0l4-4m-4 4l-4-4" />
                                </svg>
                            </button>
                        </div>

                        <StationSelect
                            label=t("search.arrival")
                            value=arrival
                            set_value=set_arrival
                            stations=stations
                        />
                    </div>

                    // Date & time
                    <div class="grid grid-cols-2 gap-3">
                        <div>
                            <label class="block text-xs font-medium text-[var(--color-text-secondary)] mb-1.5">{t("search.date")}</label>
                            <input
                                type="date"
                                class="w-full px-3 py-2.5 bg-[var(--color-bg-sunken)] border border-[var(--color-border-default)] rounded-xl text-sm text-[var(--color-text-primary)] focus:outline-none focus:border-[var(--color-border-focus)] transition-colors"
                                prop:value=date
                                on:input=move |ev| set_date.set(event_target_value(&ev))
                            />
                        </div>
                        <div>
                            <label class="block text-xs font-medium text-[var(--color-text-secondary)] mb-1.5">{t("search.time")}</label>
                            <input
                                type="time"
                                class="w-full px-3 py-2.5 bg-[var(--color-bg-sunken)] border border-[var(--color-border-default)] rounded-xl text-sm text-[var(--color-text-primary)] focus:outline-none focus:border-[var(--color-border-focus)] transition-colors"
                                prop:value=time
                                on:input=move |ev| set_time.set(event_target_value(&ev))
                            />
                        </div>
                    </div>

                    // Passenger count
                    <div>
                        <label class="block text-xs font-medium text-[var(--color-text-secondary)] mb-1.5">{t("search.adults")}</label>
                        <div class="flex items-center gap-3">
                            <button
                                class="w-8 h-8 flex items-center justify-center rounded-lg bg-[var(--color-bg-sunken)] text-[var(--color-text-secondary)] hover:bg-[var(--color-interactive-hover)] disabled:opacity-40 transition-colors"
                                disabled=move || adult_count.get() <= 1
                                on:click=move |_| set_adult_count.update(|c| *c = c.saturating_sub(1).max(1))
                            >"-"</button>
                            <span class="text-sm font-medium text-[var(--color-text-primary)] w-8 text-center">{adult_count}</span>
                            <button
                                class="w-8 h-8 flex items-center justify-center rounded-lg bg-[var(--color-bg-sunken)] text-[var(--color-text-secondary)] hover:bg-[var(--color-interactive-hover)] disabled:opacity-40 transition-colors"
                                disabled=move || adult_count.get() >= 9
                                on:click=move |_| set_adult_count.update(|c| *c = (*c + 1).min(9))
                            >"+"</button>
                        </div>
                    </div>

                    // Search button
                    <button
                        class="w-full py-3 bg-[var(--color-brand-primary)] text-white font-medium rounded-xl text-sm hover:opacity-90 disabled:opacity-50 transition-all"
                        disabled=searching
                        on:click=move |_| { search_action.dispatch(()); }
                    >
                        {move || if searching.get() { t("search.searching") } else { t("search.search_btn") }}
                    </button>
                </div>
            </GlassPanel>

            // Search results (inline) with selectable trains
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

            // Task creation panel (appears when trains are selected)
            {move || {
                let sel = selected_trains.get();
                if sel.is_empty() {
                    None
                } else {
                    Some(view! {
                        <TaskCreationPanel
                            selected_count=sel.len()
                            seat_preference=seat_preference
                            set_seat_preference=set_seat_preference
                            auto_pay=auto_pay
                            set_auto_pay=set_auto_pay
                            selected_card_id=selected_card_id
                            set_selected_card_id=set_selected_card_id
                            cards=cards
                            creating=Signal::from(creating)
                            create_result=Signal::from(create_result)
                            on_submit=move || { create_action.dispatch(()); }
                        />
                    })
                }
            }}
        </div>
    }
}

// ── Internal types ──────────────────────────────────────────────────

/// Minimal train info for tracking selection state.
#[derive(Clone, Debug, PartialEq)]
struct SelectedTrain {
    train_number: String,
    dep_time: String,
}

// ── Sub-components ──────────────────────────────────────────────────

/// Station select dropdown.
#[component]
fn StationSelect(
    label: &'static str,
    value: ReadSignal<String>,
    set_value: WriteSignal<String>,
    stations: Resource<Result<Vec<StationInfo>, ServerFnError>>,
) -> impl IntoView {
    view! {
        <div>
            <label class="block text-xs font-medium text-[var(--color-text-secondary)] mb-1.5">{label}</label>
            <select
                class="w-full px-3 py-2.5 bg-[var(--color-bg-sunken)] border border-[var(--color-border-default)] rounded-xl text-sm text-[var(--color-text-primary)] focus:outline-none focus:border-[var(--color-border-focus)] transition-colors"
                on:change=move |ev| set_value.set(event_target_value(&ev))
            >
                <option value="" disabled selected=move || value.get().is_empty()>{t("search.select_station")}</option>
                <Suspense>
                    {move || stations.get().map(|result| match result {
                        Ok(list) => list.into_iter().map(|s| {
                            let ko = s.name_ko.clone();
                            let selected = value.get() == ko;
                            let ko_text = ko.clone();
                            view! {
                                <option value=ko selected=selected>{ko_text}</option>
                            }
                        }).collect::<Vec<_>>(),
                        Err(_) => vec![],
                    })}
                </Suspense>
            </select>
        </div>
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
                    <h2 class="text-sm font-semibold text-[var(--color-text-secondary)] uppercase tracking-wider">
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
                                    // Selection indicator
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
                                            <span class="text-xs text-[var(--color-text-tertiary)]">"→"</span>
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

/// Task creation review panel — shown when trains are selected.
#[component]
fn TaskCreationPanel(
    selected_count: usize,
    seat_preference: ReadSignal<String>,
    set_seat_preference: WriteSignal<String>,
    auto_pay: ReadSignal<bool>,
    set_auto_pay: WriteSignal<bool>,
    selected_card_id: ReadSignal<String>,
    set_selected_card_id: WriteSignal<String>,
    cards: Resource<Result<Vec<crate::api::cards::CardInfo>, ServerFnError>>,
    creating: Signal<bool>,
    create_result: Signal<Option<Result<crate::api::tasks::TaskInfo, ServerFnError>>>,
    on_submit: impl Fn() + Send + Sync + 'static,
) -> impl IntoView {
    view! {
        <GlassPanel>
            <div class="p-4 space-y-4">
                <div class="flex items-center justify-between">
                    <h2 class="text-sm font-semibold text-[var(--color-text-secondary)] uppercase tracking-wider">
                        {t("search.create_task")}
                    </h2>
                    <span class="text-xs px-2 py-0.5 rounded-full bg-[var(--color-brand-primary)]/20 text-[var(--color-brand-primary)]">
                        {format!("{selected_count} {}", t("selection.selected_count"))}
                    </span>
                </div>

                // Seat preference
                <div>
                    <label class="block text-xs font-medium text-[var(--color-text-secondary)] mb-1.5">{t("search.seat_preference")}</label>
                    <select
                        class="w-full px-3 py-2.5 bg-[var(--color-bg-sunken)] border border-[var(--color-border-default)] rounded-xl text-sm text-[var(--color-text-primary)] focus:outline-none focus:border-[var(--color-border-focus)] transition-colors"
                        on:change=move |ev| set_seat_preference.set(event_target_value(&ev))
                    >
                        <option value="GeneralFirst" selected=move || seat_preference.get() == "GeneralFirst">{t("search.seat_general_first")}</option>
                        <option value="SpecialFirst" selected=move || seat_preference.get() == "SpecialFirst">{t("search.seat_special_first")}</option>
                        <option value="GeneralOnly" selected=move || seat_preference.get() == "GeneralOnly">{t("search.seat_general_only")}</option>
                        <option value="SpecialOnly" selected=move || seat_preference.get() == "SpecialOnly">{t("search.seat_special_only")}</option>
                    </select>
                </div>

                // Auto-pay toggle
                <div class="flex items-center justify-between">
                    <span class="text-sm text-[var(--color-text-primary)]">{t("search.auto_pay")}</span>
                    <button
                        class=move || if auto_pay.get() {
                            "w-10 h-6 bg-[var(--color-brand-primary)] rounded-full relative cursor-pointer transition-colors"
                        } else {
                            "w-10 h-6 bg-[var(--color-bg-sunken)] rounded-full relative cursor-pointer transition-colors"
                        }
                        on:click=move |_| set_auto_pay.update(|v| *v = !*v)
                    >
                        <div class=move || if auto_pay.get() {
                            "absolute top-0.5 right-0.5 w-5 h-5 bg-white rounded-full shadow transition-all"
                        } else {
                            "absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full shadow transition-all"
                        }></div>
                    </button>
                </div>

                // Card selection (shown only when auto-pay is enabled)
                {move || auto_pay.get().then(|| view! {
                    <div>
                        <label class="block text-xs font-medium text-[var(--color-text-secondary)] mb-1.5">{t("search.select_card")}</label>
                        <Suspense fallback=move || view! {
                            <p class="text-xs text-[var(--color-text-tertiary)]">{t("common.loading")}</p>
                        }>
                            {move || cards.get().map(|result| match result {
                                Ok(card_list) if card_list.is_empty() => view! {
                                    <p class="text-xs text-[var(--color-text-tertiary)]">
                                        {t("search.no_cards")} " "
                                        <a href="/settings" class="text-[var(--color-brand-primary)] hover:underline">{t("search.add_card")}</a>
                                    </p>
                                }.into_any(),
                                Ok(card_list) => view! {
                                    <select
                                        class="w-full px-3 py-2.5 bg-[var(--color-bg-sunken)] border border-[var(--color-border-default)] rounded-xl text-sm text-[var(--color-text-primary)] focus:outline-none focus:border-[var(--color-border-focus)] transition-colors"
                                        on:change=move |ev| set_selected_card_id.set(event_target_value(&ev))
                                    >
                                        <option value="" disabled selected=move || selected_card_id.get().is_empty()>{t("search.select_card")}</option>
                                        {card_list.into_iter().map(|card| {
                                            let id = card.id.to_string();
                                            let label = format!("{} ····{}", card.card_type_name, card.last_four);
                                            view! {
                                                <option value=id>{label}</option>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </select>
                                }.into_any(),
                                Err(_) => view! {
                                    <p class="text-xs text-[var(--color-status-error)]">{t("error.network")}</p>
                                }.into_any(),
                            })}
                        </Suspense>
                    </div>
                })}

                // Error display
                {move || create_result.get().and_then(|r| r.err()).map(|e| view! {
                    <div class="px-3 py-2 bg-[var(--color-status-error)]/10 border border-[var(--color-status-error)]/30 rounded-xl">
                        <p class="text-sm text-[var(--color-status-error)]">{format!("{e}")}</p>
                    </div>
                })}

                // Submit button
                <button
                    class="w-full py-3 bg-[var(--color-brand-primary)] text-white font-medium rounded-xl text-sm hover:opacity-90 disabled:opacity-50 transition-all"
                    disabled=creating
                    on:click=move |_| { on_submit(); }
                >
                    {move || if creating.get() { t("search.creating_task") } else { t("search.create_task") }}
                </button>

                // Success display
                {move || create_result.get().and_then(|r| r.ok()).map(|_task| view! {
                    <div class="px-3 py-2 bg-[var(--color-status-success)]/10 border border-[var(--color-status-success)]/30 rounded-xl">
                        <p class="text-sm text-[var(--color-status-success)]">
                            {t("task.created")}
                        </p>
                        <a href="/tasks" class="text-xs text-[var(--color-brand-primary)] hover:underline mt-1 inline-block">
                            {t("search.view_tasks")}
                        </a>
                    </div>
                })}
            </div>
        </GlassPanel>
    }
}

#[component]
fn SeatBadge(label: &'static str, available: bool) -> impl IntoView {
    let class = if available {
        "text-[10px] px-1.5 py-0.5 rounded bg-[var(--color-status-success)]/20 text-[var(--color-status-success)]"
    } else {
        "text-[10px] px-1.5 py-0.5 rounded bg-[var(--color-bg-sunken)] text-[var(--color-text-disabled)]"
    };
    view! { <span class=class>{label}</span> }
}

fn format_time(raw: &str) -> String {
    if raw.len() >= 4 {
        format!("{}:{}", &raw[..2], &raw[2..4])
    } else {
        raw.to_string()
    }
}
