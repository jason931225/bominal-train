//! Review modal — reorderable train list with task creation options.

use leptos::prelude::*;

use crate::components::sortable_list::{SortableItem, SortableList};
use crate::i18n::t;

/// Summary of a selected train for review.
#[derive(Debug, Clone)]
pub struct TrainSummary {
    pub train_number: String,
    pub dep_time: String,
    pub departure: String,
    pub arrival: String,
}

/// A glass-panel modal for reviewing and reordering selected trains before task creation.
#[component]
pub fn ReviewModal(
    /// Whether the modal is open.
    open: ReadSignal<bool>,
    /// Reorderable train items.
    items: ReadSignal<Vec<SortableItem>>,
    /// Callback when items are reordered.
    on_items_change: Callback<Vec<SortableItem>>,
    /// Seat preference signal.
    seat_preference: ReadSignal<String>,
    /// Set seat preference.
    set_seat_preference: WriteSignal<String>,
    /// Auto-pay enabled.
    auto_pay: ReadSignal<bool>,
    /// Selected card ID.
    selected_card_id: ReadSignal<String>,
    /// Set selected card ID.
    set_selected_card_id: WriteSignal<String>,
    /// Available cards resource.
    cards: Resource<Result<Vec<crate::api::cards::CardInfo>, ServerFnError>>,
    /// Callback on confirm.
    on_confirm: Callback<()>,
    /// Callback on cancel/close.
    on_cancel: Callback<()>,
    /// Whether task creation is pending.
    creating: Signal<bool>,
) -> impl IntoView {
    let auto_pay_blocked = Memo::new(move |_| {
        if !auto_pay.get() {
            return false;
        }
        match cards.get() {
            Some(Ok(ref list)) if list.is_empty() => true,
            Some(Ok(_)) => selected_card_id.get().is_empty(),
            _ => false,
        }
    });

    view! {
        <Show when=move || open.get()>
            <div class="fixed inset-0 z-[160] flex items-center justify-center p-4"
                 on:click=move |_| on_cancel.run(())>
                <div class="absolute inset-0 bg-black/40 backdrop-blur-sm fade-in"></div>
                <div class="glass-panel rounded-3xl w-full max-w-md relative z-10 modal-enter flex flex-col max-h-[80vh] overflow-hidden"
                     on:click=move |e| e.stop_propagation()>
                    // Header
                    <div class="p-4 border-b border-[var(--color-border-default)] flex items-center justify-between shrink-0">
                        <h3 class="text-lg font-semibold text-[var(--color-text-primary)]">
                            {t("review.title")}
                        </h3>
                        <button
                            class="p-1.5 rounded-lg hover:bg-[var(--color-interactive-hover)] text-[var(--color-text-tertiary)]"
                            on:click=move |_| on_cancel.run(())
                        >
                            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
                            </svg>
                        </button>
                    </div>

                    // Scrollable content
                    <div class="p-4 overflow-y-auto flex-1 space-y-4 no-scrollbar">
                        // Reorder hint
                        <p class="text-sm text-[var(--color-text-tertiary)]">{t("review.reorder_hint")}</p>

                        // Sortable train list
                        <SortableList items=items on_change=on_items_change />

                        // Seat preference
                        <div>
                            <label class="block text-xs font-medium text-[var(--color-text-secondary)] mb-1.5">
                                {t("search.seat_preference")}
                            </label>
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
                                                <a href="/settings" class="text-[var(--color-brand-text)] hover:underline">{t("search.add_card")}</a>
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
                                                    let label = format!("{} \u{00B7}\u{00B7}\u{00B7}\u{00B7}{}", card.card_type_name, card.last_four);
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
                    </div>

                    // Footer
                    <div class="p-4 border-t border-[var(--color-border-default)] shrink-0 space-y-2">
                        {move || auto_pay_blocked.get().then(|| view! {
                            <p class="text-xs text-[var(--color-status-error)] text-center">
                                {t("search.auto_pay_card_required")}
                            </p>
                        })}
                        <div class="flex gap-3">
                            <button
                                class="flex-1 py-2.5 rounded-xl text-sm font-medium bg-[var(--color-bg-sunken)] text-[var(--color-text-tertiary)] hover:bg-[var(--color-interactive-hover)] transition-colors"
                                on:click=move |_| on_cancel.run(())
                            >{t("common.cancel")}</button>
                            <button
                                class="flex-1 py-2.5 rounded-xl text-sm font-medium btn-glass disabled:opacity-50 transition-all"
                                disabled=move || creating.get() || auto_pay_blocked.get()
                                on:click=move |_| on_confirm.run(())
                            >
                                {move || if creating.get() { t("search.creating_task") } else { t("search.create_task") }}
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </Show>
    }.into_any()
}
