//! Browser-side SSE bridge that triggers reactive updates on task events.

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
    static SSE_CALLBACK: RefCell<Option<Callback<()>>> = const { RefCell::new(None) };
}

#[cfg(target_arch = "wasm32")]
fn init_task_events(on_event: Callback<()>) {
    // Store the current callback (swapped when a different page mounts)
    SSE_CALLBACK.with(|cb| {
        *cb.borrow_mut() = Some(on_event);
    });

    // Create EventSource only once — reused across pages
    TASK_EVENT_SOURCE.with(|source_cell| {
        if source_cell.borrow().is_some() {
            return;
        }

        let Ok(source) = EventSource::new("/api/tasks/events") else {
            return;
        };

        let on_task_update = Closure::<dyn FnMut(Event)>::wrap(Box::new(move |_| {
            SSE_CALLBACK.with(|cb| {
                if let Some(callback) = cb.borrow().as_ref() {
                    callback.run(());
                }
            });
        }));

        let _ = source.add_event_listener_with_callback(
            "task_update",
            on_task_update.as_ref().unchecked_ref(),
        );

        TASK_EVENT_HANDLER.with(|handler| {
            *handler.borrow_mut() = Some(on_task_update);
        });
        *source_cell.borrow_mut() = Some(source);
    });
}

/// Keeps an SSE connection alive and invokes a callback on task updates.
#[component]
pub fn SseReload(on_event: Callback<()>) -> impl IntoView {
    #[cfg(target_arch = "wasm32")]
    Effect::new(move |_| {
        init_task_events(on_event.clone());
    });

    #[cfg(not(target_arch = "wasm32"))]
    let _ = on_event;

    view! { <></> }
}
