//! Reservations page with provider filter and pay/cancel/refund actions.

use leptos::prelude::*;

use crate::{
    api,
    components::{Skeleton, SseReload, StatusChip},
    i18n::t,
    types::{CardInfo, ReservationInfo, TicketInfo},
    utils::{format_cost, format_date, format_time},
};

use super::{ProtectedPage, format_server_error};

fn reservation_status(reservation: &ReservationInfo) -> (&'static str, &'static str) {
    if reservation.paid {
        (t("reservation.paid"), "success")
    } else if reservation.is_waiting {
        (t("reservation.waiting"), "warning")
    } else {
        (t("reservation.unpaid"), "error")
    }
}

#[component]
fn ReservationCard(
    reservation: ReservationInfo,
    cards: Vec<CardInfo>,
    on_cancel: Callback<()>,
    on_pay: Callback<String>,
    on_refund: Callback<()>,
) -> impl IntoView {
    let (expanded, set_expanded) = signal(false);
    let default_card = cards
        .first()
        .map(|card| card.id.to_string())
        .unwrap_or_default();
    let (selected_card, set_selected_card) = signal(default_card);

    let reservation_number = reservation.reservation_number.clone();
    let provider = reservation.provider.clone();
    let tickets = Resource::new(
        move || expanded.get(),
        move |open| {
            let provider = provider.clone();
            let reservation_number = reservation_number.clone();
            async move {
                if open {
                    api::ticket_detail(provider, reservation_number).await
                } else {
                    Ok(Vec::<TicketInfo>::new())
                }
            }
        },
    );

    let (status_label, status_variant) = reservation_status(&reservation);
    let show_cancel = !reservation.paid;
    let show_pay = !reservation.paid && !reservation.is_waiting;
    let show_refund = reservation.paid;
    let has_deadline = !reservation.payment_deadline_date.is_empty();
    let has_cards = !cards.is_empty();

    view! {
        <article class="lg-list-card lg-list-card--reservation">
            <div class="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
                <div class="space-y-2">
                    <div class="flex flex-wrap items-center gap-2">
                        <span class="inline-flex items-center rounded-full border border-white/10 px-2.5 py-1 text-[10px] font-semibold uppercase tracking-[0.18em]" style="color: var(--lg-text-tertiary);">
                            {reservation.provider.clone()}
                        </span>
                        <StatusChip label=status_label variant=status_variant />
                    </div>

                    <h3 class="text-lg font-semibold tracking-tight">
                        {format!("{} {}", reservation.train_name, reservation.train_number)}
                    </h3>

                    <p class="text-sm" style="color: var(--lg-text-secondary);">
                        {format!(
                            "{} {} -> {} {}",
                            format_time(&reservation.dep_time),
                            reservation.dep_station,
                            format_time(&reservation.arr_time),
                            reservation.arr_station,
                        )}
                    </p>

                    <div class="flex flex-wrap items-center gap-3 text-xs" style="color: var(--lg-text-tertiary);">
                        <span>{format_date(&reservation.dep_date)}</span>
                        <span>{format!("{} seats", reservation.seat_count)}</span>
                        <span>{format!("₩{}", format_cost(&reservation.total_cost))}</span>
                    </div>

                    {if has_deadline {
                        Some(view! {
                            <p class="text-xs" style="color: var(--lg-text-secondary);">
                                {format!(
                                    "Pay by {} {}",
                                    format_date(&reservation.payment_deadline_date),
                                    format_time(&reservation.payment_deadline_time),
                                )}
                            </p>
                        })
                    } else {
                        None
                    }}
                </div>

                <div class="flex flex-col items-stretch gap-2 lg:min-w-52">
                    <button
                        type="button"
                        class="lg-btn-secondary text-xs"
                        on:click=move |_| set_expanded.update(|open| *open = !*open)
                    >
                        {move || if expanded.get() { "Hide details" } else { "View tickets" }}
                    </button>

                    <Show when=move || show_pay>
                        <select
                            class="lg-select"
                            prop:value=move || selected_card.get()
                            on:change=move |event| set_selected_card.set(event_target_value(&event))
                        >
                            {cards.iter().map(|card| {
                                view! {
                                    <option value=card.id.to_string()>
                                        {format!("{} •••• {}", card.label, card.last_four)}
                                    </option>
                                }
                            }).collect::<Vec<_>>()}
                        </select>
                    </Show>
                </div>
            </div>

            <Show when=move || expanded.get()>
                <div class="mt-4 rounded-2xl border border-white/8 bg-white/4 p-4">
                    <Suspense fallback=move || view! {
                        <div class="space-y-2">
                            <div class="lg-skeleton-line h-12"></div>
                            <div class="lg-skeleton-line h-12"></div>
                        </div>
                    }>
                        {move || {
                            tickets.get().map(|result| match result {
                                Ok(items) if items.is_empty() => {
                                    view! {
                                        <div class="lg-empty-state">
                                            <p>"Ticket details are not available for this reservation yet."</p>
                                        </div>
                                    }
                                    .into_any()
                                }
                                Ok(items) => {
                                    view! {
                                        <div class="space-y-2">
                                            {items.into_iter().map(|ticket| {
                                                view! {
                                                    <div class="flex items-center justify-between rounded-2xl border border-white/8 px-3 py-2 text-sm">
                                                        <div>
                                                            <p class="font-medium">
                                                                {format!("Car {} • Seat {}", ticket.car, ticket.seat)}
                                                            </p>
                                                            <p style="color: var(--lg-text-secondary);">
                                                                {format!("{} • {}", ticket.passenger_type, ticket.seat_type)}
                                                            </p>
                                                        </div>
                                                        <span class="font-semibold">
                                                            {format!("₩{}", ticket.price)}
                                                        </span>
                                                    </div>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    }
                                    .into_any()
                                }
                                Err(error) => {
                                    view! {
                                        <div class="lg-empty-state">
                                            <p style="color: var(--lg-error);">{format_server_error(&error)}</p>
                                        </div>
                                    }
                                    .into_any()
                                }
                            })
                        }}
                    </Suspense>
                </div>
            </Show>

            <div class="mt-4 flex flex-wrap gap-2">
                <Show when=move || show_cancel>
                    <button type="button" class="lg-btn-secondary text-xs" on:click=move |_| on_cancel.run(())>
                        "Cancel reservation"
                    </button>
                </Show>

                <Show when=move || show_pay>
                    <button
                        type="button"
                        class="lg-btn-primary text-xs"
                        disabled=move || !has_cards || selected_card.get().is_empty()
                        on:click=move |_| on_pay.run(selected_card.get())
                    >
                        "Pay now"
                    </button>
                </Show>

                <Show when=move || show_refund>
                    <button type="button" class="lg-btn-secondary text-xs" on:click=move |_| on_refund.run(())>
                        "Request refund"
                    </button>
                </Show>
            </div>
        </article>
    }
}

#[component]
pub fn ReservationsPage() -> impl IntoView {
    let (provider, set_provider) = signal("SRT".to_string());

    let reservations = Resource::new(move || provider.get(), api::list_reservations);
    let cards = Resource::new(|| (), |_| api::list_cards());

    let cancel_action = Action::new(|input: &(String, String)| {
        let (provider, reservation_number) = input.clone();
        async move { api::cancel_reservation(provider, reservation_number).await }
    });

    let pay_action = Action::new(|input: &(String, String, String)| {
        let (provider, reservation_number, card_id) = input.clone();
        async move { api::pay_reservation(provider, reservation_number, card_id).await }
    });

    let refund_action = Action::new(|input: &(String, String)| {
        let (provider, reservation_number) = input.clone();
        async move { api::refund_reservation(provider, reservation_number).await }
    });

    Effect::new(move |_| {
        if let Some(Ok(())) = cancel_action.value().get() {
            reservations.refetch();
        }
    });

    Effect::new(move |_| {
        if let Some(Ok(())) = pay_action.value().get() {
            reservations.refetch();
        }
    });

    Effect::new(move |_| {
        if let Some(Ok(())) = refund_action.value().get() {
            reservations.refetch();
        }
    });

    view! {
        <ProtectedPage>
            <SseReload on_event=Callback::new(move |_| { reservations.refetch(); }) />

            <section class="mx-auto flex w-full max-w-5xl flex-col gap-6 px-1 md:px-4">
                <section class="lg-page-card">
                    <div class="flex flex-col gap-4 md:flex-row md:items-end md:justify-between">
                        <div>
                            <p class="lg-route-kicker">{t("reservation.title")}</p>
                            <h1 class="text-3xl font-semibold tracking-tight">{t("reservation.title")}</h1>
                            <p class="mt-2 text-sm" style="color: var(--lg-text-secondary);">
                                "Switch provider views, inspect ticket details, and handle payment, cancel, or refund actions from one place."
                            </p>
                        </div>

                        <div class="lg-tab-strip">
                            <button
                                type="button"
                                class=move || if provider.get() == "SRT" { "lg-tab-pill lg-tab-pill--active" } else { "lg-tab-pill" }
                                on:click=move |_| set_provider.set("SRT".to_string())
                            >
                                "SRT"
                            </button>
                            <button
                                type="button"
                                class=move || if provider.get() == "KTX" { "lg-tab-pill lg-tab-pill--active" } else { "lg-tab-pill" }
                                on:click=move |_| set_provider.set("KTX".to_string())
                            >
                                "KTX"
                            </button>
                        </div>
                    </div>

                    {move || cancel_action.value().get().map(|result| match result {
                        Ok(()) => {
                            view! {
                                <div class="lg-inline-alert lg-inline-alert--success mt-5">
                                    {t("reservation.cancelled")}
                                </div>
                            }
                            .into_any()
                        }
                        Err(error) => {
                            view! {
                                <div class="lg-inline-alert lg-inline-alert--error mt-5">
                                    {format_server_error(&error)}
                                </div>
                            }
                            .into_any()
                        }
                    })}

                    {move || pay_action.value().get().map(|result| match result {
                        Ok(()) => {
                            view! {
                                <div class="lg-inline-alert lg-inline-alert--success mt-3">
                                    {t("reservation.payment_success")}
                                </div>
                            }
                            .into_any()
                        }
                        Err(error) => {
                            view! {
                                <div class="lg-inline-alert lg-inline-alert--error mt-3">
                                    {format_server_error(&error)}
                                </div>
                            }
                            .into_any()
                        }
                    })}

                    {move || refund_action.value().get().map(|result| match result {
                        Ok(()) => {
                            view! {
                                <div class="lg-inline-alert lg-inline-alert--success mt-3">
                                    {t("reservation.refunded")}
                                </div>
                            }
                            .into_any()
                        }
                        Err(error) => {
                            view! {
                                <div class="lg-inline-alert lg-inline-alert--error mt-3">
                                    {format_server_error(&error)}
                                </div>
                            }
                            .into_any()
                        }
                    })}
                </section>

                <section class="lg-page-card">
                    <Suspense fallback=move || view! {
                        <div class="space-y-3">
                            <Skeleton height="h-28" />
                            <Skeleton height="h-28" />
                        </div>
                    }>
                        {move || {
                            reservations.get().map(|result| match result {
                                Ok(items) if items.is_empty() => {
                                    view! {
                                        <div class="lg-empty-state">
                                            <p>"No reservations found for this provider."</p>
                                            <a href="/search" class="lg-btn-secondary text-xs">
                                                {t("search.go_to_search")}
                                            </a>
                                        </div>
                                    }
                                    .into_any()
                                }
                                Ok(items) => {
                                    let payment_cards = cards.get().and_then(Result::ok).unwrap_or_default();
                                    view! {
                                        <div class="space-y-3">
                                            {items.into_iter().map(|reservation| {
                                                let reservation_number = reservation.reservation_number.clone();
                                                let provider = reservation.provider.clone();
                                                let pay_reservation_number = reservation.reservation_number.clone();
                                                let pay_provider = reservation.provider.clone();
                                                let refund_reservation_number = reservation.reservation_number.clone();
                                                let refund_provider = reservation.provider.clone();
                                                let cards_for_row = payment_cards.clone();
                                                view! {
                                                    <ReservationCard
                                                        reservation=reservation
                                                        cards=cards_for_row
                                                        on_cancel=Callback::new(move |_| {
                                                            cancel_action.dispatch((provider.clone(), reservation_number.clone()));
                                                        })
                                                        on_pay=Callback::new(move |card_id| {
                                                            pay_action.dispatch((
                                                                pay_provider.clone(),
                                                                pay_reservation_number.clone(),
                                                                card_id,
                                                            ));
                                                        })
                                                        on_refund=Callback::new(move |_| {
                                                            refund_action.dispatch((
                                                                refund_provider.clone(),
                                                                refund_reservation_number.clone(),
                                                            ));
                                                        })
                                                    />
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    }
                                    .into_any()
                                }
                                Err(error) => {
                                    view! {
                                        <div class="lg-empty-state">
                                            <p style="color: var(--lg-error);">{format_server_error(&error)}</p>
                                        </div>
                                    }
                                    .into_any()
                                }
                            })
                        }}
                    </Suspense>
                </section>
            </section>
        </ProtectedPage>
    }
}
