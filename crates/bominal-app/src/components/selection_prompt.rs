use leptos::prelude::*;

use crate::i18n::t;

#[component]
pub fn SelectionPrompt(
    count: Signal<usize>,
    on_confirm: Callback<()>,
    on_clear: Callback<()>,
) -> impl IntoView {
    view! {
        <Show when=move || { count.get() > 0 }>
            <div class="lg-selection-prompt">
                <span class="text-sm font-semibold">
                    {move || format!("{} {}", count.get(), t("selection.selected_count"))}
                </span>
                <div class="flex items-center gap-2">
                    <button type="button" class="lg-btn-secondary text-xs" on:click=move |_| on_clear.run(())>
                        {t("selection.clear")}
                    </button>
                    <button type="button" class="lg-btn-primary text-xs" on:click=move |_| on_confirm.run(())>
                        {t("selection.review")}
                    </button>
                </div>
            </div>
        </Show>
    }
}
