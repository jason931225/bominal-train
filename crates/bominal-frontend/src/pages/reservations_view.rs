//! Reservations view — shows confirmed reservations with pay/cancel actions.

use leptos::prelude::*;

use crate::api::cards::{list_cards, CardInfo};
use crate::api::reservations::{cancel_reservation, list_reservations, pay_reservation, ReservationInfo};
use crate::components::glass_panel::GlassPanel;
use crate::i18n::t;
use crate::utils::{format_cost, format_date, format_time};

/// Reservations page showing active reservations from SRT and KTX.
#[component]
pub fn ReservationsView() -> impl IntoView {
    let (provider, set_provider) = signal("SRT".to_string());

    let reservations = Resource::new(
        move || provider.get(),
        |prov| list_reservations(prov),
    );

    let cards = Resource::new(|| (), |_| list_cards());

    let cancel_action = Action::new(|input: &(String, String)| {
        let (prov, pnr) = input.clone();
        async move { cancel_reservation(prov, pnr).await }
    });

    // (provider, reservation_number, card_id)
    let pay_action = Action::new(|input: &(String, String, String)| {
        let (prov, pnr, card_id) = input.clone();
        async move { pay_reservation(prov, pnr, card_id).await }
    });

    // Refetch after cancellation
    Effect::new(move || {
        if let Some(Ok(())) = cancel_action.value().get() {
            reservations.refetch();
        }
    });

    // Refetch after payment
    Effect::new(move || {
        if let Some(Ok(())) = pay_action.value().get() {
            reservations.refetch();
        }
    });

    view! {
        <div class="px-4 pt-6 pb-4 space-y-4 max-w-xl lg:max-w-2xl mx-auto page-enter">
            <h1 class="text-xl font-bold text-[var(--color-text-primary)]">{t("reservation.title")}</h1>

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

            // Cancel result feedback
            {move || cancel_action.value().get().map(|result| match result {
                Ok(()) => view! {
                    <div class="px-3 py-2 bg-[var(--color-status-success)]/10 border border-[var(--color-status-success)]/30 rounded-xl">
                        <p class="text-sm text-[var(--color-status-success)]">{t("reservation.cancelled")}</p>
                    </div>
                }.into_any(),
                Err(e) => view! {
                    <div class="px-3 py-2 bg-[var(--color-status-error)]/10 border border-[var(--color-status-error)]/30 rounded-xl">
                        <p class="text-sm text-[var(--color-status-error)]">{format!("{e}")}</p>
                    </div>
                }.into_any(),
            })}

            // Pay result feedback
            {move || pay_action.value().get().map(|result| match result {
                Ok(()) => view! {
                    <div class="px-3 py-2 bg-[var(--color-status-success)]/10 border border-[var(--color-status-success)]/30 rounded-xl">
                        <p class="text-sm text-[var(--color-status-success)]">{t("reservation.payment_success")}</p>
                    </div>
                }.into_any(),
                Err(e) => view! {
                    <div class="px-3 py-2 bg-[var(--color-status-error)]/10 border border-[var(--color-status-error)]/30 rounded-xl">
                        <p class="text-sm text-[var(--color-status-error)]">{format!("{e}")}</p>
                    </div>
                }.into_any(),
            })}

            // Reservation list
            <Suspense fallback=move || view! {
                <GlassPanel>
                    <div class="p-4 text-center py-8">
                        <p class="text-[var(--color-text-tertiary)] text-sm">{t("common.loading")}</p>
                    </div>
                </GlassPanel>
            }>
                {move || reservations.get().map(|result| match result {
                    Ok(list) if list.is_empty() => view! {
                        <GlassPanel>
                            <div class="p-4 text-center py-12">
                                <p class="text-[var(--color-text-tertiary)] text-sm">{t("reservation.no_active")}</p>
                                <a href="/search" class="inline-block mt-3 text-sm text-[var(--color-brand-text)] font-medium hover:underline">
                                    {t("search.title")}
                                </a>
                            </div>
                        </GlassPanel>
                    }.into_any(),
                    Ok(list) => {
                        let card_list: Vec<CardInfo> = cards.get()
                            .and_then(|r| r.ok())
                            .unwrap_or_default();
                        view! {
                            <div class="space-y-3">
                                {list.into_iter().map(|rsv| {
                                    let prov = rsv.provider.clone();
                                    let pnr = rsv.reservation_number.clone();
                                    let pay_prov = rsv.provider.clone();
                                    let pay_pnr = rsv.reservation_number.clone();
                                    let cards_for_card = card_list.clone();
                                    view! {
                                        <ReservationCard
                                            rsv=rsv
                                            cards=cards_for_card
                                            on_cancel=move || {
                                                cancel_action.dispatch((prov.clone(), pnr.clone()));
                                            }
                                            on_pay=move |card_id: String| {
                                                pay_action.dispatch((pay_prov.clone(), pay_pnr.clone(), card_id));
                                            }
                                        />
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any()
                    },
                    Err(e) => view! {
                        <GlassPanel>
                            <div class="p-4 text-center py-8">
                                <p class="text-[var(--color-status-error)] text-sm">{format!("{e}")}</p>
                            </div>
                        </GlassPanel>
                    }.into_any(),
                })}
            </Suspense>
        </div>
    }
}

/// Reservation card with details and actions.
#[component]
fn ReservationCard(
    rsv: ReservationInfo,
    cards: Vec<CardInfo>,
    on_cancel: impl Fn() + Send + Sync + 'static,
    on_pay: impl Fn(String) + Send + Sync + 'static,
) -> impl IntoView {
    let status_label = if rsv.paid {
        t("reservation.paid")
    } else if rsv.is_waiting {
        t("reservation.waiting")
    } else {
        t("reservation.unpaid")
    };
    let status_class = if rsv.paid {
        "text-[10px] px-1.5 py-0.5 rounded bg-[var(--color-status-success)]/20 text-[var(--color-status-success)]"
    } else if rsv.is_waiting {
        "text-[10px] px-1.5 py-0.5 rounded bg-[var(--color-status-warning)]/20 text-[var(--color-status-warning)]"
    } else {
        "text-[10px] px-1.5 py-0.5 rounded bg-[var(--color-status-error)]/20 text-[var(--color-status-error)]"
    };

    let has_deadline = !rsv.payment_deadline_date.is_empty();
    let deadline_text = if has_deadline {
        format!(
            "Pay by {} {}",
            format_date(&rsv.payment_deadline_date),
            format_time(&rsv.payment_deadline_time),
        )
    } else {
        String::new()
    };

    let show_cancel = !rsv.paid;
    let show_pay = !rsv.paid && !rsv.is_waiting && !cards.is_empty();

    // Selected card for payment
    let default_card_id = cards.first().map(|c| c.id.to_string()).unwrap_or_default();
    let (selected_card, set_selected_card) = signal(default_card_id);

    view! {
        <GlassPanel variant=crate::components::glass_panel::GlassPanelVariant::Card hover=true>
            <div class="p-4">
                <div class="flex items-start justify-between mb-2">
                    <div class="flex items-center gap-2">
                        <span class="text-xs px-1.5 py-0.5 rounded bg-[var(--color-bg-sunken)] text-[var(--color-text-secondary)]">
                            {rsv.provider.clone()}
                        </span>
                        <span class=status_class>{status_label}</span>
                    </div>
                    <span class="text-xs font-mono text-[var(--color-text-tertiary)]">
                        {rsv.reservation_number.clone()}
                    </span>
                </div>

                // Train info
                <div class="flex items-center gap-2 mb-1">
                    <span class="text-xs px-1.5 py-0.5 rounded bg-[var(--color-bg-sunken)] text-[var(--color-text-secondary)] font-mono">
                        {rsv.train_number.clone()}
                    </span>
                    <span class="text-xs text-[var(--color-text-tertiary)]">{rsv.train_name.clone()}</span>
                </div>

                // Route
                <p class="text-sm font-medium text-[var(--color-text-primary)]">
                    {format!("{} → {}", rsv.dep_station, rsv.arr_station)}
                </p>

                // Date & time
                <div class="flex items-center gap-2 mt-1">
                    <span class="text-xs text-[var(--color-text-tertiary)]">
                        {format_date(&rsv.dep_date)}
                    </span>
                    <span class="text-sm font-medium text-[var(--color-text-primary)]">
                        {format_time(&rsv.dep_time)}
                    </span>
                    <span class="text-xs text-[var(--color-text-tertiary)]">"→"</span>
                    <span class="text-sm font-medium text-[var(--color-text-primary)]">
                        {format_time(&rsv.arr_time)}
                    </span>
                </div>

                // Cost & seats
                <div class="flex items-center gap-3 mt-2">
                    <span class="text-sm font-medium text-[var(--color-text-primary)]">
                        {format!("₩{}", format_cost(&rsv.total_cost))}
                    </span>
                    <span class="text-xs text-[var(--color-text-tertiary)]">
                        {format!("{}석", rsv.seat_count)}
                    </span>
                </div>

                // Payment deadline
                {(!deadline_text.is_empty()).then(|| view! {
                    <p class="text-xs text-[var(--color-status-warning)] mt-2">{deadline_text.clone()}</p>
                })}

                // Actions
                {show_cancel.then(|| view! {
                    <div class="mt-3 pt-3 border-t border-[var(--color-border-subtle)] space-y-2">
                        // Pay section
                        {show_pay.then(|| {
                            let pay_cards = cards.clone();
                            view! {
                                <div class="flex gap-2">
                                    <select
                                        class="flex-1 text-xs bg-[var(--color-bg-sunken)] text-[var(--color-text-primary)] rounded-lg px-2 py-2 border border-[var(--color-border-subtle)]"
                                        on:change=move |ev| {
                                            set_selected_card.set(event_target_value(&ev));
                                        }
                                    >
                                        {pay_cards.into_iter().map(|card| {
                                            let id = card.id.to_string();
                                            let label = format!("{} (****{})", card.label, card.last_four);
                                            view! {
                                                <option value=id>{label}</option>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </select>
                                    <button
                                        class="px-4 py-2 text-xs font-medium rounded-lg bg-[var(--color-brand-primary)] text-white hover:opacity-90 active:scale-95 transition-all"
                                        on:click=move |_| {
                                            on_pay(selected_card.get());
                                        }
                                    >
                                        {t("payment.pay")}
                                    </button>
                                </div>
                            }
                        })}
                        // Cancel button
                        <button
                            class="w-full py-2 text-xs font-medium rounded-lg text-[var(--color-status-error)] border border-[var(--color-status-error)]/30 hover:bg-[var(--color-status-error)]/10 active:scale-95 transition-all"
                            on:click=move |_| { on_cancel(); }
                        >
                            {t("reservation.cancel")}
                        </button>
                    </div>
                })}
            </div>
        </GlassPanel>
    }
}

