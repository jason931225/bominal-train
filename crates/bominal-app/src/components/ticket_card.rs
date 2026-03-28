use leptos::prelude::*;

use super::{GlassPanel, GlassPanelVariant, StatusChip};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeatAvailability {
    Available,
    Limited,
    SoldOut,
}

impl SeatAvailability {
    fn label(self) -> &'static str {
        match self {
            Self::Available => "Available",
            Self::Limited => "Limited",
            Self::SoldOut => "Sold Out",
        }
    }

    fn variant(self) -> &'static str {
        match self {
            Self::Available => "success",
            Self::Limited => "warning",
            Self::SoldOut => "neutral",
        }
    }
}

#[component]
pub fn TicketCard(
    #[prop(into)] train_number: String,
    #[prop(into)] departure: String,
    #[prop(into)] arrival: String,
    #[prop(into)] dep_time: String,
    #[prop(into)] arr_time: String,
    #[prop(into, optional)] detail_label: Option<String>,
    #[prop(into, optional)] detail_value: Option<String>,
    #[prop(default = SeatAvailability::SoldOut)] economy: SeatAvailability,
    #[prop(default = SeatAvailability::SoldOut)] premium: SeatAvailability,
    #[prop(default = false)] selected: bool,
    #[prop(optional)] on_click: Option<Callback<()>>,
    #[prop(optional)] status: Option<String>,
    #[prop(optional)] status_variant: Option<String>,
) -> impl IntoView {
    let (expanded, set_expanded) = signal(false);
    let panel_variant = if selected {
        GlassPanelVariant::Active
    } else {
        GlassPanelVariant::Card
    };

    view! {
        <GlassPanel variant=panel_variant hover=!selected class="lg-ticket-card">
            <button
                type="button"
                class="lg-ticket-card__surface"
                on:click=move |_| {
                    if let Some(callback) = &on_click {
                        callback.run(());
                    }
                }
            >
                <div class="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
                    <div class="space-y-2 text-left">
                        <div class="flex flex-wrap items-center gap-2">
                            <span class="text-sm font-semibold">{train_number.clone()}</span>
                            {status.as_ref().map(|status| {
                                view! {
                                    <StatusChip
                                        label=status.clone()
                                        variant=status_variant.clone().unwrap_or_else(|| "neutral".to_string())
                                    />
                                }
                            })}
                        </div>

                        <p class="text-base font-medium tracking-tight">
                            {format!("{dep_time} {departure} -> {arr_time} {arrival}")}
                        </p>
                    </div>

                    <div class="flex flex-wrap items-center gap-2 lg:justify-end">
                        <StatusChip label="General" variant=economy.variant() />
                        <StatusChip label="Special" variant=premium.variant() />
                    </div>
                </div>
            </button>

            <button
                type="button"
                class="lg-ticket-card__toggle"
                on:click=move |_| set_expanded.update(|open| *open = !*open)
            >
                {move || if expanded.get() { "Hide details" } else { "Show details" }}
            </button>

            <Show when=move || expanded.get()>
                <div class="lg-ticket-card__details">
                    {detail_label
                        .as_ref()
                        .zip(detail_value.as_ref())
                        .map(|(label, value)| {
                            view! {
                                <div class="lg-ticket-card__detail-row">
                                    <span>{label.clone()}</span>
                                    <strong>{value.clone()}</strong>
                                </div>
                            }
                        })}

                    <div class="lg-ticket-card__detail-row">
                        <span>"General"</span>
                        <strong>{economy.label()}</strong>
                    </div>
                    <div class="lg-ticket-card__detail-row">
                        <span>"Special"</span>
                        <strong>{premium.label()}</strong>
                    </div>
                </div>
            </Show>
        </GlassPanel>
    }
}
