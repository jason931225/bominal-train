//! Settings page — providers, cards, appearance, accessibility.
//!
//! Each collapsible section is its own `#[component]` so the view types
//! stay within the default `recursion_limit`.

use leptos::prelude::*;

use crate::api::auth::{Logout, get_current_user};
use crate::api::cards::{CardInfo, DeleteCard, list_cards};
use crate::api::providers::{AddProvider, DeleteProvider, ProviderInfo, list_providers};
use crate::browser;
use crate::components::card_brand::{
    CardBrand, detect_brand, format_card_number, strip_non_digits,
};
use crate::components::glass_panel::GlassPanel;
use crate::i18n::t;

// ── Shared chevron SVG ──────────────────────────────────────────────

fn chevron_svg() -> impl IntoView {
    view! {
        <svg class="settings-chevron w-4 h-4 text-[var(--color-text-tertiary)]" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M19 9l-7 7-7-7"/>
        </svg>
    }
}

// ── Section: User info ──────────────────────────────────────────────

#[component]
fn UserInfoSection() -> impl IntoView {
    let user = Resource::new(|| (), |_| get_current_user());

    view! {
        <Suspense>
            {move || user.get().map(|result| match result {
                Ok(Some(info)) => view! {
                    <GlassPanel>
                        <div class="p-4">
                            <p class="text-sm font-medium text-[var(--color-text-primary)]">{info.display_name.clone()}</p>
                            <p class="text-xs text-[var(--color-text-tertiary)]">{info.email.clone()}</p>
                        </div>
                    </GlassPanel>
                }.into_any(),
                _ => view! { <div></div> }.into_any(),
            })}
        </Suspense>
    }.into_any()
}

// ── Section: Provider Credentials ───────────────────────────────────

#[component]
fn ProviderSection(
    providers: Resource<Result<Vec<ProviderInfo>, ServerFnError>>,
    delete_provider_action: ServerAction<DeleteProvider>,
    add_provider_action: ServerAction<AddProvider>,
    show_provider_form: ReadSignal<Option<&'static str>>,
    set_show_provider_form: WriteSignal<Option<&'static str>>,
) -> impl IntoView {
    view! {
        <details
            open
            class="rounded-2xl border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] overflow-hidden"
        >
            <summary class="flex items-center justify-between px-4 py-3 hover:bg-[var(--color-bg-sunken)]/40 transition-colors">
                <div class="flex items-center gap-3">
                    <div class="w-8 h-8 rounded-xl bg-[var(--color-brand-primary)]/20 flex items-center justify-center flex-shrink-0">
                        <svg class="w-4 h-4 text-[var(--color-brand-text)]" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.8">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M3 13h2l2-6h10l2 6h2M5 17a2 2 0 104 0m6 0a2 2 0 104 0"/>
                        </svg>
                    </div>
                    <span class="text-sm font-semibold text-[var(--color-text-primary)]">{t("settings.section_provider")}</span>
                </div>
                {chevron_svg()}
            </summary>
            <div class="border-t border-[var(--color-border-subtle)] p-4 space-y-3">
                <Suspense fallback=move || view! { <p class="text-xs text-[var(--color-text-tertiary)]">{t("common.loading")}</p> }>
                    {move || providers.get().map(|result| match result {
                        Ok(creds) => {
                            let has_srt = creds.iter().any(|c| c.provider == "SRT");
                            let has_ktx = creds.iter().any(|c| c.provider == "KTX");
                            view! {
                                <div class="space-y-2">
                                    {creds.into_iter().map(|cred| view! {
                                        <ProviderRow cred=cred delete_action=delete_provider_action />
                                    }).collect::<Vec<_>>()}
                                    {(!has_srt).then(|| view! {
                                        <ProviderSetupRow
                                            provider="SRT"
                                            show_form=show_provider_form
                                            set_show_form=set_show_provider_form
                                            add_action=add_provider_action
                                        />
                                    })}
                                    {(!has_ktx).then(|| view! {
                                        <ProviderSetupRow
                                            provider="KTX"
                                            show_form=show_provider_form
                                            set_show_form=set_show_provider_form
                                            add_action=add_provider_action
                                        />
                                    })}
                                </div>
                            }.into_any()
                        }
                        Err(_) => view! {
                            <div class="space-y-2">
                                <ProviderSetupRow
                                    provider="SRT"
                                    show_form=show_provider_form
                                    set_show_form=set_show_provider_form
                                    add_action=add_provider_action
                                />
                                <ProviderSetupRow
                                    provider="KTX"
                                    show_form=show_provider_form
                                    set_show_form=set_show_provider_form
                                    add_action=add_provider_action
                                />
                            </div>
                        }.into_any(),
                    })}
                </Suspense>
                {move || add_provider_action.value().get().map(|result| match result {
                    Ok(info) => view! {
                        <p class="text-xs text-[var(--color-status-success)]">
                            {format!("{} — {}", info.provider, t("provider.saved"))}
                        </p>
                    }.into_any(),
                    Err(e) => view! {
                        <p class="text-xs text-[var(--color-status-error)]">{format!("{e}")}</p>
                    }.into_any(),
                })}
            </div>
        </details>
    }.into_any()
}

// ── Section: Payment Cards ──────────────────────────────────────────

#[component]
fn PaymentSection(
    cards: Resource<Result<Vec<CardInfo>, ServerFnError>>,
    delete_card_action: ServerAction<DeleteCard>,
    show_card_form: ReadSignal<bool>,
    set_show_card_form: WriteSignal<bool>,
) -> impl IntoView {
    let cards_for_form = cards;

    view! {
        <details
            open
            class="rounded-2xl border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] overflow-hidden"
        >
            <summary class="flex items-center justify-between px-4 py-3 hover:bg-[var(--color-bg-sunken)]/40 transition-colors">
                <div class="flex items-center gap-3">
                    <div class="w-8 h-8 rounded-xl bg-[var(--color-brand-primary)]/20 flex items-center justify-center flex-shrink-0">
                        <svg class="w-4 h-4 text-[var(--color-brand-text)]" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.8">
                            <rect x="2" y="5" width="20" height="14" rx="2" stroke-linecap="round" stroke-linejoin="round"/>
                            <path stroke-linecap="round" stroke-linejoin="round" d="M2 10h20"/>
                        </svg>
                    </div>
                    <span class="text-sm font-semibold text-[var(--color-text-primary)]">{t("settings.section_payment")}</span>
                </div>
                {chevron_svg()}
            </summary>
            <div class="border-t border-[var(--color-border-subtle)] p-4 space-y-3">
                <div class="flex items-center justify-between">
                    <span class="text-xs text-[var(--color-text-tertiary)]">{t("payment.card_label")}</span>
                    <button
                        class="text-xs px-2 py-1 rounded-lg text-[var(--color-brand-text)] hover:bg-[var(--color-brand-primary)]/10 transition-colors"
                        on:click=move |_| set_show_card_form.update(|v| *v = !*v)
                    >
                        {move || if show_card_form.get() { t("common.cancel") } else { t("payment.add_card") }}
                    </button>
                </div>
                {move || show_card_form.get().then(|| view! {
                    <CardAddForm on_done=move || {
                        set_show_card_form.set(false);
                        cards_for_form.refetch();
                    } />
                })}
                <Suspense fallback=move || view! { <p class="text-xs text-[var(--color-text-tertiary)]">{t("common.loading")}</p> }>
                    {move || cards.get().map(|result| match result {
                        Ok(card_list) if card_list.is_empty() => view! {
                            <p class="text-xs text-[var(--color-text-tertiary)] py-2">{t("payment.no_cards")}</p>
                        }.into_any(),
                        Ok(card_list) => view! {
                            <div class="space-y-2">
                                {card_list.into_iter().map(|card| view! {
                                    <CardRow card=card delete_action=delete_card_action />
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any(),
                        Err(_) => view! {
                            <p class="text-xs text-[var(--color-text-tertiary)] py-2">{t("error.load_failed")}</p>
                        }.into_any(),
                    })}
                </Suspense>
            </div>
        </details>
    }.into_any()
}

// ── Section: Appearance ─────────────────────────────────────────────

#[component]
fn AppearanceSection() -> impl IntoView {
    let theme = RwSignal::new(browser::current_theme());
    let mode = RwSignal::new(browser::current_mode());
    let locale = RwSignal::new(browser::current_locale());

    let set_theme_choice = move |next: &'static str| {
        theme.set(next.to_string());
        browser::set_theme(next);
    };
    let toggle_mode = move |_| {
        let next = if mode.get() == "dark" {
            "light"
        } else {
            "dark"
        };
        mode.set(next.to_string());
        browser::set_mode(next);
    };
    let update_locale = move |ev| {
        let next = event_target_value(&ev);
        locale.set(next.clone());
        browser::set_locale(&next);
        browser::reload_page();
    };

    view! {
        <details
            class="rounded-2xl border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] overflow-hidden"
        >
            <summary class="flex items-center justify-between px-4 py-3 hover:bg-[var(--color-bg-sunken)]/40 transition-colors">
                <div class="flex items-center gap-3">
                    <div class="w-8 h-8 rounded-xl bg-[var(--color-brand-primary)]/20 flex items-center justify-center flex-shrink-0">
                        <svg class="w-4 h-4 text-[var(--color-brand-text)]" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.8">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M12 3a9 9 0 100 18A9 9 0 0012 3zm0 0v18M3 12h18"/>
                        </svg>
                    </div>
                    <span class="text-sm font-semibold text-[var(--color-text-primary)]">{t("settings.section_appearance")}</span>
                </div>
                {chevron_svg()}
            </summary>
            <div class="border-t border-[var(--color-border-subtle)] p-4 space-y-4">
                <ThemePicker theme set_theme_choice />
                <DarkModeToggle mode toggle_mode />
                <LanguageSelector locale update_locale />
            </div>
        </details>
    }.into_any()
}

#[component]
fn ThemePicker(
    theme: RwSignal<String>,
    set_theme_choice: impl Fn(&'static str) + Copy + Send + Sync + 'static,
) -> impl IntoView {
    view! {
        <div class="space-y-2">
            <p class="text-xs text-[var(--color-text-tertiary)]">{t("settings.theme")}</p>
            <div class="grid grid-cols-2 gap-3">
                <button
                    class=move || {
                        let base = "p-3 rounded-xl border transition-colors text-left";
                        if theme.get() == "rosewood" {
                            format!("{base} glass-active border-[var(--color-brand-border)]")
                        } else {
                            format!("{base} border-[var(--color-border-default)] hover:border-[var(--color-brand-border)]")
                        }
                    }
                    on:click=move |_| set_theme_choice("rosewood")
                >
                    <div class="flex gap-1.5 mb-2">
                        <span class="w-3 h-3 rounded-full" style="background:#8a6050"></span>
                        <span class="w-3 h-3 rounded-full" style="background:#5a7a62"></span>
                        <span class="w-3 h-3 rounded-full" style="background:#9a7a3a"></span>
                    </div>
                    <p class="text-xs font-medium text-[var(--color-text-primary)]">"Rosewood Dusk"</p>
                    <p class="text-[10px] text-[var(--color-text-tertiary)]">"Warm & nostalgic"</p>
                </button>
                <button
                    class=move || {
                        let base = "p-3 rounded-xl border transition-colors text-left";
                        if theme.get() == "clear-sky" {
                            format!("{base} glass-active border-[var(--color-brand-border)]")
                        } else {
                            format!("{base} border-[var(--color-border-default)] hover:border-[var(--color-brand-border)]")
                        }
                    }
                    on:click=move |_| set_theme_choice("clear-sky")
                >
                    <div class="flex gap-1.5 mb-2">
                        <span class="w-3 h-3 rounded-full" style="background:#4a6eaa"></span>
                        <span class="w-3 h-3 rounded-full" style="background:#3a8a60"></span>
                        <span class="w-3 h-3 rounded-full" style="background:#9a7a30"></span>
                    </div>
                    <p class="text-xs font-medium text-[var(--color-text-primary)]">"Clear Sky"</p>
                    <p class="text-[10px] text-[var(--color-text-tertiary)]">"Soft & airy"</p>
                </button>
            </div>
        </div>
    }.into_any()
}

#[component]
fn DarkModeToggle(
    mode: RwSignal<String>,
    toggle_mode: impl Fn(leptos::ev::MouseEvent) + Copy + Send + Sync + 'static,
) -> impl IntoView {
    view! {
        <div class="flex items-center justify-between py-1">
            <span class="text-sm text-[var(--color-text-primary)]">{t("settings.dark_mode")}</span>
            <button
                class="w-10 h-6 rounded-full relative cursor-pointer transition-colors"
                style=move || {
                    if mode.get() == "dark" {
                        "background-color: var(--color-brand-text);".to_string()
                    } else {
                        "background-color: var(--color-bg-sunken);".to_string()
                    }
                }
                on:click=toggle_mode
            >
                <div
                    class="absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full shadow transition-transform"
                    style=move || {
                        if mode.get() == "dark" {
                            "transform: translateX(16px);".to_string()
                        } else {
                            "transform: translateX(0);".to_string()
                        }
                    }
                ></div>
            </button>
        </div>
    }.into_any()
}

#[component]
fn LanguageSelector(
    locale: RwSignal<String>,
    update_locale: impl Fn(leptos::ev::Event) + Copy + Send + Sync + 'static,
) -> impl IntoView {
    view! {
        <div class="flex items-center justify-between py-1">
            <span class="text-sm text-[var(--color-text-primary)]">{t("settings.language")}</span>
            <select
                class="text-sm bg-[var(--color-bg-sunken)] text-[var(--color-text-primary)] rounded-lg px-2 py-1 border border-[var(--color-border-subtle)]"
                prop:value=move || locale.get()
                on:change=update_locale
            >
                <option value="ko">"한국어"</option>
                <option value="en">"English"</option>
                <option value="ja">"日本語"</option>
            </select>
        </div>
    }.into_any()
}

// ── Stub sections ───────────────────────────────────────────────────

#[component]
fn StubSection(icon_path: &'static str, title_key: &'static str) -> impl IntoView {
    view! {
        <details
            class="rounded-2xl border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] overflow-hidden"
        >
            <summary class="flex items-center justify-between px-4 py-3 hover:bg-[var(--color-bg-sunken)]/40 transition-colors">
                <div class="flex items-center gap-3">
                    <div class="w-8 h-8 rounded-xl bg-[var(--color-brand-primary)]/20 flex items-center justify-center flex-shrink-0">
                        <svg class="w-4 h-4 text-[var(--color-brand-text)]" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.8">
                            <path stroke-linecap="round" stroke-linejoin="round" d=icon_path/>
                        </svg>
                    </div>
                    <span class="text-sm font-semibold text-[var(--color-text-primary)]">{t(title_key)}</span>
                </div>
                {chevron_svg()}
            </summary>
            <div class="border-t border-[var(--color-border-subtle)] p-4">
                <p class="text-xs text-[var(--color-text-disabled)]">"Coming soon"</p>
            </div>
        </details>
    }.into_any()
}

// ── Main component ──────────────────────────────────────────────────

/// Settings view with collapsible sectioned hub.
#[component]
pub fn SettingsView() -> impl IntoView {
    let providers = Resource::new(|| (), |_| list_providers());
    let cards = Resource::new(|| (), |_| list_cards());
    let logout_action = ServerAction::<Logout>::new();
    let delete_provider_action = ServerAction::<DeleteProvider>::new();
    let add_provider_action = ServerAction::<AddProvider>::new();
    let delete_card_action = ServerAction::<DeleteCard>::new();

    // Refetch after mutations
    Effect::new(move || {
        if delete_provider_action.value().get().is_some() {
            providers.refetch();
        }
    });
    Effect::new(move || {
        if add_provider_action.value().get().is_some() {
            providers.refetch();
        }
    });
    Effect::new(move || {
        if delete_card_action.value().get().is_some() {
            cards.refetch();
        }
    });

    let (show_provider_form, set_show_provider_form) = signal(Option::<&'static str>::None);
    let (show_card_form, set_show_card_form) = signal(false);

    view! {
        <div class="px-4 pt-6 pb-4 space-y-3 max-w-xl lg:max-w-2xl mx-auto page-enter">
            <style>{r#"
details summary { list-style: none; cursor: pointer; }
details summary::-webkit-details-marker { display: none; }
details[open] .settings-chevron { transform: rotate(180deg); }
.settings-chevron { transition: transform 0.2s ease; }
"#}</style>

            <h1 class="text-xl font-bold text-[var(--color-text-primary)]">{t("settings.title")}</h1>

            <UserInfoSection />
            <ProviderSection providers delete_provider_action add_provider_action show_provider_form set_show_provider_form />
            <PaymentSection cards delete_card_action show_card_form set_show_card_form />
            <AppearanceSection />
            <StubSection icon_path="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" title_key="settings.section_security" />
            <StubSection icon_path="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6 6 0 10-12 0v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9" title_key="settings.section_notifications" />

            <GlassPanel>
                <div class="p-4">
                    <ActionForm action=logout_action>
                        <button
                            type="submit"
                            class="w-full py-2.5 text-sm font-medium text-[var(--color-status-error)] hover:opacity-80 transition-opacity"
                        >
                            {t("auth.logout")}
                        </button>
                    </ActionForm>
                </div>
            </GlassPanel>
        </div>
    }
}

// ── Row components ──────────────────────────────────────────────────

/// Display row for an existing provider credential.
#[component]
fn ProviderRow(cred: ProviderInfo, delete_action: ServerAction<DeleteProvider>) -> impl IntoView {
    let provider_name = cred.provider.clone();
    let status_color = match cred.status.as_str() {
        "valid" => "var(--color-status-success)",
        "invalid" => "var(--color-status-error)",
        _ => "var(--color-text-disabled)",
    };

    view! {
        <div class="flex items-center justify-between py-2 border-b border-[var(--color-border-subtle)] last:border-0">
            <div>
                <p class="text-sm font-medium text-[var(--color-text-primary)]">{cred.provider.clone()}</p>
                <p class="text-xs text-[var(--color-text-tertiary)]">{cred.login_id.clone()}</p>
            </div>
            <div class="flex items-center gap-2">
                <span
                    class="w-2 h-2 rounded-full"
                    style=format!("background-color: {status_color}")
                ></span>
                <ActionForm action=delete_action>
                    <input type="hidden" name="provider" value=provider_name />
                    <button
                        type="submit"
                        class="text-[10px] px-2 py-0.5 rounded text-[var(--color-text-disabled)] hover:text-[var(--color-status-error)] transition-colors"
                    >
                        {t("provider.remove")}
                    </button>
                </ActionForm>
            </div>
        </div>
    }
}

/// Setup prompt for a provider that hasn't been configured — toggles inline form.
#[component]
fn ProviderSetupRow(
    provider: &'static str,
    show_form: ReadSignal<Option<&'static str>>,
    set_show_form: WriteSignal<Option<&'static str>>,
    add_action: ServerAction<AddProvider>,
) -> impl IntoView {
    let is_open = move || show_form.get() == Some(provider);

    view! {
        <div class="py-2 border-b border-[var(--color-border-subtle)] last:border-0">
            <div class="flex items-center justify-between">
                <div>
                    <p class="text-sm font-medium text-[var(--color-text-primary)]">{provider}</p>
                    <p class="text-xs text-[var(--color-text-tertiary)]">{t("provider.not_configured")}</p>
                </div>
                <button
                    class="text-xs px-2 py-0.5 rounded-full bg-[var(--color-bg-sunken)] text-[var(--color-brand-text)] hover:bg-[var(--color-brand-primary)]/10 transition-colors"
                    on:click=move |_| {
                        if is_open() {
                            set_show_form.set(None);
                        } else {
                            set_show_form.set(Some(provider));
                        }
                    }
                >
                    {move || if is_open() { t("common.cancel") } else { t("provider.setup") }}
                </button>
            </div>

            // Inline credential form
            {move || is_open().then(|| view! {
                <div class="mt-3 space-y-3">
                    <ActionForm action=add_action>
                        <input type="hidden" name="provider" value=provider />
                        <div class="space-y-2">
                            <input
                                type="text"
                                name="login_id"
                                required
                                placeholder="Login ID (membership number)"
                                class="w-full px-3 py-2 bg-[var(--color-bg-sunken)] border border-[var(--color-border-default)] rounded-xl text-sm text-[var(--color-text-primary)] placeholder:text-[var(--color-text-disabled)] focus:outline-none focus:border-[var(--color-border-focus)] transition-colors"
                            />
                            <input
                                type="password"
                                name="password"
                                required
                                placeholder="Password"
                                class="w-full px-3 py-2 bg-[var(--color-bg-sunken)] border border-[var(--color-border-default)] rounded-xl text-sm text-[var(--color-text-primary)] placeholder:text-[var(--color-text-disabled)] focus:outline-none focus:border-[var(--color-border-focus)] transition-colors"
                            />
                        </div>
                        <button
                            type="submit"
                            class="w-full mt-3 py-2 btn-glass font-medium rounded-xl text-sm disabled:opacity-50 transition-all"
                        >
                            {t("provider.verify_save")}
                        </button>
                    </ActionForm>
                </div>
            })}
        </div>
    }
}

/// Inline form for adding a payment card with brand detection and formatting.
/// Encrypts via Evervault JS SDK and POSTs to /api/cards.
#[component]
fn CardAddForm(on_done: impl Fn() + Send + Sync + 'static) -> impl IntoView {
    let on_done = std::sync::Arc::new(on_done);

    let label = RwSignal::new(String::new());
    let card_number_raw = RwSignal::new(String::new());
    let card_password = RwSignal::new(String::new());
    let birthday = RwSignal::new(String::new());
    let expire_date = RwSignal::new(String::new());
    let card_type = RwSignal::new(String::from("J"));
    let submit_pending = RwSignal::new(false);
    let error_msg = RwSignal::new(Option::<String>::None);

    let brand = Memo::new(move |_| detect_brand(&card_number_raw.get()));
    let formatted_display = Memo::new(move |_| {
        let raw = card_number_raw.get();
        if raw.is_empty() {
            String::new()
        } else {
            format_card_number(&raw)
        }
    });
    let form_valid = move || {
        let card_len = card_number_raw.get().len();
        (15..=16).contains(&card_len)
            && card_password.get().len() == 2
            && birthday.get().len() == 6
            && expire_date.get().len() == 4
    };

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if submit_pending.get() || !form_valid() {
            return;
        }

        submit_pending.set(true);
        error_msg.set(None);

        let on_done = on_done.clone();
        let _label_value = label.get();
        let _card_number_value = card_number_raw.get();
        let _card_password_value = card_password.get();
        let _birthday_value = birthday.get();
        let _expire_date_value = expire_date.get();
        let _card_type_value = card_type.get();

        #[cfg(target_arch = "wasm32")]
        {
            wasm_bindgen_futures::spawn_local(async move {
                match browser::submit_card(
                    &_label_value,
                    &_card_number_value,
                    &_card_password_value,
                    &_birthday_value,
                    &_expire_date_value,
                    &_card_type_value,
                )
                .await
                {
                    Ok(()) => {
                        submit_pending.set(false);
                        error_msg.set(None);
                        (on_done)();
                    }
                    Err(error) => {
                        submit_pending.set(false);
                        error_msg.set(Some(error));
                    }
                }
            });
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = on_done;
            submit_pending.set(false);
            error_msg.set(Some(
                "Card submission is only available in the browser".to_string(),
            ));
        }
    };

    view! {
        <div class="border border-[var(--color-border-default)] rounded-xl p-3 space-y-3 bg-[var(--color-bg-sunken)]/50">
            {move || error_msg.get().map(|msg| view! {
                <p class="text-xs text-[var(--color-status-error)]">{msg}</p>
            })}
            <form on:submit=on_submit>
                <CardFormFields label card_number_raw card_password birthday expire_date card_type formatted_display brand />
                <button
                    type="submit"
                    class="w-full mt-3 py-2 btn-glass font-medium rounded-xl text-sm disabled:opacity-50 transition-all"
                    disabled=move || submit_pending.get() || !form_valid()
                >
                    {move || if submit_pending.get() {
                        t("common.loading")
                    } else {
                        t("payment.add_card")
                    }}
                </button>
            </form>
        </div>
    }
}

#[component]
fn CardFormFields(
    label: RwSignal<String>,
    card_number_raw: RwSignal<String>,
    card_password: RwSignal<String>,
    birthday: RwSignal<String>,
    expire_date: RwSignal<String>,
    card_type: RwSignal<String>,
    formatted_display: Memo<String>,
    brand: Memo<CardBrand>,
) -> impl IntoView {
    view! {
        <div class="space-y-2">
            <input
                type="text"
                placeholder="Card label (e.g. My Card)"
                prop:value=move || label.get()
                on:input=move |ev| label.set(event_target_value(&ev))
                class="w-full px-3 py-2 bg-[var(--color-bg-sunken)] border border-[var(--color-border-default)] rounded-xl text-sm text-[var(--color-text-primary)] placeholder:text-[var(--color-text-disabled)] focus:outline-none focus:border-[var(--color-border-focus)] transition-colors"
            />
            // Card number input with brand badge
            <div class="relative">
                <input
                    type="text"
                    required
                    inputmode="numeric"
                    maxlength="16"
                    placeholder="Card number (15-16 digits)"
                    class="w-full px-3 py-2 pr-16 bg-[var(--color-bg-sunken)] border border-[var(--color-border-default)] rounded-xl text-sm text-[var(--color-text-primary)] placeholder:text-[var(--color-text-disabled)] focus:outline-none focus:border-[var(--color-border-focus)] transition-colors font-mono"
                    prop:value=move || formatted_display.get()
                    on:input=move |ev| {
                        let raw = strip_non_digits(&event_target_value(&ev));
                        let capped = if raw.len() > 16 { raw[..16].to_string() } else { raw };
                        card_number_raw.set(capped);
                    }
                />
                {move || {
                    let b = brand.get();
                    (b != CardBrand::Unknown).then(|| {
                        let label = b.label();
                        let bg = b.badge_color();
                        view! {
                            <span
                                class="absolute right-2 top-1/2 -translate-y-1/2 text-[10px] font-bold text-white px-1.5 py-0.5 rounded"
                                style=format!("background-color: {bg}")
                            >
                                {label}
                            </span>
                        }
                    })
                }}
            </div>
            // Formatted card number preview
            {move || {
                let display = formatted_display.get();
                (!display.is_empty()).then(|| view! {
                    <p class="text-xs text-[var(--color-text-tertiary)] font-mono pl-1 -mt-1">
                        {display}
                    </p>
                })
            }}
            <div class="grid grid-cols-3 gap-2">
                <input
                    type="password"
                    required
                    maxlength="2"
                    inputmode="numeric"
                    placeholder="PW (2)"
                    prop:value=move || card_password.get()
                    on:input=move |ev| {
                        let raw = strip_non_digits(&event_target_value(&ev));
                        let capped = if raw.len() > 2 { raw[..2].to_string() } else { raw };
                        card_password.set(capped);
                    }
                    class="px-3 py-2 bg-[var(--color-bg-sunken)] border border-[var(--color-border-default)] rounded-xl text-sm text-[var(--color-text-primary)] placeholder:text-[var(--color-text-disabled)] focus:outline-none focus:border-[var(--color-border-focus)] transition-colors text-center"
                />
                <input
                    type="text"
                    required
                    maxlength="6"
                    inputmode="numeric"
                    placeholder="YYMMDD"
                    prop:value=move || birthday.get()
                    on:input=move |ev| {
                        let raw = strip_non_digits(&event_target_value(&ev));
                        let capped = if raw.len() > 6 { raw[..6].to_string() } else { raw };
                        birthday.set(capped);
                    }
                    class="px-3 py-2 bg-[var(--color-bg-sunken)] border border-[var(--color-border-default)] rounded-xl text-sm text-[var(--color-text-primary)] placeholder:text-[var(--color-text-disabled)] focus:outline-none focus:border-[var(--color-border-focus)] transition-colors text-center"
                />
                <input
                    type="text"
                    required
                    maxlength="4"
                    inputmode="numeric"
                    placeholder="MMYY"
                    prop:value=move || expire_date.get()
                    on:input=move |ev| {
                        let raw = strip_non_digits(&event_target_value(&ev));
                        let capped = if raw.len() > 4 { raw[..4].to_string() } else { raw };
                        expire_date.set(capped);
                    }
                    class="px-3 py-2 bg-[var(--color-bg-sunken)] border border-[var(--color-border-default)] rounded-xl text-sm text-[var(--color-text-primary)] placeholder:text-[var(--color-text-disabled)] focus:outline-none focus:border-[var(--color-border-focus)] transition-colors text-center"
                />
            </div>
            <select
                prop:value=move || card_type.get()
                on:change=move |ev| card_type.set(event_target_value(&ev))
                class="w-full px-3 py-2 bg-[var(--color-bg-sunken)] border border-[var(--color-border-default)] rounded-xl text-sm text-[var(--color-text-primary)] focus:outline-none focus:border-[var(--color-border-focus)] transition-colors"
            >
                <option value="J">{t("payment.credit_card")}</option>
                <option value="S">{t("payment.debit_card")}</option>
            </select>
        </div>
    }.into_any()
}

/// Display row for a payment card.
#[component]
fn CardRow(card: CardInfo, delete_action: ServerAction<DeleteCard>) -> impl IntoView {
    let card_id = card.id.to_string();
    view! {
        <div class="flex items-center justify-between py-2 border-b border-[var(--color-border-subtle)] last:border-0">
            <div>
                <p class="text-sm font-medium text-[var(--color-text-primary)]">{card.label.clone()}</p>
                <p class="text-xs text-[var(--color-text-tertiary)]">
                    {format!("{} ····{}", card.card_type_name, card.last_four)}
                </p>
            </div>
            <ActionForm action=delete_action>
                <input type="hidden" name="card_id" value=card_id />
                <button
                    type="submit"
                    class="text-[10px] px-2 py-0.5 rounded text-[var(--color-text-disabled)] hover:text-[var(--color-status-error)] transition-colors"
                >
                    {t("provider.remove")}
                </button>
            </ActionForm>
        </div>
    }
}
