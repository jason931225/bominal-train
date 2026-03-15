//! Task card component — typed, reactive task controls without inline scripts.

use leptos::prelude::*;

use crate::api::tasks::{
    CancelTask, TargetTrainList, TaskInfo, TaskStatus, UpdateTask, UpdateTaskInput,
};
use crate::components::glass_panel::{GlassPanel, GlassPanelVariant};
use crate::components::status_chip::StatusChip;
use crate::i18n::t;
use crate::utils::{format_date, format_time, status_variant};

fn move_up(trains: &TargetTrainList, idx: usize) -> TargetTrainList {
    let mut next = trains.clone();
    if idx > 0 && idx < next.0.len() {
        next.0.swap(idx, idx - 1);
    }
    next
}

fn move_down(trains: &TargetTrainList, idx: usize) -> TargetTrainList {
    let mut next = trains.clone();
    if idx + 1 < next.0.len() {
        next.0.swap(idx, idx + 1);
    }
    next
}

fn remove_train(trains: &TargetTrainList, idx: usize) -> TargetTrainList {
    let mut next = trains.clone();
    if idx < next.0.len() {
        next.0.remove(idx);
    }
    next
}

fn fmt_datetime(dt: &Option<chrono::DateTime<chrono::Utc>>, fallback: &str) -> String {
    dt.map(|d| {
        let kst = d + chrono::Duration::hours(9);
        kst.format("%H:%M").to_string()
    })
    .unwrap_or_else(|| fallback.to_string())
}

#[component]
pub fn TaskCard(task: TaskInfo, is_active: bool) -> impl IntoView {
    let update_action = ServerAction::<UpdateTask>::new();
    let cancel_action = ServerAction::<CancelTask>::new();
    let (confirm_cancel_open, set_confirm_cancel_open) = signal(false);

    let trains = task.target_trains.clone();
    let train_count = trains.0.len();
    let pax = task.passengers.total_count();
    let status = task.status;
    let is_actionable = matches!(status, TaskStatus::Queued | TaskStatus::Running | TaskStatus::Idle);
    let is_awaiting_payment = status == TaskStatus::AwaitingPayment;
    let is_confirmed = status == TaskStatus::Confirmed;
    let show_actions = is_actionable || is_awaiting_payment;
    let task_id = task.id.to_string();

    let toggle_status = if status == TaskStatus::Idle {
        TaskStatus::Queued
    } else {
        TaskStatus::Idle
    };
    let task_id_notify = task_id.clone();
    let task_id_status = task_id.clone();
    let task_id_retry = task_id.clone();
    let update_action_notify = update_action;
    let update_action_status = update_action;
    let update_action_retry = update_action;

    view! {
        <GlassPanel variant=GlassPanelVariant::Card>
            <div class="p-4 space-y-3">
                <div class="flex items-start justify-between">
                    <div class="flex-1">
                        <div class="flex items-center gap-2 mb-1">
                            <span class="text-xs px-1.5 py-0.5 rounded bg-[var(--color-bg-sunken)] text-[var(--color-text-secondary)]">
                                {task.provider.to_string()}
                            </span>
                            <StatusChip
                                label=t(task.status.i18n_key())
                                variant=status_variant(task.status)
                            />
                        </div>
                        <div class="flex items-center gap-2 mt-1">
                            <div class="flex items-center gap-1.5">
                                <span class="w-2 h-2 rounded-full bg-[var(--color-brand-text)]"></span>
                                <span class="text-sm font-medium text-[var(--color-text-primary)]">
                                    {task.departure_station.clone()}
                                </span>
                            </div>
                            <div class="flex-1 h-px bg-[var(--color-border-default)]"></div>
                            <div class="flex items-center gap-1.5">
                                <span class="text-sm font-medium text-[var(--color-text-primary)]">
                                    {task.arrival_station.clone()}
                                </span>
                                <span class="w-2 h-2 rounded-full bg-[var(--color-status-success)]"></span>
                            </div>
                        </div>
                        <p class="text-xs text-[var(--color-text-tertiary)] mt-1">
                            {format!("{} {}", format_date(&task.travel_date), format_time(&task.departure_time))}
                        </p>
                        {task.reservation_number.as_ref().map(|pnr| view! {
                            <a href="/reservations" class="text-xs text-[var(--color-status-success)] mt-1 font-mono hover:underline inline-block">
                                {format!("PNR: {pnr} ->")}
                            </a>
                        })}
                        <p class="text-[10px] text-[var(--color-text-disabled)] mt-1">
                            {format!("{}: {}", t("task.attempts"), task.attempt_count)}
                        </p>
                    </div>
                </div>

                <details class="group">
                    <summary class="w-full px-3 py-2 text-xs text-[var(--color-text-tertiary)] hover:bg-[var(--color-interactive-hover)] rounded-xl cursor-pointer flex items-center justify-center gap-1 transition-colors list-none [&::-webkit-details-marker]:hidden">
                        <span class="group-open:hidden">{format!("{} ▼", t("task.view_details"))}</span>
                        <span class="hidden group-open:inline">{format!("{} ▲", t("task.hide_details"))}</span>
                    </summary>
                    <div class="pt-3 space-y-3 border-t border-[var(--color-border-default)] mt-2">
                        {if is_confirmed {
                            view! {
                                <div class="grid grid-cols-2 gap-3">
                                    <DetailStat label=t("task.seat_class").to_string() value=task.seat_preference.to_string() />
                                    <DetailStat label=t("task.passengers_label").to_string() value=pax.to_string() />
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="rounded-xl border border-[var(--color-border-default)] bg-[var(--color-bg-sunken)] p-3 space-y-2">
                                    <div class="flex items-center justify-between">
                                        <span class="text-[10px] font-bold uppercase tracking-wider text-[var(--color-text-disabled)]">
                                            {t("task.schedules_title")}
                                        </span>
                                        <span class="text-xs font-semibold text-[var(--color-text-tertiary)]">
                                            {format!("{} {}", train_count, t("task.total"))}
                                        </span>
                                    </div>
                                    {trains.0.into_iter().enumerate().map(|(idx, train)| {
                                        let display_time = format_time(&train.dep_time);
                                        let move_up_value = move_up(&task.target_trains, idx);
                                        let move_down_value = move_down(&task.target_trains, idx);
                                        let remove_value = remove_train(&task.target_trains, idx);
                                        let task_id_up = task_id.clone();
                                        let task_id_down = task_id.clone();
                                        let task_id_remove = task_id.clone();
                                        let update_action_up = update_action;
                                        let update_action_down = update_action;
                                        let update_action_remove = update_action;
                                        view! {
                                            <div class="rounded-lg border border-[var(--color-border-default)] px-3 py-2 flex items-center justify-between">
                                                <div class="flex items-center gap-2">
                                                    <span class="w-5 h-5 rounded-full flex items-center justify-center text-[10px] font-bold bg-[var(--color-bg-elevated)] text-[var(--color-text-tertiary)]">
                                                        {idx + 1}
                                                    </span>
                                                    <span class="text-xs font-semibold text-[var(--color-text-primary)]">
                                                        {format!("#{}", train.train_number)}
                                                    </span>
                                                    <span class="text-xs text-[var(--color-text-tertiary)]">
                                                        {display_time}
                                                    </span>
                                                </div>
                                                {is_active.then(|| view! {
                                                    <div class="flex items-center gap-0.5">
                                                        <button
                                                            type="button"
                                                            class="p-1 rounded hover:bg-[var(--color-interactive-hover)] text-[var(--color-text-tertiary)] text-xs"
                                                            disabled=idx == 0
                                                            on:click=move |_| {
                                                                update_action_up.dispatch(UpdateTask {
                                                                    task_id: task_id_up.clone(),
                                                                    input: UpdateTaskInput {
                                                                        target_trains: Some(move_up_value.clone()),
                                                                        ..Default::default()
                                                                    },
                                                                });
                                                            }
                                                        >
                                                            "↑"
                                                        </button>
                                                        <button
                                                            type="button"
                                                            class="p-1 rounded hover:bg-[var(--color-interactive-hover)] text-[var(--color-text-tertiary)] text-xs"
                                                            disabled=idx + 1 >= train_count
                                                            on:click=move |_| {
                                                                update_action_down.dispatch(UpdateTask {
                                                                    task_id: task_id_down.clone(),
                                                                    input: UpdateTaskInput {
                                                                        target_trains: Some(move_down_value.clone()),
                                                                        ..Default::default()
                                                                    },
                                                                });
                                                            }
                                                        >
                                                            "↓"
                                                        </button>
                                                        <button
                                                            type="button"
                                                            class="p-1 rounded hover:bg-[var(--color-status-error)]/20 text-[var(--color-status-error)] text-xs"
                                                            disabled=train_count <= 1
                                                            on:click=move |_| {
                                                                update_action_remove.dispatch(UpdateTask {
                                                                    task_id: task_id_remove.clone(),
                                                                    input: UpdateTaskInput {
                                                                        target_trains: Some(remove_value.clone()),
                                                                        ..Default::default()
                                                                    },
                                                                });
                                                            }
                                                        >
                                                            "×"
                                                        </button>
                                                    </div>
                                                })}
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                                <div class="grid grid-cols-2 gap-2">
                                    <TimingStat
                                        label=t("task.started_at").to_string()
                                        value=fmt_datetime(&task.started_at, t("task.not_started"))
                                    />
                                    <TimingStat
                                        label=t("task.last_attempt").to_string()
                                        value=fmt_datetime(&task.last_attempt_at, t("task.no_attempt"))
                                    />
                                </div>
                                <div class="flex justify-between text-xs text-[var(--color-text-tertiary)] pt-1">
                                    <span>{format!("{}: {}", t("task.seat_class"), task.seat_preference)}</span>
                                    <span>{format!("{}: {}", t("task.passengers_label"), pax)}</span>
                                </div>
                            }.into_any()
                        }}
                    </div>
                </details>

                {show_actions.then(|| view! {
                    <div class="border-t border-[var(--color-border-default)] pt-3 space-y-2">
                        {is_awaiting_payment.then(|| view! {
                            <a href="/reservations"
                               class="w-full flex items-center justify-center gap-2 py-2.5 rounded-xl text-sm font-semibold btn-glass transition-all">
                                {t("task.pay_fare")}
                            </a>
                        })}

                        <div class="grid grid-cols-4 gap-2">
                            <button
                                type="button"
                                class=move || toggle_button_class(task.notify_enabled)
                                on:click=move |_| {
                                    update_action_notify.dispatch(UpdateTask {
                                        task_id: task_id_notify.clone(),
                                        input: UpdateTaskInput {
                                            notify_enabled: Some(!task.notify_enabled),
                                            ..Default::default()
                                        },
                                    });
                                }
                            >
                                {t("task.notify")}
                            </button>
                            <button
                                type="button"
                                class=move || toggle_button_class(task.status == TaskStatus::Idle)
                                on:click=move |_| {
                                    update_action_status.dispatch(UpdateTask {
                                        task_id: task_id_status.clone(),
                                        input: UpdateTaskInput {
                                            status: Some(toggle_status),
                                            ..Default::default()
                                        },
                                    });
                                }
                            >
                                {if task.status == TaskStatus::Idle { t("task.resume") } else { t("task.pause") }}
                            </button>
                            <button
                                type="button"
                                class=move || toggle_button_class(task.auto_retry)
                                on:click=move |_| {
                                    update_action_retry.dispatch(UpdateTask {
                                        task_id: task_id_retry.clone(),
                                        input: UpdateTaskInput {
                                            auto_retry: Some(!task.auto_retry),
                                            ..Default::default()
                                        },
                                    });
                                }
                            >
                                {t("task.auto_retry")}
                            </button>
                            <button
                                type="button"
                                class="w-full min-h-[2.5rem] px-2 rounded-xl font-medium text-xs border border-[var(--color-status-error)]/30 bg-[var(--color-status-error)]/10 text-[var(--color-status-error)] hover:bg-[var(--color-status-error)]/20 transition-colors"
                                on:click=move |_| set_confirm_cancel_open.set(true)
                            >
                                {t("task.cancel")}
                            </button>
                        </div>
                    </div>
                })}

                <Show when=move || confirm_cancel_open.get()>
                    <div class="fixed inset-0 z-[200] flex items-center justify-center p-4">
                        <button
                            type="button"
                            class="absolute inset-0 bg-black/40 backdrop-blur-sm fade-in"
                            on:click=move |_| set_confirm_cancel_open.set(false)
                        ></button>
                        <div class="glass-panel rounded-3xl w-full max-w-md relative z-10 p-5 modal-enter">
                            <h3 class="text-lg font-semibold text-[var(--color-text-primary)]">
                                {t("task.cancel_title")}
                            </h3>
                            <p class="text-sm text-[var(--color-text-tertiary)] mt-1">
                                {t("task.cancel_description")}
                            </p>
                            <div class="mt-4 rounded-xl border border-[var(--color-border-default)] bg-[var(--color-bg-sunken)] p-3">
                                <div class="flex items-center gap-2 text-sm">
                                    <span class="font-medium text-[var(--color-text-primary)]">{task.departure_station.clone()}</span>
                                    <span class="text-[var(--color-text-disabled)]">"->"</span>
                                    <span class="font-medium text-[var(--color-text-primary)]">{task.arrival_station.clone()}</span>
                                </div>
                                <p class="text-xs text-[var(--color-text-tertiary)] mt-1">{format_date(&task.travel_date)}</p>
                            </div>
                            <div class="mt-4 flex gap-3">
                                <button
                                    type="button"
                                    class="flex-1 py-2.5 rounded-xl text-sm font-medium bg-[var(--color-bg-sunken)] text-[var(--color-text-tertiary)] hover:bg-[var(--color-interactive-hover)] transition-colors"
                                    on:click=move |_| set_confirm_cancel_open.set(false)
                                >
                                    {t("task.keep")}
                                </button>
                                <button
                                    type="button"
                                    class="flex-1 py-2.5 rounded-xl text-sm font-medium bg-[var(--color-status-error)] text-white hover:opacity-90 transition-opacity"
                                    on:click=move |_| {
                                        cancel_action.dispatch(CancelTask { task_id: task.id.to_string() });
                                        set_confirm_cancel_open.set(false);
                                    }
                                >
                                    {t("task.cancel_confirm")}
                                </button>
                            </div>
                        </div>
                    </div>
                </Show>
            </div>
        </GlassPanel>
    }
}

#[component]
fn DetailStat(label: String, value: String) -> impl IntoView {
    view! {
        <div class="flex flex-col bg-[var(--color-bg-sunken)] p-3 rounded-xl">
            <span class="text-[10px] font-semibold uppercase text-[var(--color-text-disabled)]">
                {label}
            </span>
            <span class="text-sm font-bold text-[var(--color-text-primary)]">
                {value}
            </span>
        </div>
    }
}

#[component]
fn TimingStat(label: String, value: String) -> impl IntoView {
    view! {
        <div class="flex justify-between gap-2 rounded-xl border border-[var(--color-border-default)] bg-[var(--color-bg-sunken)] p-3 text-xs">
            <span class="text-[var(--color-text-tertiary)]">{label}</span>
            <span class="text-[var(--color-text-primary)] font-medium">{value}</span>
        </div>
    }
}

fn toggle_button_class(active: bool) -> &'static str {
    if active {
        "w-full min-h-[2.5rem] px-2 rounded-xl font-medium text-xs border bg-[var(--color-brand-primary)]/20 border-[var(--color-brand-text)]/30 text-[var(--color-brand-text)]"
    } else {
        "w-full min-h-[2.5rem] px-2 rounded-xl font-medium text-xs border bg-[var(--color-bg-elevated)] border-[var(--color-border-default)] text-[var(--color-text-tertiary)] hover:bg-[var(--color-interactive-hover)]"
    }
}
