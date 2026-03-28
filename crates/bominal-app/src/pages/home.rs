//! Home dashboard page with live task summary and refresh affordance.

use leptos::{ev, prelude::*};

use crate::{
    api,
    components::{SseReload, StatusChip},
    i18n::t,
    types::TaskInfo,
    utils::{format_date, format_time, status_variant},
};

use super::{ProtectedPage, is_active_task};

fn pull_hint(distance: f64) -> &'static str {
    if distance >= 72.0 {
        "Release to refresh"
    } else if distance > 12.0 {
        "Pull down to refresh"
    } else {
        "Tap refresh or pull down on mobile"
    }
}

fn start_pull_refresh(set_origin: WriteSignal<Option<f64>>, event: ev::PointerEvent) {
    #[cfg(target_arch = "wasm32")]
    {
        let scroll_y = web_sys::window()
            .and_then(|window| window.scroll_y().ok())
            .unwrap_or_default();
        if scroll_y <= 0.0 {
            set_origin.set(Some(event.client_y() as f64));
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    let _ = (set_origin, event);
}

fn update_pull_refresh(
    origin: ReadSignal<Option<f64>>,
    set_distance: WriteSignal<f64>,
    event: ev::PointerEvent,
) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(origin) = origin.get() {
            let delta = (event.client_y() as f64 - origin).clamp(0.0, 110.0);
            set_distance.set(delta);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    let _ = (origin, set_distance, event);
}

fn finish_pull_refresh(
    distance: ReadSignal<f64>,
    set_origin: WriteSignal<Option<f64>>,
    set_distance: WriteSignal<f64>,
    refetch: impl Fn() + 'static,
) {
    if distance.get_untracked() >= 72.0 {
        refetch();
    }

    set_origin.set(None);
    set_distance.set(0.0);
}

#[component]
pub fn HomePage() -> impl IntoView {
    let tasks = Resource::new(|| (), |_| api::list_tasks());
    let (pull_origin, set_pull_origin) = signal::<Option<f64>>(None);
    let (pull_distance, set_pull_distance) = signal(0.0);

    let refetch_tasks = move || {
        tasks.refetch();
    };

    view! {
        <ProtectedPage>
            <SseReload on_event=Callback::new(move |_| { tasks.refetch(); }) />

            <section
                class="mx-auto flex w-full max-w-5xl flex-col gap-6 px-1 md:px-4"
                on:pointerdown=move |event| start_pull_refresh(set_pull_origin, event)
                on:pointermove=move |event| update_pull_refresh(pull_origin, set_pull_distance, event)
                on:pointerup=move |_| {
                    finish_pull_refresh(
                        pull_distance,
                        set_pull_origin,
                        set_pull_distance,
                        refetch_tasks,
                    )
                }
                on:pointercancel=move |_| {
                    set_pull_origin.set(None);
                    set_pull_distance.set(0.0);
                }
            >
                <div class="lg-pull-strip">
                    <div class="flex items-center gap-3">
                        <span class="h-2.5 w-2.5 rounded-full bg-white/70"></span>
                        <span>{move || pull_hint(pull_distance.get())}</span>
                    </div>
                    <button
                        type="button"
                        class="lg-btn-secondary text-xs"
                        on:click=move |_| tasks.refetch()
                    >
                        "Refresh"
                    </button>
                </div>

                <section class="lg-route-card lg-route-card--protected overflow-hidden">
                    <div class="flex flex-col gap-6 lg:flex-row lg:items-end lg:justify-between">
                        <div class="max-w-2xl space-y-3">
                            <p class="lg-route-kicker">{t("home.quick_actions")}</p>
                            <h1>{t("home.welcome")}</h1>
                            <p class="max-w-xl">{t("home.description")}</p>
                        </div>

                        <div class="grid w-full gap-3 sm:grid-cols-2 lg:w-auto">
                            <a
                                href="/search"
                                class="lg-home-link lg-home-link--primary"
                            >
                                <span class="text-sm font-semibold">{t("home.start_search")}</span>
                                <span class="text-xs text-white/75">{t("home.start_search_desc")}</span>
                            </a>

                            <a
                                href="/tasks"
                                class="lg-home-link"
                            >
                                <span class="text-sm font-semibold">{t("home.open_tasks")}</span>
                                <span class="text-xs" style="color: var(--lg-text-secondary);">
                                    {t("home.open_tasks_desc")}
                                </span>
                            </a>
                        </div>
                    </div>
                </section>

                <section class="grid gap-4 lg:grid-cols-[minmax(0,1.2fr)_minmax(20rem,0.8fr)]">
                    <div class="lg-page-card">
                        <div class="flex items-center justify-between gap-3">
                            <div>
                                <p class="lg-route-kicker">{t("home.quick_search")}</p>
                                <h2 class="text-2xl font-semibold tracking-tight">
                                    {t("search.title")}
                                </h2>
                            </div>
                            <a href="/search" class="lg-btn-secondary text-xs">
                                {t("search.go_to_search")}
                            </a>
                        </div>

                        <div class="mt-5 grid gap-3 sm:grid-cols-2">
                            <a href="/search" class="lg-quick-card">
                                <span class="text-sm font-semibold">{t("home.start_search")}</span>
                                <span class="text-xs" style="color: var(--lg-text-secondary);">
                                    {t("home.start_search_desc")}
                                </span>
                            </a>
                            <a href="/reservations" class="lg-quick-card">
                                <span class="text-sm font-semibold">{t("home.tickets")}</span>
                                <span class="text-xs" style="color: var(--lg-text-secondary);">
                                    "Open your latest reservations and payment actions."
                                </span>
                            </a>
                        </div>
                    </div>

                    <div class="lg-page-card">
                        <div class="flex items-center justify-between gap-3">
                            <div>
                                <p class="lg-route-kicker">{t("home.active_tasks")}</p>
                                <h2 class="text-2xl font-semibold tracking-tight">
                                    {t("nav.tasks")}
                                </h2>
                            </div>
                            <a href="/tasks" class="lg-btn-secondary text-xs">
                                {t("home.open_tasks")}
                            </a>
                        </div>

                        <Suspense fallback=move || view! {
                            <div class="mt-5 space-y-3">
                                <div class="lg-skeleton-line h-16"></div>
                                <div class="lg-skeleton-line h-16"></div>
                                <div class="lg-skeleton-line h-16"></div>
                            </div>
                        }>
                            {move || {
                                tasks.get().map(|result| match result {
                                    Ok(all_tasks) => {
                                        let active: Vec<TaskInfo> = all_tasks
                                            .into_iter()
                                            .filter(|task| is_active_task(task.status))
                                            .take(4)
                                            .collect();

                                        if active.is_empty() {
                                            view! {
                                                <div class="lg-empty-state mt-5">
                                                    <p>{t("home.no_active_tasks")}</p>
                                                    <a href="/search" class="lg-btn-secondary text-xs">
                                                        {t("task.create_new")}
                                                    </a>
                                                </div>
                                            }
                                            .into_any()
                                        } else {
                                            view! {
                                                <div class="mt-5 space-y-3">
                                                    {active.into_iter().map(|task| {
                                                        view! {
                                                            <article class="lg-list-card">
                                                                <div class="flex items-start justify-between gap-3">
                                                                    <div class="space-y-1">
                                                                        <p class="text-sm font-semibold tracking-tight">
                                                                            {format!(
                                                                                "{} -> {}",
                                                                                task.departure_station,
                                                                                task.arrival_station,
                                                                            )}
                                                                        </p>
                                                                        <p class="text-xs" style="color: var(--lg-text-secondary);">
                                                                            {format!(
                                                                                "{} {} {}",
                                                                                task.provider,
                                                                                format_date(&task.travel_date),
                                                                                format_time(&task.departure_time),
                                                                            )}
                                                                        </p>
                                                                    </div>

                                                                    <StatusChip
                                                                        label=t(task.status.i18n_key())
                                                                        variant=status_variant(task.status)
                                                                    />
                                                                </div>
                                                            </article>
                                                        }
                                                    }).collect::<Vec<_>>()}
                                                </div>
                                            }
                                            .into_any()
                                        }
                                    }
                                    Err(_) => {
                                        view! {
                                            <div class="lg-empty-state mt-5">
                                                <p>{t("error.load_failed")}</p>
                                            </div>
                                        }
                                        .into_any()
                                    }
                                })
                            }}
                        </Suspense>
                    </div>
                </section>
            </section>
        </ProtectedPage>
    }
}
