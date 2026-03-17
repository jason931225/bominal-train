//! Browser-side SSE bridge that reloads task pages on task updates.

use leptos::prelude::*;

#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, closure::Closure};

#[cfg(target_arch = "wasm32")]
use web_sys::{Event, EventSource};

#[cfg(target_arch = "wasm32")]
thread_local! {
    static TASK_EVENT_SOURCE: RefCell<Option<EventSource>> = const { RefCell::new(None) };
    static TASK_EVENT_HANDLER: RefCell<Option<Closure<dyn FnMut(Event)>>> = const { RefCell::new(None) };
    static TASK_ERROR_HANDLER: RefCell<Option<Closure<dyn FnMut(Event)>>> = const { RefCell::new(None) };
}

#[cfg(target_arch = "wasm32")]
fn init_task_events() {
    TASK_EVENT_SOURCE.with(|source_cell| {
        if source_cell.borrow().is_some() {
            return;
        }

        let Ok(source) = EventSource::new("/api/tasks/events") else {
            return;
        };

        let on_task_update = Closure::<dyn FnMut(Event)>::wrap(Box::new(move |_| {
            crate::browser::reload_page();
        }));
        let on_error = Closure::<dyn FnMut(Event)>::wrap(Box::new(move |_| {
            TASK_EVENT_SOURCE.with(|source_cell| {
                if let Some(src) = source_cell.borrow().as_ref() {
                    if src.ready_state() == 2 {
                        crate::browser::reload_page();
                    }
                }
            });
        }));

        let _ = source.add_event_listener_with_callback(
            "task_update",
            on_task_update.as_ref().unchecked_ref(),
        );
        let _ = source.add_event_listener_with_callback("error", on_error.as_ref().unchecked_ref());

        TASK_EVENT_HANDLER.with(|handler| {
            *handler.borrow_mut() = Some(on_task_update);
        });
        TASK_ERROR_HANDLER.with(|handler| {
            *handler.borrow_mut() = Some(on_error);
        });
        *source_cell.borrow_mut() = Some(source);
    });
}

/// Keeps an SSE connection alive while task-heavy pages are open.
#[component]
pub fn SseReload() -> impl IntoView {
    #[cfg(target_arch = "wasm32")]
    {
        Effect::new(|_| {
            init_task_events();
        });
    }

    view! { <></> }
}
