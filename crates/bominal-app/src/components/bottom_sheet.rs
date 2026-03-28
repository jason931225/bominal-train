use leptos::prelude::*;

#[component]
pub fn BottomSheet(
    open: ReadSignal<bool>,
    on_close: Callback<()>,
    #[prop(into, optional)] title: Option<String>,
    children: Children,
) -> impl IntoView {
    let labelled_by = title.as_ref().map(|_| "lg-bottom-sheet-title");
    let rendered_children = children();

    view! {
        <div
            class="lg-bottom-sheet-root"
            style=move || if open.get() { "display: block;" } else { "display: none;" }
        >
            <div class="lg-bottom-sheet-backdrop" on:click=move |_| on_close.run(())></div>
            <div class="lg-bottom-sheet-host">
                <section
                    role="dialog"
                    aria-modal="true"
                    aria-labelledby=labelled_by
                    class="lg-bottom-sheet"
                    on:click=|event| event.stop_propagation()
                >
                    <div class="lg-bottom-sheet__handle"></div>
                    {title.as_ref().map(|title| {
                        view! {
                            <div class="lg-bottom-sheet__header">
                                <h3 id="lg-bottom-sheet-title">{title.clone()}</h3>
                            </div>
                        }
                    })}
                    <div class="lg-bottom-sheet__content">{rendered_children}</div>
                </section>
            </div>
        </div>
    }
}
