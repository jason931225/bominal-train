use leptos::prelude::*;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <main class="mx-auto flex min-h-screen w-full max-w-6xl flex-col px-6 py-12 lg:px-10">
            <section class="rounded-3xl border border-slate-200 bg-white p-8 shadow-sm">
                <p class="text-xs uppercase tracking-[0.24em] text-slate-500">"Bominal Rust Cutover"</p>
                <h1 class="mt-3 text-4xl font-black leading-tight text-slate-900 lg:text-6xl">
                    "Leptos SSR + Tailwind"
                </h1>
                <p class="mt-4 max-w-3xl text-base leading-7 text-slate-600 lg:text-lg">
                    "Parallel runtime is active. This frontend is server-rendered by Leptos on axum with a Supabase-first backend contract and Redis-backed runtime services."
                </p>
                <div class="mt-8 grid grid-cols-1 gap-4 sm:grid-cols-3">
                    <article class="rounded-2xl bg-slate-900 p-5 text-slate-100">
                        <p class="text-xs uppercase tracking-wide text-slate-400">"API"</p>
                        <p class="mt-2 text-2xl font-semibold">"axum 0.8"</p>
                    </article>
                    <article class="rounded-2xl bg-cyan-600 p-5 text-cyan-50">
                        <p class="text-xs uppercase tracking-wide text-cyan-100">"Data"</p>
                        <p class="mt-2 text-2xl font-semibold">"sqlx + Supabase"</p>
                    </article>
                    <article class="rounded-2xl bg-emerald-600 p-5 text-emerald-50">
                        <p class="text-xs uppercase tracking-wide text-emerald-100">"Runtime"</p>
                        <p class="mt-2 text-2xl font-semibold">"Tokio + Redis"</p>
                    </article>
                </div>
            </section>
        </main>
    }
}

pub fn render_home() -> String {
    view! { <HomePage /> }.to_html()
}
