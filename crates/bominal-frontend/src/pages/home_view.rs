//! Home page — dashboard with hero panel, quick action cards, and active task summary.

use leptos::prelude::*;

use crate::api::tasks::{TaskInfo, list_tasks};
use crate::components::glass_panel::GlassPanel;
use crate::components::sse_reload::SseReload;
use crate::components::status_chip::StatusChip;
use crate::i18n::t;
use crate::utils::status_variant;

/// Home dashboard with hero panel and quick actions.
#[component]
pub fn HomeView() -> impl IntoView {
    let tasks = Resource::new(|| (), |_| list_tasks());

    view! {
        <SseReload />
        <div class="px-4 pt-6 pb-4 space-y-5 max-w-xl lg:max-w-2xl mx-auto page-enter">
            // Wordmark header
            <div class="flex items-center gap-3">
                <span class="app-brand-wordmark text-2xl font-bold bg-clip-text text-transparent tracking-tight">"Bominal"</span>
            </div>

            // Hero glass panel
            <GlassPanel>
                <div class="p-5">
                    <h1 class="text-2xl font-bold tracking-tight text-[var(--color-text-primary)]">{t("home.welcome")}</h1>
                    <p class="mt-2 text-sm text-[var(--color-text-tertiary)] leading-relaxed max-w-md">
                        {t("home.description")}
                    </p>

                    // 2-column action cards
                    <div class="mt-5 grid grid-cols-2 gap-3">
                        <a href="/search"
                            class="flex flex-col items-start justify-between min-h-[7rem] rounded-2xl border border-[var(--color-brand-border)] bg-[var(--color-brand-primary)] px-4 py-3.5 text-left hover:opacity-80 transition-opacity"
                        >
                            <svg class="w-5 h-5 text-[var(--color-brand-text)]" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                            </svg>
                            <div>
                                <div class="text-sm font-semibold text-[var(--color-text-primary)]">{t("home.start_search")}</div>
                                <div class="mt-0.5 text-xs text-[var(--color-text-tertiary)]">{t("home.start_search_desc")}</div>
                            </div>
                        </a>

                        <a href="/tasks"
                            class="flex flex-col items-start justify-between min-h-[7rem] rounded-2xl border border-[var(--color-border-default)] bg-[var(--color-bg-elevated)] px-4 py-3.5 text-left hover:bg-[var(--color-interactive-hover)] transition-colors"
                        >
                            <svg class="w-5 h-5 text-[var(--color-text-tertiary)]" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-6 9l2 2 4-4" />
                            </svg>
                            <div>
                                <div class="text-sm font-semibold text-[var(--color-text-primary)]">{t("home.open_tasks")}</div>
                                <div class="mt-0.5 text-xs text-[var(--color-text-tertiary)]">{t("home.open_tasks_desc")}</div>
                            </div>
                        </a>
                    </div>
                </div>
            </GlassPanel>

            // Active tasks summary
            <GlassPanel>
                <div class="p-4">
                    <h2 class="text-sm font-semibold text-[var(--color-text-secondary)] uppercase tracking-[0.18em] mb-3">{t("home.active_tasks")}</h2>
                    <Suspense fallback=move || view! {
                        <div class="text-center py-6">
                            <div class="shimmer h-10 rounded-xl"></div>
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
                                        <div class="text-center py-6">
                                            <p class="text-[var(--color-text-tertiary)] text-sm">{t("home.no_active_tasks")}</p>
                                            <a href="/search" class="inline-block mt-3 text-sm text-[var(--color-brand-text)] font-medium hover:underline">
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
                                <div class="text-center py-6">
                                    <p class="text-[var(--color-text-tertiary)] text-sm">{t("error.load_failed")}</p>
                                </div>
                            }.into_any(),
                        })}
                    </Suspense>
                </div>
            </GlassPanel>
        </div>
    }
}
