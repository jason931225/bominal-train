//! Home page — dashboard with hero panel, quick action cards, and active task summary.

use leptos::prelude::*;

use crate::api::tasks::{TaskInfo, TaskStatus, list_tasks};
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
        <SseReload on_event=Callback::new(move |_| { tasks.refetch(); }) />
        <div class="px-4 pt-6 pb-4 space-y-6 max-w-xl lg:max-w-2xl mx-auto page-enter">
            // Wordmark header
            <div class="flex items-center gap-3 px-1">
                <span class="app-brand-wordmark text-3xl font-bold bg-clip-text text-transparent tracking-tight">"Bominal"</span>
            </div>

            // Hero glass panel
            <GlassPanel>
                <div class="p-6">
                    <h1 class="text-3xl font-extrabold tracking-tight text-[var(--color-text-primary)] flex items-center gap-3">
                        {t("home.welcome")}
                        <span class="inline-block animate-bounce origin-bottom hover:scale-125 hover:rotate-6 transition-all cursor-default">"👋"</span>
                    </h1>
                    <p class="mt-2 text-base text-[var(--color-text-secondary)] font-medium leading-relaxed max-w-md">
                        {t("home.description")}
                    </p>

                    // 2-column action cards
                    <div class="mt-6 grid grid-cols-2 gap-4">
                        <a href="/search"
                            class="group relative flex flex-col items-start justify-between min-h-[8rem] rounded-3xl p-5 text-left overflow-hidden transition-all duration-300 squish shadow-lg hover:shadow-xl hover:-rotate-2 hover:scale-[1.03]"
                        >
                            <div class="absolute inset-0 bg-gradient-to-br from-[#FF2A54] to-[#FF5E3A] opacity-90 group-hover:opacity-100 transition-opacity"></div>
                            <svg class="w-8 h-8 text-white relative z-10 drop-shadow-sm group-hover:scale-110 transition-transform" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                            </svg>
                            <div class="relative z-10 mt-auto pt-4">
                                <div class="text-sm font-semibold text-white tracking-tight">{t("home.start_search")}</div>
                                <div class="mt-0.5 text-xs text-white/80 font-medium">{t("home.start_search_desc")}</div>
                            </div>
                        </a>

                        <a href="/tasks"
                            class="group flex flex-col items-start justify-between min-h-[8rem] glass-card glass-card-hover p-5 text-left rounded-3xl transition-all duration-300 hover:rotate-1 hover:scale-[1.03]"
                        >
                            <div class="w-10 h-10 rounded-2xl bg-[var(--color-brand-primary)] flex items-center justify-center shrink-0 group-hover:scale-110 transition-transform">
                                <svg class="w-5 h-5 text-[var(--color-brand-text)]" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                                    <path stroke-linecap="round" stroke-linejoin="round" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-6 9l2 2 4-4" />
                                </svg>
                            </div>
                            <div class="mt-auto pt-4">
                                <div class="text-sm font-semibold text-[var(--color-text-primary)]">{t("home.open_tasks")}</div>
                                <div class="mt-0.5 text-xs text-[var(--color-text-secondary)] font-medium">{t("home.open_tasks_desc")}</div>
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
                                    .filter(|t| matches!(t.status, TaskStatus::Queued | TaskStatus::Running | TaskStatus::Idle))
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
                                        <div class="space-y-3 mt-1">
                                            {active.into_iter().enumerate().map(|(i, task)| {
                                                let delay = format!("stagger-{}", i + 1);
                                                view! {
                                                    <div class=format!("flex items-center justify-between p-3 rounded-xl border border-[var(--color-border-subtle)] hover:bg-[var(--color-interactive-hover)] transition-all squish {delay}")>
                                                        <div>
                                                            <p class="text-sm font-semibold tracking-tight text-[var(--color-text-primary)]">
                                                                {format!("{} → {}", task.departure_station, task.arrival_station)}
                                                            </p>
                                                            <p class="text-xs text-[var(--color-text-secondary)] mt-0.5">
                                                                {format!("{} {} · {}", task.provider, task.travel_date, task.departure_time)}
                                                            </p>
                                                        </div>
                                                        <StatusChip label=t(task.status.i18n_key()) variant=status_variant(task.status) />
                                                    </div>
                                                }
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
