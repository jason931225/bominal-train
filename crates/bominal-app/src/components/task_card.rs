use leptos::prelude::*;

use crate::{
    i18n::t,
    types::{TaskInfo, TaskStatus},
    utils::{format_date, format_time, status_variant},
};

use super::{GlassPanel, GlassPanelVariant, Icon, StatusChip};

fn is_active(status: TaskStatus) -> bool {
    matches!(
        status,
        TaskStatus::Queued | TaskStatus::Running | TaskStatus::Idle | TaskStatus::AwaitingPayment
    )
}

fn fmt_datetime(value: &Option<chrono::DateTime<chrono::Utc>>, fallback: &str) -> String {
    value
        .map(|date| {
            let kst = date + chrono::Duration::hours(9);
            kst.format("%Y-%m-%d %H:%M").to_string()
        })
        .unwrap_or_else(|| fallback.to_string())
}

#[component]
pub fn TaskCard(
    task: TaskInfo,
    #[prop(optional)] on_cancel: Option<Callback<String>>,
) -> impl IntoView {
    let (expanded, set_expanded) = signal(false);
    let task_id = task.id.to_string();
    let active = is_active(task.status);

    view! {
        <GlassPanel variant=GlassPanelVariant::Card hover=true class="lg-task-card">
            <div class="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
                <div class="space-y-2">
                    <div class="flex flex-wrap items-center gap-2">
                        <span class="inline-flex items-center rounded-full border border-white/10 px-2.5 py-1 text-[10px] font-semibold uppercase tracking-[0.18em]" style="color: var(--lg-text-tertiary);">
                            {task.provider.to_string()}
                        </span>
                        <StatusChip
                            label=t(task.status.i18n_key())
                            variant=status_variant(task.status)
                        />
                    </div>

                    <h3 class="text-lg font-semibold tracking-tight">
                        {format!("{} -> {}", task.departure_station, task.arrival_station)}
                    </h3>

                    <p class="text-sm" style="color: var(--lg-text-secondary);">
                        {format!(
                            "{} {} • {} {}",
                            format_date(&task.travel_date),
                            format_time(&task.departure_time),
                            task.passengers.total_count(),
                            t("task.passengers_label"),
                        )}
                    </p>

                    <div class="flex flex-wrap items-center gap-3 text-xs" style="color: var(--lg-text-tertiary);">
                        <span>{format!("{} {}", t("task.attempts"), task.attempt_count)}</span>
                        <span>{format!("{} {}", t("task.notify"), if task.notify_enabled { "ON" } else { "OFF" })}</span>
                        <span>{format!("{} {}", t("task.retry"), if task.auto_retry { "ON" } else { "OFF" })}</span>
                    </div>

                    {task.reservation_number.as_ref().map(|reservation_number| {
                        view! {
                            <a href="/reservations" class="text-xs font-mono hover:underline" style="color: var(--lg-success);">
                                {format!("PNR {}", reservation_number)}
                            </a>
                        }
                    })}
                </div>

                <div class="flex flex-wrap items-center gap-2">
                    {if task.status == TaskStatus::AwaitingPayment {
                        Some(view! {
                            <a href="/reservations" class="lg-btn-primary text-xs">
                                {t("task.pay_fare")}
                            </a>
                        })
                    } else {
                        None
                    }}

                    {if active {
                        on_cancel.as_ref().map(|callback| {
                            let callback = callback.clone();
                            view! {
                                <button
                                    type="button"
                                    class="lg-btn-secondary text-xs"
                                    on:click=move |_| callback.run(task_id.clone())
                                >
                                    {t("reservation.cancel")}
                                </button>
                            }
                        })
                    } else {
                        None
                    }}
                </div>
            </div>

            <button
                type="button"
                class="lg-task-card__toggle"
                on:click=move |_| set_expanded.update(|open| *open = !*open)
            >
                <span class=move || {
                    if expanded.get() {
                        "inline-flex rotate-180 transition-transform"
                    } else {
                        "inline-flex transition-transform"
                    }
                }>
                    <Icon name="chevron-down" class="h-4 w-4" />
                </span>
                <span>{move || if expanded.get() { t("task.hide_details") } else { t("task.view_details") }}</span>
            </button>

            <Show when=move || expanded.get()>
                <div class="lg-task-card__details">
                    <div class="grid gap-3 md:grid-cols-2">
                        <div class="lg-task-card__stat">
                            <span>{t("task.started_at")}</span>
                            <strong>{fmt_datetime(&task.started_at, t("task.not_started"))}</strong>
                        </div>
                        <div class="lg-task-card__stat">
                            <span>{t("task.last_attempt")}</span>
                            <strong>{fmt_datetime(&task.last_attempt_at, t("task.no_attempt"))}</strong>
                        </div>
                    </div>

                    <div class="lg-task-card__train-list">
                        <div class="flex items-center justify-between">
                            <span class="text-[10px] font-bold uppercase tracking-wider" style="color: var(--lg-text-tertiary);">
                                {t("task.schedules_title")}
                            </span>
                            <span class="text-xs font-semibold" style="color: var(--lg-text-secondary);">
                                {format!("{} {}", task.target_trains.0.len(), t("task.total"))}
                            </span>
                        </div>
                        {task.target_trains.0.iter().enumerate().map(|(index, train)| {
                            view! {
                                <div class="lg-task-card__train-row">
                                    <span>{format!("#{} {}", train.train_number, format_time(&train.dep_time))}</span>
                                    <strong>{format!("Priority {}", index + 1)}</strong>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>

                    <div class="flex flex-wrap justify-between gap-2 text-xs" style="color: var(--lg-text-secondary);">
                        <span>{format!("{}: {}", t("task.seat_class"), task.seat_preference)}</span>
                        <span>{format!("{}: {}", t("task.passengers_label"), task.passengers.total_count())}</span>
                    </div>
                </div>
            </Show>
        </GlassPanel>
    }
}
