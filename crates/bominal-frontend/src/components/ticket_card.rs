//! Ticket card component — expandable train schedule card.

use leptos::prelude::*;

/// Seat availability level.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SeatAvailability {
    Available,
    Limited,
    SoldOut,
}

/// A collapsible train ticket card showing route, time, status, and availability.
#[component]
pub fn TicketCard(
    /// Train number (e.g., "KTX 305").
    #[prop(into)]
    train_number: String,
    /// Departure station.
    #[prop(into)]
    departure: String,
    /// Arrival station.
    #[prop(into)]
    arrival: String,
    /// Departure time.
    #[prop(into)]
    dep_time: String,
    /// Arrival time.
    #[prop(into)]
    arr_time: String,
    /// Price display string.
    #[prop(into)]
    price: String,
    /// Economy seat availability.
    #[prop(default = SeatAvailability::SoldOut)]
    economy: SeatAvailability,
    /// Premium seat availability.
    #[prop(default = SeatAvailability::SoldOut)]
    premium: SeatAvailability,
    /// Whether this card is selected.
    #[prop(default = false)]
    selected: bool,
    /// Optional click handler.
    #[prop(optional)]
    on_click: Option<Callback<()>>,
    /// Status label.
    #[prop(into, optional)]
    status: Option<String>,
    /// Status variant for StatusChip.
    #[prop(into, optional)]
    status_variant: Option<String>,
) -> impl IntoView {
    let (expanded, set_expanded) = signal(false);

    let card_class = if selected {
        "glass-card glass-active rounded-3xl overflow-hidden cursor-pointer transition-all duration-200"
    } else {
        "glass-card glass-card-hover rounded-3xl overflow-hidden cursor-pointer transition-all duration-200"
    };

    let dot_class = |avail: SeatAvailability| match avail {
        SeatAvailability::Available => "availability-dot availability-dot-economy",
        SeatAvailability::Limited => "availability-dot availability-dot-premium",
        SeatAvailability::SoldOut => "availability-dot availability-dot-muted",
    };

    let economy_dot = dot_class(economy);
    let premium_dot = dot_class(premium);

    view! {
        <div class=card_class
             on:click=move |_| {
                 if let Some(cb) = &on_click {
                     cb.run(());
                 }
             }>
            // Collapsed header
            <div class="p-4">
                <div class="flex items-center justify-between mb-2">
                    <div class="flex items-center gap-2">
                        <span class="text-sm font-semibold text-[var(--theme-text-strong)]">
                            {train_number.clone()}
                        </span>
                        {status.clone().map(|s| {
                            let variant = status_variant.clone().unwrap_or_else(|| "neutral".to_string());
                            view! {
                                <span class=format!("inline-flex items-center px-2 py-0.5 rounded-full text-[10px] font-medium status-{}", variant)>
                                    {s}
                                </span>
                            }
                        })}
                    </div>
                    <div class="flex items-center gap-1.5">
                        <div class=economy_dot title="Economy"></div>
                        <div class=premium_dot title="Premium"></div>
                    </div>
                </div>
                <div class="flex items-center gap-2 text-sm">
                    <span class="font-medium text-[var(--theme-text-primary)]">{dep_time.clone()}</span>
                    <span class="text-[var(--theme-text-subtle)]">{departure.clone()}</span>
                    <span class="text-[var(--theme-text-subtle)]">{"\u{2192}"}</span>
                    <span class="font-medium text-[var(--theme-text-primary)]">{arr_time.clone()}</span>
                    <span class="text-[var(--theme-text-subtle)]">{arrival.clone()}</span>
                </div>
            </div>
            // Expand toggle
            <button class="w-full px-4 py-2 text-xs text-[var(--theme-text-muted)] hover:bg-[var(--theme-surface-hover)] border-t border-[var(--theme-border-subtle)] flex items-center justify-center gap-1"
                    on:click=move |e| {
                        e.stop_propagation();
                        set_expanded.update(|v| *v = !*v);
                    }>
                {move || if expanded.get() { "Hide details \u{2191}" } else { "Show details \u{2193}" }}
            </button>
            // Expanded details
            <div class="overflow-hidden transition-all duration-300"
                 style=move || if expanded.get() { "max-height: 200px; opacity: 1;" } else { "max-height: 0; opacity: 0;" }>
                <div class="px-4 pb-4 space-y-2 border-t border-[var(--theme-border-subtle)]">
                    <div class="flex justify-between pt-3">
                        <span class="text-xs text-[var(--theme-text-muted)]">"Price"</span>
                        <span class="text-sm font-semibold text-[var(--theme-accent-text)]">{price.clone()}</span>
                    </div>
                    <div class="flex justify-between">
                        <span class="text-xs text-[var(--theme-text-muted)]">"Economy"</span>
                        <span class=move || format!("text-xs font-medium {}", match economy {
                            SeatAvailability::Available => "text-[var(--theme-positive-text)]",
                            SeatAvailability::Limited => "text-[var(--theme-warning-text)]",
                            SeatAvailability::SoldOut => "text-[var(--theme-text-subtle)]",
                        })>{match economy {
                            SeatAvailability::Available => "Available",
                            SeatAvailability::Limited => "Limited",
                            SeatAvailability::SoldOut => "Sold Out",
                        }}</span>
                    </div>
                    <div class="flex justify-between">
                        <span class="text-xs text-[var(--theme-text-muted)]">"Premium"</span>
                        <span class=move || format!("text-xs font-medium {}", match premium {
                            SeatAvailability::Available => "text-[var(--theme-positive-text)]",
                            SeatAvailability::Limited => "text-[var(--theme-warning-text)]",
                            SeatAvailability::SoldOut => "text-[var(--theme-text-subtle)]",
                        })>{match premium {
                            SeatAvailability::Available => "Available",
                            SeatAvailability::Limited => "Limited",
                            SeatAvailability::SoldOut => "Sold Out",
                        }}</span>
                    </div>
                </div>
            </div>
        </div>
    }
}
