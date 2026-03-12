//! Home page — dashboard with active task summary and quick actions.

use leptos::prelude::*;

use crate::api::tasks::{list_tasks, TaskInfo};
use crate::components::glass_panel::GlassPanel;
use crate::components::sse_reload::SseReload;
use crate::components::status_chip::StatusChip;
use crate::i18n::t;

/// Home dashboard showing active tasks summary and quick actions.
#[component]
pub fn HomeView() -> impl IntoView {
    let tasks = Resource::new(|| (), |_| list_tasks());

    view! {
        <SseReload />
        <div class="px-4 pt-6 pb-4 space-y-4">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-xl font-bold text-[var(--color-text-primary)]">{t("home.welcome")}</h1>
                    <p class="text-sm text-[var(--color-text-secondary)]">{t("home.active_tasks")}</p>
                </div>
            </div>

            // Active tasks summary
            <GlassPanel>
                <div class="p-4">
                    <h2 class="text-sm font-semibold text-[var(--color-text-secondary)] uppercase tracking-wider mb-3">{t("home.active_tasks")}</h2>
                    <Suspense fallback=move || view! {
                        <div class="text-center py-8">
                            <p class="text-[var(--color-text-tertiary)] text-sm">{t("common.loading")}</p>
                        </div>
                    }>
                        {move || tasks.get().map(|result| match result {
                            Ok(all_tasks) => {
                                let active: Vec<TaskInfo> = all_tasks
                                    .into_iter()
                                    .filter(|t| matches!(t.status.as_str(), "queued" | "running" | "idle"))
                                    .collect();

                                if active.is_empty() {
                                    view! {
                                        <div class="text-center py-8">
                                            <p class="text-[var(--color-text-tertiary)] text-sm">{t("home.no_active_tasks")}</p>
                                            <a href="/search" class="inline-block mt-3 text-sm text-[var(--color-brand-primary)] font-medium hover:underline">
                                                {t("search.go_to_search")}
                                            </a>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div class="space-y-3">
                                            {active.into_iter().map(|task| view! {
                                                <div class="flex items-center justify-between py-2 border-b border-[var(--color-border-subtle)] last:border-0">
                                                    <div>
                                                        <p class="text-sm font-medium text-[var(--color-text-primary)]">
                                                            {format!("{} → {}", task.departure_station, task.arrival_station)}
                                                        </p>
                                                        <p class="text-xs text-[var(--color-text-tertiary)]">
                                                            {format!("{} {} · {}", task.provider, task.travel_date, task.departure_time)}
                                                        </p>
                                                    </div>
                                                    <StatusChip label=task.status.clone() variant=status_variant(&task.status) />
                                                </div>
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    }.into_any()
                                }
                            }
                            Err(_) => view! {
                                <div class="text-center py-8">
                                    <p class="text-[var(--color-text-tertiary)] text-sm">{t("error.load_failed")}</p>
                                </div>
                            }.into_any(),
                        })}
                    </Suspense>
                </div>
            </GlassPanel>

            // Quick actions
            <GlassPanel>
                <div class="p-4">
                    <h2 class="text-sm font-semibold text-[var(--color-text-secondary)] uppercase tracking-wider mb-3">{t("home.quick_actions")}</h2>
                    <div class="grid grid-cols-3 gap-3">
                        <a href="/search" class="flex flex-col items-center gap-2 p-4 rounded-xl bg-[var(--color-bg-sunken)] hover:bg-[var(--color-interactive-hover)] transition-colors">
                            <svg class="w-6 h-6 text-[var(--color-brand-primary)]" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                            </svg>
                            <span class="text-xs font-medium text-[var(--color-text-secondary)]">{t("nav.search")}</span>
                        </a>
                        <a href="/tasks" class="flex flex-col items-center gap-2 p-4 rounded-xl bg-[var(--color-bg-sunken)] hover:bg-[var(--color-interactive-hover)] transition-colors">
                            <svg class="w-6 h-6 text-[var(--color-brand-primary)]" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-6 9l2 2 4-4" />
                            </svg>
                            <span class="text-xs font-medium text-[var(--color-text-secondary)]">{t("nav.tasks")}</span>
                        </a>
                        <a href="/reservations" class="flex flex-col items-center gap-2 p-4 rounded-xl bg-[var(--color-bg-sunken)] hover:bg-[var(--color-interactive-hover)] transition-colors">
                            <svg class="w-6 h-6 text-[var(--color-brand-primary)]" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M15 5v2m0 4v2m0 4v2M5 5a2 2 0 00-2 2v3a2 2 0 110 4v3a2 2 0 002 2h14a2 2 0 002-2v-3a2 2 0 110-4V7a2 2 0 00-2-2H5z" />
                            </svg>
                            <span class="text-xs font-medium text-[var(--color-text-secondary)]">{t("home.tickets")}</span>
                        </a>
                    </div>
                </div>
            </GlassPanel>
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
