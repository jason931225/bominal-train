//! Task card component — expandable card with action controls for reservation tasks.

use leptos::prelude::*;

use crate::api::tasks::{CancelTask, TaskInfo, UpdateTask};
use crate::components::glass_panel::{GlassPanel, GlassPanelVariant};
use crate::components::status_chip::StatusChip;
use crate::i18n::t;
use crate::utils::{format_date, format_time, status_variant};

/// Parse target_trains JSON array into displayable (train_no, dep_time) pairs.
fn parse_target_trains(trains: &serde_json::Value) -> Vec<(String, String)> {
    trains
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| {
                    let num = v
                        .get("train_no")
                        .or_else(|| v.get("trainNumber"))
                        .and_then(|n| n.as_str())
                        .unwrap_or_default();
                    let time = v
                        .get("dep_time")
                        .or_else(|| v.get("departureTime"))
                        .and_then(|t| t.as_str())
                        .unwrap_or_default();
                    if num.is_empty() {
                        None
                    } else {
                        Some((num.to_string(), time.to_string()))
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Build a new target_trains JSON with one entry moved up.
fn trains_move_up(trains: &serde_json::Value, idx: usize) -> String {
    let mut arr = trains.as_array().cloned().unwrap_or_default();
    if idx > 0 && idx < arr.len() {
        arr.swap(idx, idx - 1);
    }
    serde_json::to_string(&arr).unwrap_or_default()
}

/// Build a new target_trains JSON with one entry moved down.
fn trains_move_down(trains: &serde_json::Value, idx: usize) -> String {
    let mut arr = trains.as_array().cloned().unwrap_or_default();
    if idx + 1 < arr.len() {
        arr.swap(idx, idx + 1);
    }
    serde_json::to_string(&arr).unwrap_or_default()
}

/// Build a new target_trains JSON with one entry removed.
fn trains_remove(trains: &serde_json::Value, idx: usize) -> String {
    let mut arr = trains.as_array().cloned().unwrap_or_default();
    if idx < arr.len() {
        arr.remove(idx);
    }
    serde_json::to_string(&arr).unwrap_or_default()
}

/// Format an optional datetime for display.
fn fmt_datetime(dt: &Option<chrono::DateTime<chrono::Utc>>, fallback: &str) -> String {
    dt.map(|d| {
        let kst = d + chrono::Duration::hours(9);
        kst.format("%H:%M").to_string()
    })
    .unwrap_or_else(|| fallback.to_string())
}

/// Count total passengers from the passengers JSON field.
fn passenger_count(passengers: &serde_json::Value) -> u32 {
    passengers
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|p| p.get("count").and_then(|c| c.as_u64()))
                .sum::<u64>() as u32
        })
        .unwrap_or(1)
}

/// A full-featured task card with expand/collapse, action buttons, and cancel confirmation.
#[component]
pub fn TaskCard(
    /// The task data.
    task: TaskInfo,
    /// Whether this task belongs to the active tab.
    is_active: bool,
) -> impl IntoView {
    let task_id = task.id.to_string();
    let status = task.status.as_str();
    let is_actionable = matches!(status, "queued" | "running" | "idle");
    let is_awaiting_payment = status == "awaiting_payment";
    let is_confirmed = status == "confirmed";
    let show_actions = is_actionable || is_awaiting_payment;

    let update_action = ServerAction::<UpdateTask>::new();
    let cancel_action = ServerAction::<CancelTask>::new();

    let trains = parse_target_trains(&task.target_trains);
    let train_count = trains.len();
    let pax = passenger_count(&task.passengers);
    let cancel_modal_id = format!("cancel-modal-{}", task.id);
    let cancel_btn_id = format!("cancel-btn-{}", task.id);

    view! {
        <GlassPanel variant=GlassPanelVariant::Card>
            <div class="p-4 space-y-3">
                // ── Header ───────────────────────────────────────────
                <div class="flex items-start justify-between">
                    <div class="flex-1">
                        <div class="flex items-center gap-2 mb-1">
                            <span class="text-xs px-1.5 py-0.5 rounded bg-[var(--color-bg-sunken)] text-[var(--color-text-secondary)]">
                                {task.provider.clone()}
                            </span>
                            <StatusChip
                                label=t(&format!("task.{}", task.status))
                                variant=status_variant(&task.status)
                            />
                        </div>
                        // Route visualization
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
                                {format!("PNR: {pnr} \u{2192}")}
                            </a>
                        })}
                        <p class="text-[10px] text-[var(--color-text-disabled)] mt-1">
                            {format!("{}: {}", t("task.attempts"), task.attempt_count)}
                        </p>
                    </div>
                </div>

                // ── Expandable Details ───────────────────────────────
                <details class="group">
                    <summary class="w-full px-3 py-2 text-xs text-[var(--color-text-tertiary)] hover:bg-[var(--color-interactive-hover)] rounded-xl cursor-pointer flex items-center justify-center gap-1 transition-colors list-none [&::-webkit-details-marker]:hidden">
                        <span class="group-open:hidden">{t("task.view_details")}" \u{25BC}"</span>
                        <span class="hidden group-open:inline">{t("task.hide_details")}" \u{25B2}"</span>
                    </summary>
                    <div class="pt-3 space-y-3 border-t border-[var(--color-border-default)] mt-2">
                        {if is_confirmed {
                            // Confirmed: show seat class + passengers
                            view! {
                                <div class="grid grid-cols-2 gap-3">
                                    <div class="flex flex-col bg-[var(--color-bg-sunken)] p-3 rounded-xl">
                                        <span class="text-[10px] font-semibold uppercase text-[var(--color-text-disabled)]">
                                            {t("task.seat_class")}
                                        </span>
                                        <span class="text-sm font-bold text-[var(--color-text-primary)]">
                                            {task.seat_preference.clone()}
                                        </span>
                                    </div>
                                    <div class="flex flex-col bg-[var(--color-bg-sunken)] p-3 rounded-xl">
                                        <span class="text-[10px] font-semibold uppercase text-[var(--color-text-disabled)]">
                                            {t("task.passengers_label")}
                                        </span>
                                        <span class="text-sm font-bold text-[var(--color-text-primary)]">
                                            {pax}
                                        </span>
                                    </div>
                                </div>
                            }.into_any()
                        } else {
                            // Active: show target trains + timing info
                            let task_id_for_trains = task_id.clone();
                            let trains_json = task.target_trains.clone();
                            view! {
                                // Target train list
                                {(train_count > 0).then(|| {
                                    let trains_for_list = trains.clone();
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
                                            {trains_for_list.into_iter().enumerate().map(|(idx, (num, time))| {
                                                let is_primary = idx == 0;
                                                let tid = task_id_for_trains.clone();
                                                let tid2 = task_id_for_trains.clone();
                                                let tid3 = task_id_for_trains.clone();
                                                let up_json = trains_move_up(&trains_json, idx);
                                                let down_json = trains_move_down(&trains_json, idx);
                                                let rm_json = trains_remove(&trains_json, idx);
                                                let display_time = if time.len() >= 4 { format!("{}:{}", &time[..2], &time[2..4]) } else { time.clone() };
                                                view! {
                                                    <div class=format!(
                                                        "rounded-lg border px-3 py-2 flex items-center justify-between {}",
                                                        if is_primary { "border-[var(--color-brand-text)]/30 bg-[var(--color-brand-primary)]/10" }
                                                        else { "border-[var(--color-border-default)]" }
                                                    )>
                                                        <div class="flex items-center gap-2">
                                                            <span class=format!(
                                                                "w-5 h-5 rounded-full flex items-center justify-center text-[10px] font-bold {}",
                                                                if is_primary { "bg-[var(--color-brand-primary)] text-white" }
                                                                else { "bg-[var(--color-bg-elevated)] text-[var(--color-text-tertiary)]" }
                                                            )>{idx + 1}</span>
                                                            <span class="text-xs font-semibold text-[var(--color-text-primary)]">
                                                                {format!("#{num}")}
                                                            </span>
                                                            <span class="text-xs text-[var(--color-text-tertiary)]">
                                                                {display_time}
                                                            </span>
                                                        </div>
                                                        {is_active.then(|| view! {
                                                            <div class="flex items-center gap-0.5">
                                                                // Move up
                                                                {(idx > 0).then(|| {
                                                                    view! {
                                                                        <ActionForm action=update_action>
                                                                            <input type="hidden" name="task_id" value=tid.clone() />
                                                                            <input type="hidden" name="target_trains" value=up_json />
                                                                            <button type="submit" class="p-1 rounded hover:bg-[var(--color-interactive-hover)] text-[var(--color-text-tertiary)] text-xs">
                                                                                "\u{2191}"
                                                                            </button>
                                                                        </ActionForm>
                                                                    }
                                                                })}
                                                                // Move down
                                                                {(idx + 1 < train_count).then(|| {
                                                                    view! {
                                                                        <ActionForm action=update_action>
                                                                            <input type="hidden" name="task_id" value=tid2.clone() />
                                                                            <input type="hidden" name="target_trains" value=down_json />
                                                                            <button type="submit" class="p-1 rounded hover:bg-[var(--color-interactive-hover)] text-[var(--color-text-tertiary)] text-xs">
                                                                                "\u{2193}"
                                                                            </button>
                                                                        </ActionForm>
                                                                    }
                                                                })}
                                                                // Remove
                                                                {(train_count > 1).then(|| {
                                                                    view! {
                                                                        <ActionForm action=update_action>
                                                                            <input type="hidden" name="task_id" value=tid3.clone() />
                                                                            <input type="hidden" name="target_trains" value=rm_json />
                                                                            <button type="submit" class="p-1 rounded hover:bg-[var(--color-status-error)]/20 text-[var(--color-status-error)] text-xs">
                                                                                "\u{00D7}"
                                                                            </button>
                                                                        </ActionForm>
                                                                    }
                                                                })}
                                                            </div>
                                                        })}
                                                    </div>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    }
                                })}
                                // Timing info
                                <div class="grid grid-cols-2 gap-2">
                                    <div class="flex justify-between gap-2 rounded-xl border border-[var(--color-border-default)] bg-[var(--color-bg-sunken)] p-3 text-xs">
                                        <span class="text-[var(--color-text-tertiary)]">{t("task.started_at")}</span>
                                        <span class="text-[var(--color-text-primary)] font-medium">
                                            {fmt_datetime(&task.started_at, &t("task.not_started"))}
                                        </span>
                                    </div>
                                    <div class="flex justify-between gap-2 rounded-xl border border-[var(--color-border-default)] bg-[var(--color-bg-sunken)] p-3 text-xs">
                                        <span class="text-[var(--color-text-tertiary)]">{t("task.last_attempt")}</span>
                                        <span class="text-[var(--color-text-primary)] font-medium">
                                            {fmt_datetime(&task.last_attempt_at, &t("task.no_attempt"))}
                                        </span>
                                    </div>
                                </div>
                                // Seat class + passengers row
                                <div class="flex justify-between text-xs text-[var(--color-text-tertiary)] pt-1">
                                    <span>{format!("{}: {}", t("task.seat_class"), task.seat_preference)}</span>
                                    <span>{format!("{}: {}", t("task.passengers_label"), pax)}</span>
                                </div>
                            }.into_any()
                        }}
                    </div>
                </details>

                // ── Action Footer ────────────────────────────────────
                {show_actions.then(|| {
                    let task_id_notify = task_id.clone();
                    let task_id_pause = task_id.clone();
                    let task_id_retry = task_id.clone();
                    let cancel_btn_id_ref = cancel_btn_id.clone();
                    let _cancel_modal_id_ref = cancel_modal_id.clone();
                    view! {
                        <div class="border-t border-[var(--color-border-default)] pt-3 space-y-2">
                            // Pay Fare button for awaiting_payment
                            {is_awaiting_payment.then(|| view! {
                                <a href="/reservations"
                                   class="w-full flex items-center justify-center gap-2 py-2.5 rounded-xl text-sm font-semibold btn-glass transition-all">
                                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                                        <rect x="1" y="4" width="22" height="16" rx="2" ry="2" />
                                        <line x1="1" y1="10" x2="23" y2="10" />
                                    </svg>
                                    {t("task.pay_fare")}
                                </a>
                            })}

                            // Action button grid
                            <div class="grid grid-cols-4 gap-2">
                                // Notify toggle
                                <ActionForm action=update_action>
                                    <input type="hidden" name="task_id" value=task_id_notify />
                                    <input type="hidden" name="notify_enabled" value=(!task.notify_enabled).to_string() />
                                    <button type="submit" class=format!(
                                        "w-full min-h-[2.5rem] px-2 rounded-xl font-medium text-xs flex items-center justify-center gap-1.5 border transition-colors {}",
                                        if task.notify_enabled {
                                            "bg-[var(--color-brand-primary)]/20 border-[var(--color-brand-text)]/30 text-[var(--color-brand-text)]"
                                        } else {
                                            "bg-[var(--color-bg-elevated)] border-[var(--color-border-default)] text-[var(--color-text-tertiary)] hover:bg-[var(--color-interactive-hover)]"
                                        }
                                    )>
                                        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                                            <path d="M18 8A6 6 0 006 8c0 7-3 9-3 9h18s-3-2-3-9" />
                                            <path d="M13.73 21a2 2 0 01-3.46 0" />
                                        </svg>
                                        {t("task.notify")}
                                    </button>
                                </ActionForm>

                                // Pause / Resume
                                <ActionForm action=update_action>
                                    <input type="hidden" name="task_id" value=task_id_pause />
                                    <input type="hidden" name="status" value={
                                        if task.status == "idle" { "queued" } else { "idle" }
                                    } />
                                    <button type="submit" class=format!(
                                        "w-full min-h-[2.5rem] px-2 rounded-xl font-medium text-xs flex items-center justify-center gap-1.5 border transition-colors {}",
                                        if task.status == "idle" {
                                            "bg-[var(--color-status-success)]/20 border-[var(--color-status-success)]/30 text-[var(--color-status-success)]"
                                        } else {
                                            "bg-[var(--color-bg-elevated)] border-[var(--color-border-default)] text-[var(--color-text-tertiary)] hover:bg-[var(--color-interactive-hover)]"
                                        }
                                    )>
                                        {if task.status == "idle" {
                                            view! {
                                                <svg class="w-3.5 h-3.5" fill="currentColor" viewBox="0 0 24 24">
                                                    <polygon points="5 3 19 12 5 21" />
                                                </svg>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <svg class="w-3.5 h-3.5" fill="currentColor" viewBox="0 0 24 24">
                                                    <rect x="6" y="4" width="4" height="16" />
                                                    <rect x="14" y="4" width="4" height="16" />
                                                </svg>
                                            }.into_any()
                                        }}
                                        {if task.status == "idle" { t("task.resume") } else { t("task.pause") }}
                                    </button>
                                </ActionForm>

                                // Auto-retry toggle
                                <ActionForm action=update_action>
                                    <input type="hidden" name="task_id" value=task_id_retry />
                                    <input type="hidden" name="auto_retry" value=(!task.auto_retry).to_string() />
                                    <button type="submit" class=format!(
                                        "w-full min-h-[2.5rem] px-2 rounded-xl font-medium text-xs flex items-center justify-center gap-1.5 border transition-colors {}",
                                        if task.auto_retry {
                                            "bg-[var(--color-brand-primary)]/20 border-[var(--color-brand-text)]/30 text-[var(--color-brand-text)]"
                                        } else {
                                            "bg-[var(--color-bg-elevated)] border-[var(--color-border-default)] text-[var(--color-text-tertiary)] hover:bg-[var(--color-interactive-hover)]"
                                        }
                                    )>
                                        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                                            <polyline points="1 4 1 10 7 10" />
                                            <path d="M3.51 15a9 9 0 102.13-9.36L1 10" />
                                        </svg>
                                        {t("task.auto_retry")}
                                    </button>
                                </ActionForm>

                                // Cancel (opens confirmation)
                                <button
                                    id=cancel_btn_id_ref
                                    type="button"
                                    class="w-full min-h-[2.5rem] px-2 rounded-xl font-medium text-xs flex items-center justify-center gap-1.5 border border-[var(--color-status-error)]/30 bg-[var(--color-status-error)]/10 text-[var(--color-status-error)] hover:bg-[var(--color-status-error)]/20 transition-colors"
                                >
                                    <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                                        <polyline points="3 6 5 6 21 6" />
                                        <path d="M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2" />
                                    </svg>
                                    {t("task.cancel")}
                                </button>
                            </div>
                        </div>
                    }
                })}

                // ── Cancel Confirmation Modal ────────────────────────
                {show_actions.then(|| {
                    let cancel_task_id = task_id.clone();
                    let modal_id = cancel_modal_id.clone();
                    let btn_id = cancel_btn_id.clone();
                    let dep = task.departure_station.clone();
                    let arr = task.arrival_station.clone();
                    let date_str = format_date(&task.travel_date);
                    view! {
                        <div id=modal_id.clone() class="hidden fixed inset-0 z-[200] flex items-center justify-center p-4">
                            <div class="absolute inset-0 bg-black/40 backdrop-blur-sm fade-in"
                                 data-dismiss=modal_id.clone()></div>
                            <div class="glass-panel rounded-3xl w-full max-w-md relative z-10 p-5 modal-enter">
                                <h3 class="text-lg font-semibold text-[var(--color-text-primary)]">
                                    {t("task.cancel_title")}
                                </h3>
                                <p class="text-sm text-[var(--color-text-tertiary)] mt-1">
                                    {t("task.cancel_description")}
                                </p>
                                // Task preview
                                <div class="mt-4 rounded-xl border border-[var(--color-border-default)] bg-[var(--color-bg-sunken)] p-3">
                                    <div class="flex items-center gap-2 text-sm">
                                        <span class="font-medium text-[var(--color-text-primary)]">{dep}</span>
                                        <span class="text-[var(--color-text-disabled)]">"\u{2192}"</span>
                                        <span class="font-medium text-[var(--color-text-primary)]">{arr}</span>
                                    </div>
                                    <p class="text-xs text-[var(--color-text-tertiary)] mt-1">{date_str}</p>
                                </div>
                                // Actions
                                <div class="mt-4 flex gap-3">
                                    <button
                                        type="button"
                                        data-dismiss=modal_id.clone()
                                        class="flex-1 py-2.5 rounded-xl text-sm font-medium bg-[var(--color-bg-sunken)] text-[var(--color-text-tertiary)] hover:bg-[var(--color-interactive-hover)] transition-colors"
                                    >{t("task.keep")}</button>
                                    <ActionForm action=cancel_action attr:class="flex-1">
                                        <input type="hidden" name="task_id" value=cancel_task_id />
                                        <button
                                            type="submit"
                                            class="w-full py-2.5 rounded-xl text-sm font-medium bg-[var(--color-status-error)] text-white hover:opacity-90 transition-opacity"
                                        >{t("task.cancel_confirm")}</button>
                                    </ActionForm>
                                </div>
                            </div>
                        </div>
                        // Inline script for modal toggle
                        <script>{format!(r#"
(function(){{
  var btn=document.getElementById('{btn_id}');
  var modal=document.getElementById('{modal_id}');
  if(!btn||!modal) return;
  btn.addEventListener('click',function(e){{e.stopPropagation();modal.classList.remove('hidden');}});
  modal.querySelectorAll('[data-dismiss]').forEach(function(el){{
    el.addEventListener('click',function(){{modal.classList.add('hidden');}});
  }});
}})();
"#)}</script>
                    }
                })}
            </div>
        </GlassPanel>
    }
}
