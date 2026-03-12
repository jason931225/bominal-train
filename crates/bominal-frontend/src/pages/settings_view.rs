//! Settings page — providers, cards, appearance, accessibility.

use leptos::prelude::*;

use crate::api::auth::{Logout, get_current_user};
use crate::api::cards::{CardInfo, DeleteCard, list_cards};
use crate::api::providers::{AddProvider, DeleteProvider, ProviderInfo, list_providers};
use crate::components::card_brand::{
    CardBrand, detect_brand, format_card_number, strip_non_digits,
};
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

    // Toggle states for inline forms
    let (show_provider_form, set_show_provider_form) = signal(Option::<&'static str>::None);
    let (show_card_form, set_show_card_form) = signal(false);

    view! {
        <div class="px-4 pt-6 pb-4 space-y-4 max-w-xl lg:max-w-2xl mx-auto page-enter">
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
                    <h2 class="text-sm font-semibold text-[var(--color-text-secondary)] uppercase tracking-[0.18em]">{t("provider.settings")}</h2>
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
                        <h2 class="text-sm font-semibold text-[var(--color-text-secondary)] uppercase tracking-[0.18em]">{t("payment.card_label")}</h2>
                        <button
                            class="text-xs px-2 py-1 rounded-lg text-[var(--color-brand-text)] hover:bg-[var(--color-brand-primary)]/10 transition-colors"
                            on:click=move |_| set_show_card_form.update(|v| *v = !*v)
                        >
                            {move || if show_card_form.get() { t("common.cancel") } else { t("payment.add_card") }}
                        </button>
                    </div>

                    // Card add form
                    {move || show_card_form.get().then(|| view! {
                        <CardAddForm
                            on_done=move || set_show_card_form.set(false)
                        />
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

            // Appearance — Theme picker + Mode toggle
            <GlassPanel>
                <div class="p-4 space-y-4">
                    <h2 class="text-sm font-semibold text-[var(--color-text-secondary)] uppercase tracking-[0.18em]">{t("settings.theme")}</h2>

                    // Theme picker — two cards
                    <div class="grid grid-cols-2 gap-3">
                        <button
                            id="theme-rosewood"
                            class="p-3 rounded-xl border border-[var(--color-border-default)] hover:border-[var(--color-brand-border)] transition-colors text-left"
                            attr:onclick="window.__bSetTheme('rosewood');document.getElementById('theme-rosewood').classList.add('glass-active');document.getElementById('theme-clearsky').classList.remove('glass-active');"
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
                            id="theme-clearsky"
                            class="p-3 rounded-xl border border-[var(--color-border-default)] hover:border-[var(--color-brand-border)] transition-colors text-left"
                            attr:onclick="window.__bSetTheme('clear-sky');document.getElementById('theme-clearsky').classList.add('glass-active');document.getElementById('theme-rosewood').classList.remove('glass-active');"
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

                    // Mode toggle — Light / Dark
                    <div class="flex items-center justify-between py-2">
                        <span class="text-sm text-[var(--color-text-primary)]">{t("settings.dark_mode")}</span>
                        <button
                            id="mode-toggle"
                            class="mode-toggle w-10 h-6 rounded-full relative cursor-pointer transition-colors"
                            attr:onclick="var h=document.documentElement;var m=h.getAttribute('data-mode')==='dark'?'light':'dark';window.__bSetMode(m);var b=this;b.style.backgroundColor=m==='dark'?'var(--color-brand-text)':'var(--color-bg-sunken)';b.firstElementChild.style.transform=m==='dark'?'translateX(16px)':'translateX(0)';"
                        >
                            <div class="absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full shadow transition-transform"></div>
                        </button>
                    </div>

                    // Language selector
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

            // Script to sync theme/mode toggle states on page load
            <script>{r#"
(function(){
  var h=document.documentElement;
  var t=h.getAttribute('data-theme')||'rosewood';
  var m=h.getAttribute('data-mode')||'dark';
  var tr=document.getElementById('theme-rosewood');
  var tc=document.getElementById('theme-clearsky');
  var mt=document.getElementById('mode-toggle');
  if(tr&&tc){
    if(t==='rosewood'){tr.classList.add('glass-active');tc.classList.remove('glass-active');}
    else{tc.classList.add('glass-active');tr.classList.remove('glass-active');}
  }
  if(mt){
    mt.style.backgroundColor=m==='dark'?'var(--color-brand-text)':'var(--color-bg-sunken)';
    if(mt.firstElementChild)mt.firstElementChild.style.transform=m==='dark'?'translateX(16px)':'translateX(0)';
  }
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
    let _ = on_done; // used by JS — form reloads page on success

    // Reactive card number signal (raw digits only)
    let (card_number_raw, set_card_number_raw) = signal(String::new());

    // Derived: card brand based on leading digits
    let brand = Memo::new(move |_| detect_brand(&card_number_raw.get()));

    // Derived: formatted display string (e.g. "4111 1111 1111 1111")
    let formatted_display = Memo::new(move |_| {
        let raw = card_number_raw.get();
        if raw.is_empty() {
            String::new()
        } else {
            format_card_number(&raw)
        }
    });

    view! {
        <div class="border border-[var(--color-border-default)] rounded-xl p-3 space-y-3 bg-[var(--color-bg-sunken)]/50">
            <form id="card-add-form">
                <div class="space-y-2">
                    <input
                        type="text"
                        name="label"
                        placeholder="Card label (e.g. My Card)"
                        class="w-full px-3 py-2 bg-[var(--color-bg-sunken)] border border-[var(--color-border-default)] rounded-xl text-sm text-[var(--color-text-primary)] placeholder:text-[var(--color-text-disabled)] focus:outline-none focus:border-[var(--color-border-focus)] transition-colors"
                    />
                    // Card number input with brand badge
                    <div class="relative">
                        <input
                            type="text"
                            name="card_number"
                            required
                            inputmode="numeric"
                            maxlength="16"
                            placeholder="Card number (15-16 digits)"
                            class="w-full px-3 py-2 pr-16 bg-[var(--color-bg-sunken)] border border-[var(--color-border-default)] rounded-xl text-sm text-[var(--color-text-primary)] placeholder:text-[var(--color-text-disabled)] focus:outline-none focus:border-[var(--color-border-focus)] transition-colors font-mono"
                            prop:value=card_number_raw
                            on:input=move |ev| {
                                let raw = strip_non_digits(&event_target_value(&ev));
                                // Cap at 16 digits
                                let capped = if raw.len() > 16 { raw[..16].to_string() } else { raw };
                                set_card_number_raw.set(capped);
                            }
                        />
                        // Brand badge (absolute-positioned inside the input)
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
                    class="w-full mt-3 py-2 btn-glass font-medium rounded-xl text-sm disabled:opacity-50 transition-all"
                >
                    {t("payment.add_card")}
                </button>
            </form>
            <script>{r#"
(function(){
  var form = document.getElementById('card-add-form');
  if (!form) return;
  form.onsubmit = function(e) {
    e.preventDefault();
    var f = e.target;
    if (!window.__submitCard) { alert('Encryption not ready'); return; }
    var btn = f.querySelector('button[type=submit]');
    if (btn) btn.disabled = true;
    window.__submitCard(
      f.label.value, f.card_number.value, f.card_password.value,
      f.birthday.value, f.expire_date.value, f.card_type.value
    ).then(function(r) {
      if (r.ok) { location.reload(); } else { alert(r.error || 'Failed'); if (btn) btn.disabled = false; }
    });
  };
})();
"#}</script>
        </div>
    }
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
