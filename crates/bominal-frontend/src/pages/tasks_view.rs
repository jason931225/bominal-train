//! Tasks view — lists active and completed reservation tasks.

use leptos::prelude::*;

use crate::api::tasks::{cancel_task, list_tasks, TaskInfo};
use crate::components::glass_panel::GlassPanel;
use crate::components::sse_reload::SseReload;
use crate::components::status_chip::StatusChip;
use crate::i18n::t;

/// Tasks dashboard showing active and completed reservation tasks.
#[component]
pub fn TasksView() -> impl IntoView {
    let (active_tab, set_active_tab) = signal(true);
    let tasks = Resource::new(|| (), |_| list_tasks());
    let cancel_action = Action::new(|task_id: &String| {
        let id = task_id.clone();
        async move { cancel_task(id).await }
    });

    // Refetch tasks after cancellation
    Effect::new(move || {
        if let Some(Ok(())) = cancel_action.value().get() {
            tasks.refetch();
        }
    });

    view! {
        <SseReload />
        <div class="px-4 pt-6 pb-4 space-y-4">
            <h1 class="text-xl font-bold text-[var(--color-text-primary)]">{t("nav.tasks")}</h1>

            // Tab toggle
            <div class="flex bg-[var(--color-bg-sunken)] rounded-xl p-1">
                <button
                    class=move || if active_tab.get() {
                        "flex-1 py-2 text-sm font-medium rounded-lg bg-[var(--color-bg-elevated)] text-[var(--color-text-primary)] shadow-sm transition-all"
                    } else {
                        "flex-1 py-2 text-sm font-medium rounded-lg text-[var(--color-text-tertiary)] transition-all"
                    }
                    on:click=move |_| set_active_tab.set(true)
                >
                    {t("task.active")}
                </button>
                <button
                    class=move || if !active_tab.get() {
                        "flex-1 py-2 text-sm font-medium rounded-lg bg-[var(--color-bg-elevated)] text-[var(--color-text-primary)] shadow-sm transition-all"
                    } else {
                        "flex-1 py-2 text-sm font-medium rounded-lg text-[var(--color-text-tertiary)] transition-all"
                    }
                    on:click=move |_| set_active_tab.set(false)
                >
                    {t("task.completed")}
                </button>
            </div>

            // Task list
            <Suspense fallback=move || view! {
                <GlassPanel>
                    <div class="p-4 text-center py-8">
                        <p class="text-[var(--color-text-tertiary)] text-sm">{t("common.loading")}</p>
                    </div>
                </GlassPanel>
            }>
                {move || tasks.get().map(|result| match result {
                    Ok(all_tasks) => {
                        let is_active_tab = active_tab.get();
                        let display_tasks: Vec<TaskInfo> = all_tasks
                            .into_iter()
                            .filter(|t| {
                                let task_active = matches!(t.status.as_str(), "queued" | "running" | "idle");
                                if is_active_tab { task_active } else { !task_active }
                            })
                            .collect();

                        if display_tasks.is_empty() {
                            view! {
                                <GlassPanel>
                                    <div class="p-4 text-center py-12">
                                        <p class="text-[var(--color-text-tertiary)] text-sm">
                                            {if is_active_tab { t("task.no_active") } else { t("task.no_completed") }}
                                        </p>
                                        <a href="/search" class="inline-block mt-3 text-sm text-[var(--color-brand-primary)] font-medium hover:underline">
                                            {t("task.create_new")}
                                        </a>
                                    </div>
                                </GlassPanel>
                            }.into_any()
                        } else {
                            view! {
                                <div class="space-y-3">
                                    {display_tasks.into_iter().map(|task| {
                                        let task_id = task.id.to_string();
                                        let is_active = matches!(task.status.as_str(), "queued" | "running" | "idle");
                                        view! {
                                            <GlassPanel>
                                                <div class="p-4">
                                                    <div class="flex items-start justify-between">
                                                        <div class="flex-1">
                                                            <div class="flex items-center gap-2 mb-1">
                                                                <span class="text-xs px-1.5 py-0.5 rounded bg-[var(--color-bg-sunken)] text-[var(--color-text-secondary)]">
                                                                    {task.provider.clone()}
                                                                </span>
                                                                <StatusChip label=task.status.clone() variant=status_variant(&task.status) />
                                                            </div>
                                                            <p class="text-sm font-medium text-[var(--color-text-primary)]">
                                                                {format!("{} → {}", task.departure_station, task.arrival_station)}
                                                            </p>
                                                            <p class="text-xs text-[var(--color-text-tertiary)] mt-0.5">
                                                                {format!("{} {}", task.travel_date, format_time(&task.departure_time))}
                                                            </p>
                                                            {task.reservation_number.as_ref().map(|pnr| view! {
                                                                <a href="/reservations" class="text-xs text-[var(--color-status-success)] mt-1 font-mono hover:underline inline-block">
                                                                    {format!("PNR: {pnr} →")}
                                                                </a>
                                                            })}
                                                            <p class="text-[10px] text-[var(--color-text-disabled)] mt-1">
                                                                {format!("{}: {}", t("task.attempts"), task.attempt_count)}
                                                            </p>
                                                        </div>
                                                        {is_active.then(|| {
                                                            let id = task_id.clone();
                                                            view! {
                                                                <button
                                                                    class="text-xs px-2 py-1 rounded-lg text-[var(--color-status-error)] hover:bg-[var(--color-status-error)]/10 transition-colors"
                                                                    on:click=move |_| { cancel_action.dispatch(id.clone()); }
                                                                >
                                                                    {t("task.cancel")}
                                                                </button>
                                                            }
                                                        })}
                                                    </div>
                                                </div>
                                            </GlassPanel>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_any()
                        }
                    }
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

fn status_variant(status: &str) -> String {
    match status {
        "queued" => "idle",
        "running" => "running",
        "confirmed" => "success",
        "failed" | "error" => "error",
        "cancelled" => "warning",
        _ => "info",
    }
    .to_string()
}

fn format_time(raw: &str) -> String {
    if raw.len() >= 4 {
        format!("{}:{}", &raw[..2], &raw[2..4])
    } else {
        raw.to_string()
    }
}
