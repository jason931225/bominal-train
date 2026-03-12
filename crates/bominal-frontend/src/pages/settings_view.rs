//! Settings page — providers, cards, appearance, accessibility.

use leptos::prelude::*;

use crate::api::auth::{Logout, get_current_user};
use crate::api::cards::{AddCard, DeleteCard, list_cards, CardInfo};
use crate::api::providers::{AddProvider, DeleteProvider, list_providers, ProviderInfo};
use crate::components::glass_panel::GlassPanel;
use crate::i18n::t;

/// Settings view with sections for providers, cards, appearance, accessibility.
#[component]
pub fn SettingsView() -> impl IntoView {
    let user = Resource::new(|| (), |_| get_current_user());
    let providers = Resource::new(|| (), |_| list_providers());
    let cards = Resource::new(|| (), |_| list_cards());
    let logout_action = ServerAction::<Logout>::new();
    let delete_provider_action = ServerAction::<DeleteProvider>::new();
    let add_provider_action = ServerAction::<AddProvider>::new();
    let delete_card_action = ServerAction::<DeleteCard>::new();
    let add_card_action = ServerAction::<AddCard>::new();

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
    Effect::new(move || {
        if add_card_action.value().get().is_some() {
            cards.refetch();
        }
    });

    // Toggle states for inline forms
    let (show_provider_form, set_show_provider_form) = signal(Option::<&'static str>::None);
    let (show_card_form, set_show_card_form) = signal(false);

    view! {
        <div class="px-4 pt-6 pb-4 space-y-4">
            <h1 class="text-xl font-bold text-[var(--color-text-primary)]">{t("settings.title")}</h1>

            // User info
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

            // Provider Credentials
            <GlassPanel>
                <div class="p-4 space-y-3">
                    <h2 class="text-sm font-semibold text-[var(--color-text-secondary)] uppercase tracking-wider">{t("provider.settings")}</h2>
                    <Suspense fallback=move || view! { <p class="text-xs text-[var(--color-text-tertiary)]">{t("common.loading")}</p> }>
                        {move || providers.get().map(|result| match result {
                            Ok(creds) => {
                                let has_srt = creds.iter().any(|c| c.provider == "SRT");
                                let has_ktx = creds.iter().any(|c| c.provider == "KTX");
                                view! {
                                    <div class="space-y-2">
                                        {creds.into_iter().map(|cred| view! {
                                            <ProviderRow
                                                cred=cred
                                                delete_action=delete_provider_action
                                            />
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

                    // Add provider result feedback
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
            </GlassPanel>

            // Payment Cards
            <GlassPanel>
                <div class="p-4 space-y-3">
                    <div class="flex items-center justify-between">
                        <h2 class="text-sm font-semibold text-[var(--color-text-secondary)] uppercase tracking-wider">{t("payment.card_label")}</h2>
                        <button
                            class="text-xs px-2 py-1 rounded-lg text-[var(--color-brand-primary)] hover:bg-[var(--color-brand-primary)]/10 transition-colors"
                            on:click=move |_| set_show_card_form.update(|v| *v = !*v)
                        >
                            {move || if show_card_form.get() { t("common.cancel") } else { t("payment.add_card") }}
                        </button>
                    </div>

                    // Card add form
                    {move || show_card_form.get().then(|| view! {
                        <CardAddForm
                            add_action=add_card_action
                            on_done=move || set_show_card_form.set(false)
                        />
                    })}

                    // Card add result feedback
                    {move || add_card_action.value().get().map(|result| match result {
                        Ok(card) => view! {
                            <p class="text-xs text-[var(--color-status-success)]">
                                {format!("Card ····{} added", card.last_four)}
                            </p>
                        }.into_any(),
                        Err(e) => view! {
                            <p class="text-xs text-[var(--color-status-error)]">{format!("{e}")}</p>
                        }.into_any(),
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
            </GlassPanel>

            // Appearance
            <GlassPanel>
                <div class="p-4 space-y-3">
                    <h2 class="text-sm font-semibold text-[var(--color-text-secondary)] uppercase tracking-wider">{t("settings.theme")}</h2>
                    <div class="flex items-center justify-between py-2">
                        <span class="text-sm text-[var(--color-text-primary)]">{t("settings.dark_mode")}</span>
                        <button
                            id="dark-mode-toggle"
                            class="dark-mode-toggle w-10 h-6 rounded-full relative cursor-pointer transition-colors"
                            attr:onclick="var d=document.documentElement;var t=d.getAttribute('data-theme')==='dark'?'light':'dark';window.__bSetTheme(t);var b=this;b.className=t==='dark'?'dark-mode-toggle w-10 h-6 rounded-full relative cursor-pointer transition-colors bg-[var(--color-brand-primary)]':'dark-mode-toggle w-10 h-6 rounded-full relative cursor-pointer transition-colors bg-[var(--color-bg-sunken)]';b.firstElementChild.style.transform=t==='dark'?'translateX(16px)':'translateX(0)';"
                        >
                            <div class="absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full shadow transition-transform"></div>
                        </button>
                    </div>
                    <div class="flex items-center justify-between py-2">
                        <span class="text-sm text-[var(--color-text-primary)]">{t("settings.theme")}</span>
                        <select
                            id="palette-select"
                            class="text-sm bg-[var(--color-bg-sunken)] text-[var(--color-text-primary)] rounded-lg px-2 py-1 border border-[var(--color-border-subtle)]"
                            attr:onchange="window.__bSetPalette(this.value);"
                        >
                            // Palette names kept as literals (Leptos <option> limitation)
                            <option value="current">"Default"</option>
                            <option value="transit-slate">"Transit Slate"</option>
                            <option value="night-teal">"Night Teal"</option>
                            <option value="warm-platform">"Warm Platform"</option>
                        </select>
                    </div>
                </div>
            </GlassPanel>

            // Accessibility
            <GlassPanel>
                <div class="p-4 space-y-3">
                    <h2 class="text-sm font-semibold text-[var(--color-text-secondary)] uppercase tracking-wider">{t("settings.accessibility")}</h2>
                    <div class="flex items-center justify-between py-2">
                        <span class="text-sm text-[var(--color-text-primary)]">{t("settings.colorblind")}</span>
                        <button
                            id="colorblind-toggle"
                            class="colorblind-toggle w-10 h-6 rounded-full relative cursor-pointer transition-colors"
                            attr:onclick="var d=document.documentElement;var c=d.getAttribute('data-colorblind')==='true'?'false':'true';window.__bSetColorblind(c);var b=this;b.className=c==='true'?'colorblind-toggle w-10 h-6 rounded-full relative cursor-pointer transition-colors bg-[var(--color-brand-primary)]':'colorblind-toggle w-10 h-6 rounded-full relative cursor-pointer transition-colors bg-[var(--color-bg-sunken)]';b.firstElementChild.style.transform=c==='true'?'translateX(16px)':'translateX(0)';"
                        >
                            <div class="absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full shadow transition-transform"></div>
                        </button>
                    </div>
                    <div class="flex items-center justify-between py-2">
                        <span class="text-sm text-[var(--color-text-primary)]">{t("settings.language")}</span>
                        <select
                            id="language-select"
                            class="text-sm bg-[var(--color-bg-sunken)] text-[var(--color-text-primary)] rounded-lg px-2 py-1 border border-[var(--color-border-subtle)]"
                            attr:onchange="document.cookie='bominal-locale='+this.value+';path=/;max-age=31536000';location.reload();"
                        >
                            <option value="ko">"한국어"</option>
                            <option value="en">"English"</option>
                            <option value="ja">"日本語"</option>
                        </select>
                    </div>
                </div>
            </GlassPanel>

            // Script to sync toggle states on page load
            <script>{r#"
(function(){
  var h=document.documentElement;
  var dm=document.getElementById('dark-mode-toggle');
  var cb=document.getElementById('colorblind-toggle');
  var ps=document.getElementById('palette-select');
  if(dm){
    var isDark=h.getAttribute('data-theme')==='dark';
    dm.className=isDark?'dark-mode-toggle w-10 h-6 rounded-full relative cursor-pointer transition-colors bg-[var(--color-brand-primary)]':'dark-mode-toggle w-10 h-6 rounded-full relative cursor-pointer transition-colors bg-[var(--color-bg-sunken)]';
    if(dm.firstElementChild)dm.firstElementChild.style.transform=isDark?'translateX(16px)':'translateX(0)';
  }
  if(cb){
    var isCb=h.getAttribute('data-colorblind')==='true';
    cb.className=isCb?'colorblind-toggle w-10 h-6 rounded-full relative cursor-pointer transition-colors bg-[var(--color-brand-primary)]':'colorblind-toggle w-10 h-6 rounded-full relative cursor-pointer transition-colors bg-[var(--color-bg-sunken)]';
    if(cb.firstElementChild)cb.firstElementChild.style.transform=isCb?'translateX(16px)':'translateX(0)';
  }
  if(ps){ps.value=h.getAttribute('data-palette')||'current';}
})();
"#}</script>

            // Sign out
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

/// Display row for an existing provider credential.
#[component]
fn ProviderRow(
    cred: ProviderInfo,
    delete_action: ServerAction<DeleteProvider>,
) -> impl IntoView {
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
                    class="text-xs px-2 py-0.5 rounded-full bg-[var(--color-bg-sunken)] text-[var(--color-brand-primary)] hover:bg-[var(--color-brand-primary)]/10 transition-colors"
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
                            class="w-full mt-3 py-2 bg-[var(--color-brand-primary)] text-white font-medium rounded-xl text-sm hover:opacity-90 disabled:opacity-50 transition-all"
                        >
                            {t("provider.verify_save")}
                        </button>
                    </ActionForm>
                </div>
            })}
        </div>
    }
}

/// Inline form for adding a payment card.
#[component]
fn CardAddForm(
    add_action: ServerAction<AddCard>,
    on_done: impl Fn() + Send + Sync + 'static,
) -> impl IntoView {
    // Close form on successful add
    Effect::new(move || {
        if let Some(Ok(_)) = add_action.value().get() {
            on_done();
        }
    });

    view! {
        <div class="border border-[var(--color-border-default)] rounded-xl p-3 space-y-3 bg-[var(--color-bg-sunken)]/50">
            <ActionForm action=add_action>
                <div class="space-y-2">
                    <input
                        type="text"
                        name="label"
                        placeholder="Card label (e.g. My Card)"
                        class="w-full px-3 py-2 bg-[var(--color-bg-sunken)] border border-[var(--color-border-default)] rounded-xl text-sm text-[var(--color-text-primary)] placeholder:text-[var(--color-text-disabled)] focus:outline-none focus:border-[var(--color-border-focus)] transition-colors"
                    />
                    <input
                        type="text"
                        name="card_number"
                        required
                        inputmode="numeric"
                        maxlength="16"
                        placeholder="Card number (15-16 digits)"
                        class="w-full px-3 py-2 bg-[var(--color-bg-sunken)] border border-[var(--color-border-default)] rounded-xl text-sm text-[var(--color-text-primary)] placeholder:text-[var(--color-text-disabled)] focus:outline-none focus:border-[var(--color-border-focus)] transition-colors font-mono"
                    />
                    <div class="grid grid-cols-3 gap-2">
                        <input
                            type="password"
                            name="card_password"
                            required
                            maxlength="2"
                            inputmode="numeric"
                            placeholder="PW (2)"
                            class="px-3 py-2 bg-[var(--color-bg-sunken)] border border-[var(--color-border-default)] rounded-xl text-sm text-[var(--color-text-primary)] placeholder:text-[var(--color-text-disabled)] focus:outline-none focus:border-[var(--color-border-focus)] transition-colors text-center"
                        />
                        <input
                            type="text"
                            name="birthday"
                            required
                            maxlength="6"
                            inputmode="numeric"
                            placeholder="YYMMDD"
                            class="px-3 py-2 bg-[var(--color-bg-sunken)] border border-[var(--color-border-default)] rounded-xl text-sm text-[var(--color-text-primary)] placeholder:text-[var(--color-text-disabled)] focus:outline-none focus:border-[var(--color-border-focus)] transition-colors text-center"
                        />
                        <input
                            type="text"
                            name="expire_date"
                            required
                            maxlength="4"
                            inputmode="numeric"
                            placeholder="MMYY"
                            class="px-3 py-2 bg-[var(--color-bg-sunken)] border border-[var(--color-border-default)] rounded-xl text-sm text-[var(--color-text-primary)] placeholder:text-[var(--color-text-disabled)] focus:outline-none focus:border-[var(--color-border-focus)] transition-colors text-center"
                        />
                    </div>
                    <select
                        name="card_type"
                        class="w-full px-3 py-2 bg-[var(--color-bg-sunken)] border border-[var(--color-border-default)] rounded-xl text-sm text-[var(--color-text-primary)] focus:outline-none focus:border-[var(--color-border-focus)] transition-colors"
                    >
                        <option value="J" selected>{t("payment.credit_card")}</option>
                        <option value="S">{t("payment.debit_card")}</option>
                    </select>
                </div>
                <button
                    type="submit"
                    class="w-full mt-3 py-2 bg-[var(--color-brand-primary)] text-white font-medium rounded-xl text-sm hover:opacity-90 disabled:opacity-50 transition-all"
                >
                    {t("payment.add_card")}
                </button>
            </ActionForm>
        </div>
    }
}

/// Display row for a payment card.
#[component]
fn CardRow(
    card: CardInfo,
    delete_action: ServerAction<DeleteCard>,
) -> impl IntoView {
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
