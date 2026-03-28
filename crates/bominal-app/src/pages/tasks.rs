//! Tasks page with active/completed tabs, SSE refresh, and swipe-reveal cancel.

use leptos::prelude::*;

use crate::{
    api,
    components::{SseReload, TaskCard},
    i18n::t,
    types::TaskInfo,
};

use super::{ProtectedPage, format_server_error, is_active_task};

#[component]
fn TaskCardRow(task: TaskInfo, active: bool, on_cancel: Callback<String>) -> impl IntoView {
    let task_id = task.id.to_string();

    let card = view! { <TaskCard task=task /> };

    if active {
        view! {
            <div class="lg-swipe-row">
                <div class="lg-swipe-track">
                    {card}
                    <button
                        type="button"
                        class="lg-swipe-action"
                        on:click=move |_| on_cancel.run(task_id.clone())
                    >
                        "Cancel"
                    </button>
                </div>
            </div>
        }
        .into_any()
    } else {
        card.into_any()
    }
}

#[component]
pub fn TasksPage() -> impl IntoView {
    let (active_tab, set_active_tab) = signal(true);
    let tasks = Resource::new(|| (), |_| api::list_tasks());

    let cancel_action = Action::new(|task_id: &String| {
        let task_id = task_id.clone();
        async move { api::delete_task(task_id).await }
    });

    Effect::new(move |_| {
        if let Some(Ok(())) = cancel_action.value().get() {
            tasks.refetch();
        }
    });

    view! {
        <ProtectedPage>
            <SseReload on_event=Callback::new(move |_| { tasks.refetch(); }) />

            <section class="mx-auto flex w-full max-w-5xl flex-col gap-6 px-1 md:px-4">
                <section class="lg-page-card">
                    <div class="flex flex-col gap-4 md:flex-row md:items-end md:justify-between">
                        <div>
                            <p class="lg-route-kicker">{t("nav.tasks")}</p>
                            <h1 class="text-3xl font-semibold tracking-tight">{t("nav.tasks")}</h1>
                            <p class="mt-2 text-sm" style="color: var(--lg-text-secondary);">
                                "Live reservation tasks refresh automatically from the server event stream. Swipe left on mobile to reveal cancel."
                            </p>
                        </div>

                        <div class="lg-tab-strip">
                            <button
                                type="button"
                                class=move || if active_tab.get() { "lg-tab-pill lg-tab-pill--active" } else { "lg-tab-pill" }
                                on:click=move |_| set_active_tab.set(true)
                            >
                                {t("task.active")}
                            </button>
                            <button
                                type="button"
                                class=move || if active_tab.get() { "lg-tab-pill" } else { "lg-tab-pill lg-tab-pill--active" }
                                on:click=move |_| set_active_tab.set(false)
                            >
                                {t("task.completed")}
                            </button>
                        </div>
                    </div>

                    {move || cancel_action.value().get().map(|result| match result {
                        Ok(()) => {
                            view! {
                                <div class="lg-inline-alert lg-inline-alert--success mt-5">
                                    {t("task.cancelled")}
                                </div>
                            }
                            .into_any()
                        }
                        Err(error) => {
                            view! {
                                <div class="lg-inline-alert lg-inline-alert--error mt-5">
                                    {format_server_error(&error)}
                                </div>
                            }
                            .into_any()
                        }
                    })}
                </section>

                <section class="lg-page-card">
                    <Suspense fallback=move || view! {
                        <div class="space-y-3">
                            <div class="lg-skeleton-line h-24"></div>
                            <div class="lg-skeleton-line h-24"></div>
                            <div class="lg-skeleton-line h-24"></div>
                        </div>
                    }>
                        {move || {
                            tasks.get().map(|result| match result {
                                Ok(all_tasks) => {
                                    let display_tasks: Vec<TaskInfo> = all_tasks
                                        .into_iter()
                                        .filter(|task| {
                                            if active_tab.get() {
                                                is_active_task(task.status)
                                            } else {
                                                !is_active_task(task.status)
                                            }
                                        })
                                        .collect();

                                    if display_tasks.is_empty() {
                                        view! {
                                            <div class="lg-empty-state">
                                                <p>
                                                    {if active_tab.get() {
                                                        t("task.no_active")
                                                    } else {
                                                        t("task.no_completed")
                                                    }}
                                                </p>
                                                <a href="/search" class="lg-btn-secondary text-xs">
                                                    {t("task.create_new")}
                                                </a>
                                            </div>
                                        }
                                        .into_any()
                                    } else {
                                        view! {
                                            <div class="space-y-3">
                                                {display_tasks.into_iter().map(|task| {
                                                    let task_id = task.id.to_string();
                                                    view! {
                                                        <TaskCardRow
                                                            task=task.clone()
                                                            active=is_active_task(task.status)
                                                            on_cancel=Callback::new(move |_| {
                                                                cancel_action.dispatch(task_id.clone());
                                                            })
                                                        />
                                                    }
                                                }).collect::<Vec<_>>()}
                                            </div>
                                        }
                                        .into_any()
                                    }
                                }
                                Err(error) => {
                                    view! {
                                        <div class="lg-empty-state">
                                            <p style="color: var(--lg-error);">{format_server_error(&error)}</p>
                                        </div>
                                    }
                                    .into_any()
                                }
                            })
                        }}
                    </Suspense>
                </section>
            </section>
        </ProtectedPage>
    }
}
