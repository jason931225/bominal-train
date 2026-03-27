//! Search page with provider toggle, station autocomplete, results, and task creation.

use std::str::FromStr;

use leptos::prelude::*;
use leptos_router::{NavigateOptions, hooks::use_navigate};
use uuid::Uuid;

use crate::{
    api,
    components::StatusChip,
    i18n::t,
    types::{
        CreateTaskInput, PassengerCount, PassengerKind, PassengerList, Provider, SeatPreference,
        TargetTrain, TargetTrainList, TrainInfo,
    },
    utils::format_time,
};

use super::{ProtectedPage, format_server_error};

fn parse_provider(value: &str) -> Provider {
    Provider::from_str(value).unwrap_or(Provider::Srt)
}

fn parse_seat_preference(value: &str) -> SeatPreference {
    SeatPreference::from_str(value).unwrap_or(SeatPreference::GeneralFirst)
}

fn build_passenger_list(adults: u8, children: u8, seniors: u8) -> PassengerList {
    let mut items = Vec::new();

    if adults > 0 {
        items.push(PassengerCount::new(PassengerKind::Adult, adults));
    }
    if children > 0 {
        items.push(PassengerCount::new(PassengerKind::Child, children));
    }
    if seniors > 0 {
        items.push(PassengerCount::new(PassengerKind::Senior, seniors));
    }

    PassengerList(items)
}

fn provider_badge_class(provider: &str, active: bool) -> &'static str {
    match (provider, active) {
        ("SRT", true) => "lg-provider-pill lg-provider-pill--srt-active",
        ("KTX", true) => "lg-provider-pill lg-provider-pill--ktx-active",
        ("SRT", false) => "lg-provider-pill lg-provider-pill--srt",
        _ => "lg-provider-pill lg-provider-pill--ktx",
    }
}

fn train_selection(train: &TrainInfo) -> TargetTrain {
    TargetTrain {
        train_number: train.train_number.clone(),
        dep_time: train.dep_time.clone(),
    }
}

fn is_selected(selected: &[TargetTrain], train: &TrainInfo) -> bool {
    selected
        .iter()
        .any(|item| item.train_number == train.train_number && item.dep_time == train.dep_time)
}

fn normalize_time_input(value: &str) -> Option<String> {
    if value.is_empty() {
        return None;
    }

    let digits: String = value.chars().filter(|ch| ch.is_ascii_digit()).collect();
    match digits.len() {
        4 => Some(format!("{digits}00")),
        6 => Some(digits),
        _ => None,
    }
}

fn seat_chip(label: &'static str, active: bool) -> (&'static str, &'static str) {
    if active {
        (label, "success")
    } else {
        (label, "neutral")
    }
}

#[component]
pub fn SearchPage() -> impl IntoView {
    let navigate = use_navigate();

    let (provider, set_provider) = signal("SRT".to_string());
    let (departure, set_departure) = signal(String::new());
    let (arrival, set_arrival) = signal(String::new());
    let (travel_date, set_travel_date) = signal(String::new());
    let (time_value, set_time_value) = signal("08:00".to_string());
    let (adult_count, set_adult_count) = signal(1u8);
    let (child_count, set_child_count) = signal(0u8);
    let (senior_count, set_senior_count) = signal(0u8);
    let (seat_preference, set_seat_preference) = signal("GeneralFirst".to_string());
    let (auto_pay, set_auto_pay) = signal(false);
    let (notify_enabled, set_notify_enabled) = signal(true);
    let (auto_retry, set_auto_retry) = signal(false);
    let (selected_card_id, set_selected_card_id) = signal(String::new());
    let (selected_trains, set_selected_trains) = signal(Vec::<TargetTrain>::new());
    let (form_error, set_form_error) = signal(String::new());
    let (create_error, set_create_error) = signal(String::new());

    let stations = Resource::new(move || provider.get(), api::list_stations);
    let cards = Resource::new(|| (), |_| api::list_cards());

    let search_action = Action::new(move |_: &()| {
        let provider = provider.get_untracked();
        let departure = departure.get_untracked();
        let arrival = arrival.get_untracked();
        let travel_date = travel_date.get_untracked();
        let time_value = time_value.get_untracked();

        async move {
            api::search_trains(
                provider,
                departure,
                arrival,
                if travel_date.is_empty() {
                    None
                } else {
                    Some(travel_date.replace('-', ""))
                },
                normalize_time_input(&time_value),
            )
            .await
        }
    });

    let create_action = Action::new(move |_: &()| {
        let provider = parse_provider(&provider.get_untracked());
        let departure = departure.get_untracked();
        let arrival = arrival.get_untracked();
        let travel_date = travel_date.get_untracked().replace('-', "");
        let time_value = time_value.get_untracked();
        let selected_trains = selected_trains.get_untracked();
        let adult_count = adult_count.get_untracked();
        let child_count = child_count.get_untracked();
        let senior_count = senior_count.get_untracked();
        let seat_preference = parse_seat_preference(&seat_preference.get_untracked());
        let auto_pay = auto_pay.get_untracked();
        let selected_card_id = selected_card_id.get_untracked();
        let notify_enabled = notify_enabled.get_untracked();
        let auto_retry = auto_retry.get_untracked();

        async move {
            api::create_task(CreateTaskInput {
                provider,
                departure_station: departure,
                arrival_station: arrival,
                travel_date,
                departure_time: normalize_time_input(&time_value)
                    .unwrap_or_else(|| "080000".to_string()),
                passengers: build_passenger_list(adult_count, child_count, senior_count),
                seat_preference,
                target_trains: TargetTrainList(selected_trains),
                auto_pay,
                payment_card_id: if selected_card_id.is_empty() {
                    None
                } else {
                    Uuid::parse_str(&selected_card_id).ok()
                },
                notify_enabled,
                auto_retry,
            })
            .await
        }
    });

    Effect::new(move |_| {
        if let Some(result) = create_action.value().get() {
            match result {
                Ok(_) => {
                    set_create_error.set(String::new());
                    navigate("/tasks", NavigateOptions::default());
                }
                Err(error) => {
                    set_create_error.set(format_server_error(&error));
                }
            }
        }
    });

    let station_names = move || {
        stations
            .get()
            .and_then(Result::ok)
            .unwrap_or_default()
            .into_iter()
            .map(|station| station.name_ko)
            .collect::<Vec<_>>()
    };

    let total_passengers = move || adult_count.get() + child_count.get() + senior_count.get();

    view! {
        <ProtectedPage>
            <section class="mx-auto flex w-full max-w-5xl flex-col gap-6 px-1 md:px-4">
                <section class="lg-page-card">
                    <div class="flex flex-col gap-3 lg:flex-row lg:items-end lg:justify-between">
                        <div class="space-y-2">
                            <p class="lg-route-kicker">{t("search.provider")}</p>
                            <h1 class="text-3xl font-semibold tracking-tight">{t("search.title")}</h1>
                            <p class="text-sm" style="color: var(--lg-text-secondary);">
                                "Search one provider at a time, compare live seat availability, and queue the trains you want the worker to watch."
                            </p>
                        </div>

                        <div class="flex items-center gap-2">
                            <button
                                type="button"
                                class=move || provider_badge_class("SRT", provider.get() == "SRT")
                                on:click=move |_| set_provider.set("SRT".to_string())
                            >
                                "SRT"
                            </button>
                            <button
                                type="button"
                                class=move || provider_badge_class("KTX", provider.get() == "KTX")
                                on:click=move |_| set_provider.set("KTX".to_string())
                            >
                                "KTX"
                            </button>
                        </div>
                    </div>
                </section>

                <section class="grid gap-4 xl:grid-cols-[minmax(0,1.1fr)_minmax(24rem,0.9fr)]">
                    <div class="lg-page-card">
                        <div class="grid gap-4 md:grid-cols-2">
                            <label class="lg-field">
                                <span>{t("search.departure")}</span>
                                <input
                                    list="search-station-options"
                                    type="text"
                                    placeholder=t("search.select_station")
                                    prop:value=move || departure.get()
                                    on:input=move |event| set_departure.set(event_target_value(&event))
                                />
                            </label>

                            <label class="lg-field">
                                <span>{t("search.arrival")}</span>
                                <input
                                    list="search-station-options"
                                    type="text"
                                    placeholder=t("search.select_station")
                                    prop:value=move || arrival.get()
                                    on:input=move |event| set_arrival.set(event_target_value(&event))
                                />
                            </label>

                            <label class="lg-field">
                                <span>{t("search.date")}</span>
                                <input
                                    type="date"
                                    prop:value=move || travel_date.get()
                                    on:input=move |event| set_travel_date.set(event_target_value(&event))
                                />
                            </label>

                            <label class="lg-field">
                                <span>{t("search.time")}</span>
                                <input
                                    type="time"
                                    step="1800"
                                    prop:value=move || time_value.get()
                                    on:input=move |event| set_time_value.set(event_target_value(&event))
                                />
                                <small>"Search from this departure time onward."</small>
                            </label>
                        </div>

                        <datalist id="search-station-options">
                            {move || station_names().into_iter().map(|name| {
                                view! { <option value=name></option> }
                            }).collect::<Vec<_>>()}
                        </datalist>

                        <div class="mt-5 grid gap-4 md:grid-cols-3">
                            <label class="lg-field">
                                <span>"Adults"</span>
                                <input
                                    type="number"
                                    min="1"
                                    max="9"
                                    prop:value=move || adult_count.get().to_string()
                                    on:input=move |event| {
                                        set_adult_count.set(
                                            event_target_value(&event).parse::<u8>().ok().unwrap_or(1).max(1),
                                        )
                                    }
                                />
                            </label>

                            <label class="lg-field">
                                <span>"Children"</span>
                                <input
                                    type="number"
                                    min="0"
                                    max="9"
                                    prop:value=move || child_count.get().to_string()
                                    on:input=move |event| {
                                        set_child_count.set(
                                            event_target_value(&event).parse::<u8>().ok().unwrap_or(0),
                                        )
                                    }
                                />
                            </label>

                            <label class="lg-field">
                                <span>"Seniors"</span>
                                <input
                                    type="number"
                                    min="0"
                                    max="9"
                                    prop:value=move || senior_count.get().to_string()
                                    on:input=move |event| {
                                        set_senior_count.set(
                                            event_target_value(&event).parse::<u8>().ok().unwrap_or(0),
                                        )
                                    }
                                />
                            </label>
                        </div>

                        {move || {
                            let error = form_error.get();
                            if error.is_empty() {
                                None
                            } else {
                                Some(view! {
                                    <p class="mt-4 text-sm" style="color: var(--lg-error);">{error}</p>
                                })
                            }
                        }}

                        <div class="mt-5 flex flex-wrap gap-3">
                            <button
                                type="button"
                                class="lg-btn-primary"
                                on:click=move |_| {
                                    set_form_error.set(String::new());
                                    set_create_error.set(String::new());

                                    let departure = departure.get_untracked();
                                    let arrival = arrival.get_untracked();
                                    if departure.trim().is_empty() || arrival.trim().is_empty() {
                                        set_form_error.set("Choose both departure and arrival stations.".to_string());
                                        return;
                                    }
                                    if departure == arrival {
                                        set_form_error.set("Departure and arrival must differ.".to_string());
                                        return;
                                    }

                                    set_selected_trains.set(Vec::new());
                                    search_action.dispatch(());
                                }
                            >
                                {move || if search_action.pending().get() {
                                    t("search.searching")
                                } else {
                                    t("search.search_btn")
                                }}
                            </button>

                            <span class="inline-flex items-center rounded-full border border-white/10 px-4 py-2 text-xs uppercase tracking-[0.18em]" style="color: var(--lg-text-tertiary);">
                                {move || format!("{} {}", total_passengers(), t("search.passengers"))}
                            </span>
                        </div>
                    </div>

                    <div class="lg-page-card">
                        <div class="flex items-center justify-between gap-3">
                            <div>
                                <p class="lg-route-kicker">"Task Setup"</p>
                                <h2 class="text-2xl font-semibold tracking-tight">"Queue Selection"</h2>
                            </div>
                            <span class="inline-flex items-center rounded-full border border-white/10 px-3 py-1 text-xs" style="color: var(--lg-text-secondary);">
                                {move || format!("{} selected", selected_trains.get().len())}
                            </span>
                        </div>

                        <div class="mt-5 grid gap-4">
                            <label class="lg-field">
                                <span>{t("search.seat_preference")}</span>
                                <select
                                    prop:value=move || seat_preference.get()
                                    on:change=move |event| set_seat_preference.set(event_target_value(&event))
                                >
                                    <option value="GeneralFirst">{t("search.seat_general_first")}</option>
                                    <option value="SpecialFirst">{t("search.seat_special_first")}</option>
                                    <option value="GeneralOnly">{t("search.seat_general_only")}</option>
                                    <option value="SpecialOnly">{t("search.seat_special_only")}</option>
                                </select>
                            </label>

                            <label class="lg-toggle-row">
                                <span>{t("search.auto_pay")}</span>
                                <input
                                    type="checkbox"
                                    prop:checked=move || auto_pay.get()
                                    on:change=move |event| set_auto_pay.set(event_target_checked(&event))
                                />
                            </label>

                            <Show when=move || auto_pay.get()>
                                <label class="lg-field">
                                    <span>{t("search.select_card")}</span>
                                    <select
                                        prop:value=move || selected_card_id.get()
                                        on:change=move |event| set_selected_card_id.set(event_target_value(&event))
                                    >
                                        <option value="">{t("search.select_card")}</option>
                                        {move || {
                                            cards
                                                .get()
                                                .and_then(Result::ok)
                                                .unwrap_or_default()
                                                .into_iter()
                                                .map(|card| {
                                                    view! {
                                                        <option value=card.id.to_string()>
                                                            {format!("{} •••• {}", card.label, card.last_four)}
                                                        </option>
                                                    }
                                                })
                                                .collect::<Vec<_>>()
                                        }}
                                    </select>
                                    <small>{move || {
                                        if cards.get().and_then(Result::ok).unwrap_or_default().is_empty() {
                                            t("search.no_cards")
                                        } else {
                                            ""
                                        }
                                    }}</small>
                                </label>
                            </Show>

                            <label class="lg-toggle-row">
                                <span>{t("search.notify")}</span>
                                <input
                                    type="checkbox"
                                    prop:checked=move || notify_enabled.get()
                                    on:change=move |event| set_notify_enabled.set(event_target_checked(&event))
                                />
                            </label>

                            <label class="lg-toggle-row">
                                <span>{t("search.auto_retry")}</span>
                                <input
                                    type="checkbox"
                                    prop:checked=move || auto_retry.get()
                                    on:change=move |event| set_auto_retry.set(event_target_checked(&event))
                                />
                            </label>
                        </div>

                        {move || {
                            let error = create_error.get();
                            if error.is_empty() {
                                None
                            } else {
                                Some(view! {
                                    <p class="mt-4 text-sm" style="color: var(--lg-error);">{error}</p>
                                })
                            }
                        }}

                        <button
                            type="button"
                            class="lg-btn-primary mt-5"
                            disabled=move || selected_trains.get().is_empty() || create_action.pending().get()
                            on:click=move |_| {
                                set_create_error.set(String::new());

                                if selected_trains.get_untracked().is_empty() {
                                    set_create_error.set("Select at least one train first.".to_string());
                                    return;
                                }

                                if auto_pay.get_untracked() && selected_card_id.get_untracked().is_empty() {
                                    set_create_error.set(t("search.auto_pay_card_required").to_string());
                                    return;
                                }

                                create_action.dispatch(());
                            }
                        >
                            {move || if create_action.pending().get() {
                                "Creating task..."
                            } else {
                                "Create reservation task"
                            }}
                        </button>
                    </div>
                </section>

                <section class="lg-page-card">
                    <div class="flex items-center justify-between gap-3">
                        <div>
                            <p class="lg-route-kicker">"Results"</p>
                            <h2 class="text-2xl font-semibold tracking-tight">"Available trains"</h2>
                        </div>
                        <span class="inline-flex items-center rounded-full border border-white/10 px-3 py-1 text-xs" style="color: var(--lg-text-secondary);">
                            {move || provider.get()}
                        </span>
                    </div>

                    {move || match search_action.value().get() {
                        None => {
                            view! {
                                <div class="lg-empty-state mt-5">
                                    <p>"Run a search to load departure results."</p>
                                </div>
                            }
                            .into_any()
                        }
                        Some(Err(error)) => {
                            view! {
                                <div class="lg-empty-state mt-5">
                                    <p style="color: var(--lg-error);">{format_server_error(&error)}</p>
                                </div>
                            }
                            .into_any()
                        }
                        Some(Ok(results)) if results.is_empty() => {
                            view! {
                                <div class="lg-empty-state mt-5">
                                    <p>"No trains matched the current search."</p>
                                </div>
                            }
                            .into_any()
                        }
                        Some(Ok(results)) => {
                            view! {
                                <div class="mt-5 grid gap-3">
                                    {results.into_iter().map(|train| {
                                        let selected_now = Signal::derive({
                                            let train = train.clone();
                                            move || is_selected(&selected_trains.get(), &train)
                                        });
                                        let train_for_click = train.clone();
                                        view! {
                                            <button
                                                type="button"
                                                class=move || {
                                                    if selected_now.get() {
                                                        "lg-result-card lg-result-card--selected"
                                                    } else {
                                                        "lg-result-card"
                                                    }
                                                }
                                                on:click=move |_| {
                                                    let candidate = train_selection(&train_for_click);
                                                    set_selected_trains.update(|items| {
                                                        if let Some(index) = items.iter().position(|item| {
                                                            item.train_number == candidate.train_number
                                                                && item.dep_time == candidate.dep_time
                                                        }) {
                                                            items.remove(index);
                                                        } else {
                                                            items.push(candidate.clone());
                                                        }
                                                    });
                                                }
                                            >
                                                <div class="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
                                                    <div class="space-y-2 text-left">
                                                        <div class="flex items-center gap-2">
                                                            <span class="text-sm font-semibold">
                                                                {format!("{} {}", train.provider, train.train_number)}
                                                            </span>
                                                            <span class="text-xs" style="color: var(--lg-text-secondary);">
                                                                {train.train_type_name.clone()}
                                                            </span>
                                                        </div>
                                                        <p class="text-base font-medium tracking-tight">
                                                            {format!(
                                                                "{} {} -> {} {}",
                                                                format_time(&train.dep_time),
                                                                train.dep_station,
                                                                format_time(&train.arr_time),
                                                                train.arr_station,
                                                            )}
                                                        </p>
                                                    </div>

                                                    <div class="flex flex-wrap items-center gap-2">
                                                        {{
                                                            let (general_label, general_variant) =
                                                                seat_chip(t("seat.general"), train.general_available);
                                                            view! {
                                                                <StatusChip
                                                                    label=general_label
                                                                    variant=general_variant
                                                                />
                                                            }
                                                        }}
                                                        {{
                                                            let (special_label, special_variant) =
                                                                seat_chip(t("seat.special"), train.special_available);
                                                            view! {
                                                                <StatusChip
                                                                    label=special_label
                                                                    variant=special_variant
                                                                />
                                                            }
                                                        }}
                                                        <StatusChip
                                                            label="Standby"
                                                            variant=if train.standby_available { "warning" } else { "neutral" }
                                                        />
                                                    </div>
                                                </div>
                                            </button>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }
                            .into_any()
                        }
                    }}
                </section>
            </section>
        </ProtectedPage>
    }
}
