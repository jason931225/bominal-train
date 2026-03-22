//! Tasks view — lists active and completed reservation tasks.

use leptos::prelude::*;

use crate::api::tasks::{TaskInfo, TaskStatus, list_tasks};
use crate::components::glass_panel::GlassPanel;
use crate::components::sse_reload::SseReload;
use crate::components::task_card::TaskCard;
use crate::i18n::t;

/// Tasks dashboard showing active and completed reservation tasks.
#[component]
pub fn TasksView() -> impl IntoView {
    let (active_tab, set_active_tab) = signal(true);
    let tasks = Resource::new(|| (), |_| list_tasks());

    view! {
        <SseReload />
        <div class="px-4 pt-6 pb-4 space-y-4 max-w-xl lg:max-w-2xl mx-auto page-enter">
            <h1 class="text-xl font-bold text-[var(--color-text-primary)]">{t("nav.tasks")}</h1>

            // Tab toggle
            <div role="tablist" class="flex bg-[var(--color-bg-sunken)] rounded-xl p-1">
                <button
                    role="tab"
                    aria-selected=move || if active_tab.get() { "true" } else { "false" }
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
                    role="tab"
                    aria-selected=move || if !active_tab.get() { "true" } else { "false" }
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
                            .filter(|task| {
                                let task_active = matches!(
                                    task.status,
                                    TaskStatus::Queued
                                        | TaskStatus::Running
                                        | TaskStatus::Idle
                                        | TaskStatus::AwaitingPayment
                                );
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
                                        <a href="/search" class="inline-block mt-3 text-sm text-[var(--color-brand-text)] font-medium hover:underline">
                                            {t("task.create_new")}
                                        </a>
                                    </div>
                                </GlassPanel>
                            }.into_any()
                        } else {
                            view! {
                                <div aria-live="polite" class="space-y-3">
                                    {display_tasks.into_iter().map(|task| {
                                        let is_active = matches!(
                                            task.status,
                                            TaskStatus::Queued
                                                | TaskStatus::Running
                                                | TaskStatus::Idle
                                                | TaskStatus::AwaitingPayment
                                        );
                                        view! {
                                            <TaskCard task=task is_active=is_active />
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
