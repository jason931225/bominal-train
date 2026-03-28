use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

use crate::{
    api, browser,
    components::{
        CardBrand, GlassPanel, Icon, Skeleton, StatusChip, detect_brand, format_card_number,
        strip_non_digits,
    },
    i18n::{self, t},
    state::{ThemeMode, ThemeName, use_auth_state, use_theme_state},
    types::{CardInfo, ProviderInfo},
};

use super::ProtectedPage;

fn provider_status(status: &str) -> (&'static str, &'static str) {
    match status {
        "valid" => (t("provider.status_valid"), "success"),
        "invalid" => (t("provider.status_invalid"), "error"),
        "disabled" => (t("provider.status_disabled"), "neutral"),
        _ => (t("provider.status_unverified"), "warning"),
    }
}

#[component]
fn SectionFrame(
    #[prop(into)] title: String,
    #[prop(into)] icon: String,
    #[prop(default = true)] open: bool,
    children: Children,
) -> impl IntoView {
    view! {
        <details class="lg-settings-section" open=open>
            <summary class="lg-settings-section__summary">
                <div class="lg-settings-section__summary-main">
                    <span class="lg-settings-section__icon">
                        <Icon name=icon class="h-4 w-4" />
                    </span>
                    <span class="text-sm font-semibold">{title}</span>
                </div>
                <Icon name="chevron-down" class="h-4 w-4 lg-settings-section__chevron" />
            </summary>
            <div class="lg-settings-section__content">{children()}</div>
        </details>
    }
}

#[component]
fn ProviderRow(cred: ProviderInfo) -> impl IntoView {
    let provider_name = cred.provider.clone();
    let (status_label, status_variant) = provider_status(&cred.status);
    let delete_action = ServerAction::<api::DeleteProviderSubmit>::new();

    view! {
        <div class="lg-settings-row">
            <div class="space-y-1">
                <div class="flex flex-wrap items-center gap-2">
                    <strong>{cred.provider.clone()}</strong>
                    <StatusChip label=status_label variant=status_variant />
                </div>
                <p class="text-sm" style="color: var(--lg-text-secondary);">{cred.login_id.clone()}</p>
                {cred.last_verified_at.map(|value| {
                    view! {
                        <p class="text-xs" style="color: var(--lg-text-tertiary);">{format!("Verified {value}")}</p>
                    }
                })}
            </div>

            <ActionForm action=delete_action attr:class="inline">
                <input type="hidden" name="provider" value=provider_name />
                <button type="submit" class="lg-btn-secondary text-xs">
                    {t("provider.remove")}
                </button>
            </ActionForm>
        </div>
    }
}

#[component]
fn ProviderSetupRow(provider: &'static str) -> impl IntoView {
    let add_action = ServerAction::<api::AddProviderSubmit>::new();

    view! {
        <div class="lg-settings-row lg-settings-row--stacked">
            <div class="space-y-1">
                <strong>{provider}</strong>
                <p class="text-sm" style="color: var(--lg-text-secondary);">{t("provider.not_configured")}</p>
            </div>

            <ActionForm action=add_action attr:class="lg-settings-form lg-settings-form--provider">
                <input type="hidden" name="provider" value=provider />

                <label class="lg-field">
                    <span>{t("provider.login_id")}</span>
                    <input type="text" name="login_id" required />
                </label>

                <label class="lg-field">
                    <span>{t("provider.password")}</span>
                    <input type="password" name="password" required />
                </label>

                <button type="submit" class="lg-btn-primary">
                    {t("provider.verify_save")}
                </button>
            </ActionForm>
        </div>
    }
}

#[island]
fn CardAddIsland(
    label_label: String,
    number_label: String,
    password_label: String,
    birthday_label: String,
    expiry_label: String,
    card_type_label: String,
    add_label: String,
    loading_label: String,
    credit_label: String,
    debit_label: String,
    error_fallback: String,
) -> impl IntoView {
    let label = RwSignal::new(String::new());
    let card_number_raw = RwSignal::new(String::new());
    let card_password = RwSignal::new(String::new());
    let birthday = RwSignal::new(String::new());
    let expire_date = RwSignal::new(String::new());
    let card_type = RwSignal::new("J".to_string());
    let submit_pending = RwSignal::new(false);
    let error_msg = RwSignal::new(None::<String>);

    let brand = Memo::new(move |_| detect_brand(&card_number_raw.get()));
    let formatted_display = Memo::new(move |_| format_card_number(&card_number_raw.get()));
    let form_valid = move || {
        let digits = card_number_raw.get();
        (15..=16).contains(&digits.len())
            && card_password.get().len() == 2
            && birthday.get().len() == 6
            && expire_date.get().len() == 4
    };

    let on_submit = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        if submit_pending.get() || !form_valid() {
            return;
        }

        submit_pending.set(true);
        error_msg.set(None);
        let fallback = error_fallback.clone();

        wasm_bindgen_futures::spawn_local(async move {
            match browser::submit_card(
                &label.get_untracked(),
                &card_number_raw.get_untracked(),
                &card_password.get_untracked(),
                &birthday.get_untracked(),
                &expire_date.get_untracked(),
                &card_type.get_untracked(),
            )
            .await
            {
                Ok(()) => {
                    browser::redirect_to("/settings?notice=Card%20saved.");
                }
                Err(error) => {
                    submit_pending.set(false);
                    let message = if error.trim().is_empty() {
                        fallback.clone()
                    } else {
                        error
                    };
                    error_msg.set(Some(message));
                }
            }
        });
    };

    view! {
        <div class="lg-settings-card-form">
            {move || error_msg.get().map(|message| {
                view! { <div class="lg-inline-alert lg-inline-alert--error">{message}</div> }
            })}

            <form class="lg-settings-form" on:submit=on_submit>
                <label class="lg-field">
                    <span>{label_label.clone()}</span>
                    <input
                        type="text"
                        placeholder="My Card"
                        prop:value=move || label.get()
                        on:input=move |event| label.set(event_target_value(&event))
                    />
                </label>

                <label class="lg-field">
                    <span>{number_label.clone()}</span>
                    <div class="lg-card-input">
                        <input
                            type="text"
                            inputmode="numeric"
                            maxlength="19"
                            placeholder="0000 0000 0000 0000"
                            prop:value=move || formatted_display.get()
                            on:input=move |event| {
                                let raw = strip_non_digits(&event_target_value(&event));
                                let capped = if raw.len() > 16 {
                                    raw[..16].to_string()
                                } else {
                                    raw
                                };
                                card_number_raw.set(capped);
                            }
                        />
                        <CardBrand kind=brand.get() />
                    </div>
                </label>

                <div class="grid gap-3 sm:grid-cols-3">
                    <label class="lg-field">
                        <span>{password_label.clone()}</span>
                        <input
                            type="password"
                            inputmode="numeric"
                            maxlength="2"
                            prop:value=move || card_password.get()
                            on:input=move |event| {
                                let raw = strip_non_digits(&event_target_value(&event));
                                let capped = if raw.len() > 2 {
                                    raw[..2].to_string()
                                } else {
                                    raw
                                };
                                card_password.set(capped);
                            }
                        />
                    </label>

                    <label class="lg-field">
                        <span>{birthday_label.clone()}</span>
                        <input
                            type="text"
                            inputmode="numeric"
                            maxlength="6"
                            prop:value=move || birthday.get()
                            on:input=move |event| {
                                let raw = strip_non_digits(&event_target_value(&event));
                                let capped = if raw.len() > 6 {
                                    raw[..6].to_string()
                                } else {
                                    raw
                                };
                                birthday.set(capped);
                            }
                        />
                    </label>

                    <label class="lg-field">
                        <span>{expiry_label.clone()}</span>
                        <input
                            type="text"
                            inputmode="numeric"
                            maxlength="4"
                            prop:value=move || expire_date.get()
                            on:input=move |event| {
                                let raw = strip_non_digits(&event_target_value(&event));
                                let capped = if raw.len() > 4 {
                                    raw[..4].to_string()
                                } else {
                                    raw
                                };
                                expire_date.set(capped);
                            }
                        />
                    </label>
                </div>

                <label class="lg-field">
                    <span>{card_type_label.clone()}</span>
                    <select
                        prop:value=move || card_type.get()
                        on:change=move |event| card_type.set(event_target_value(&event))
                    >
                        <option value="J">{credit_label.clone()}</option>
                        <option value="S">{debit_label.clone()}</option>
                    </select>
                </label>

                <button
                    type="submit"
                    class="lg-btn-primary"
                    disabled=move || submit_pending.get() || !form_valid()
                >
                    {move || if submit_pending.get() { loading_label.clone() } else { add_label.clone() }}
                </button>
            </form>
        </div>
    }
}

#[component]
fn CardRow(card: CardInfo) -> impl IntoView {
    let update_action = ServerAction::<api::UpdateCardSubmit>::new();
    let delete_action = ServerAction::<api::DeleteCardSubmit>::new();

    view! {
        <div class="lg-settings-row lg-settings-row--stacked">
            <div class="flex flex-wrap items-start justify-between gap-3">
                <div class="space-y-2">
                    <div class="flex flex-wrap items-center gap-2">
                        <CardBrand label=card.card_type_name.clone() />
                        <p class="text-sm" style="color: var(--lg-text-secondary);">
                            {format!("•••• {}", card.last_four)}
                        </p>
                    </div>
                    <strong>{card.label.clone()}</strong>
                </div>

                <div class="flex flex-wrap items-center gap-2">
                    <ActionForm action=update_action attr:class="flex flex-wrap items-center gap-2">
                        <input type="hidden" name="card_id" value=card.id.to_string() />
                        <input
                            type="text"
                            name="label"
                            value=card.label.clone()
                            class="lg-select min-w-48"
                        />
                        <button type="submit" class="lg-btn-secondary text-xs">
                            "Save label"
                        </button>
                    </ActionForm>

                    <ActionForm action=delete_action attr:class="inline">
                        <input type="hidden" name="card_id" value=card.id.to_string() />
                        <button type="submit" class="lg-btn-secondary text-xs">
                            {t("provider.remove")}
                        </button>
                    </ActionForm>
                </div>
            </div>
        </div>
    }
}

#[component]
fn AppearanceSection() -> impl IntoView {
    let theme_state = use_theme_state();
    let appearance_action = ServerAction::<api::SetAppearance>::new();
    let current_locale = i18n::current_locale().code().to_string();

    view! {
        <SectionFrame title=t("settings.section_appearance") icon="appearance" open=false>
            <div class="lg-settings-grid">
                <div class="space-y-3">
                    <p class="lg-route-kicker">{t("settings.theme")}</p>
                    <div class="grid gap-3 sm:grid-cols-2">
                        <ActionForm action=appearance_action attr:class="contents">
                            <input type="hidden" name="theme" value=ThemeName::Glass.as_str() />
                            <button
                                type="submit"
                                class=move || {
                                    if theme_state.theme.get() == ThemeName::Glass {
                                        "lg-theme-button lg-theme-button--active"
                                    } else {
                                        "lg-theme-button"
                                    }
                                }
                            >
                                <strong>{t("settings.theme_current")}</strong>
                                <span>"Reflective charcoal glass with bright accents."</span>
                            </button>
                        </ActionForm>

                        <ActionForm action=appearance_action attr:class="contents">
                            <input type="hidden" name="theme" value=ThemeName::ClearSky.as_str() />
                            <button
                                type="submit"
                                class=move || {
                                    if theme_state.theme.get() == ThemeName::ClearSky {
                                        "lg-theme-button lg-theme-button--active"
                                    } else {
                                        "lg-theme-button"
                                    }
                                }
                            >
                                <strong>{t("settings.theme_transit_slate")}</strong>
                                <span>"Cool blue surfaces with softer daylight contrast."</span>
                            </button>
                        </ActionForm>
                    </div>
                </div>

                <div class="space-y-3">
                    <p class="lg-route-kicker">{t("settings.dark_mode")}</p>
                    <div class="grid gap-3 sm:grid-cols-2">
                        <ActionForm action=appearance_action attr:class="contents">
                            <input type="hidden" name="mode" value=ThemeMode::Light.as_str() />
                            <button
                                type="submit"
                                class=move || {
                                    if theme_state.mode.get() == ThemeMode::Light {
                                        "lg-theme-button lg-theme-button--active"
                                    } else {
                                        "lg-theme-button"
                                    }
                                }
                            >
                                <strong>{t("settings.light_mode")}</strong>
                                <span>"Higher contrast cards with daylight backgrounds."</span>
                            </button>
                        </ActionForm>

                        <ActionForm action=appearance_action attr:class="contents">
                            <input type="hidden" name="mode" value=ThemeMode::Dark.as_str() />
                            <button
                                type="submit"
                                class=move || {
                                    if theme_state.mode.get() == ThemeMode::Dark {
                                        "lg-theme-button lg-theme-button--active"
                                    } else {
                                        "lg-theme-button"
                                    }
                                }
                            >
                                <strong>{t("settings.dark_mode")}</strong>
                                <span>"Low-glare panels for night bookings."</span>
                            </button>
                        </ActionForm>
                    </div>
                </div>

                <ActionForm action=appearance_action attr:class="lg-field">
                    <span>{t("settings.language")}</span>
                    <select name="locale">
                        {i18n::locale_options().iter().map(|(locale_value, label)| {
                            let selected = locale_value.code() == current_locale;
                            view! {
                                <option value=locale_value.code() selected=selected>{label.to_string()}</option>
                            }
                        }).collect::<Vec<_>>()}
                    </select>
                    <button type="submit" class="lg-btn-secondary text-xs">
                        "Apply language"
                    </button>
                </ActionForm>
            </div>
        </SectionFrame>
    }
}

#[component]
pub fn SettingsPage() -> impl IntoView {
    let auth = use_auth_state();
    let query = use_query_map();
    let providers = Resource::new(|| (), |_| api::list_providers());
    let cards = Resource::new(|| (), |_| api::list_cards());

    let logout_action = ServerAction::<api::Logout>::new();
    let notice = move || query.get().get("notice");
    let error = move || query.get().get("error");

    view! {
        <ProtectedPage>
            <section class="mx-auto flex w-full max-w-5xl flex-col gap-6 px-1 md:px-4">
                <section class="lg-page-card">
                    <div class="flex flex-col gap-4 lg:flex-row lg:items-end lg:justify-between">
                        <div class="space-y-2">
                            <p class="lg-route-kicker">{t("settings.title")}</p>
                            <h1 class="text-3xl font-semibold tracking-tight">{t("settings.title")}</h1>
                            <p class="text-sm" style="color: var(--lg-text-secondary);">
                                "Manage provider credentials, saved cards, interface preferences, and your session from one protected hub."
                            </p>
                        </div>

                        <GlassPanel class="lg-settings-user" hover=true>
                            <div class="flex items-center gap-3">
                                <span class="lg-settings-section__icon">
                                    <Icon name="user" class="h-4 w-4" />
                                </span>
                                <div class="space-y-1">
                                    <strong>{move || auth.user.get().map(|user| user.display_name).unwrap_or_else(|| "Bominal".to_string())}</strong>
                                    <p class="text-sm" style="color: var(--lg-text-secondary);">
                                        {move || auth.user.get().map(|user| user.email).unwrap_or_else(|| "Signed in".to_string())}
                                    </p>
                                </div>
                            </div>
                        </GlassPanel>
                    </div>
                </section>

                {move || notice().map(|message| {
                    view! {
                        <div class="lg-inline-alert lg-inline-alert--success">
                            {message}
                        </div>
                    }
                })}

                {move || error().map(|message| {
                    view! {
                        <div class="lg-inline-alert lg-inline-alert--error">
                            {message}
                        </div>
                    }
                })}

                <section class="lg-settings-stack">
                    <SectionFrame title=t("settings.section_provider") icon="train">
                        <Suspense fallback=move || view! {
                            <div class="space-y-3">
                                <Skeleton height="h-16" />
                                <Skeleton height="h-16" />
                            </div>
                        }>
                            {move || providers.get().map(|result| match result {
                                Ok(items) => {
                                    let has_srt = items.iter().any(|item| item.provider == "SRT");
                                    let has_ktx = items.iter().any(|item| item.provider == "KTX");
                                    view! {
                                        <div class="space-y-3">
                                            {items.into_iter().map(|item| {
                                                view! {
                                                    <ProviderRow cred=item />
                                                }
                                            }).collect::<Vec<_>>()}

                                            {(!has_srt).then(|| view! {
                                                <ProviderSetupRow provider="SRT" />
                                            })}
                                            {(!has_ktx).then(|| view! {
                                                <ProviderSetupRow provider="KTX" />
                                            })}
                                        </div>
                                    }.into_any()
                                }
                                Err(error) => {
                                    view! {
                                        <div class="lg-inline-alert lg-inline-alert--error">
                                            {error.to_string()}
                                        </div>
                                    }.into_any()
                                }
                            })}
                        </Suspense>
                    </SectionFrame>

                    <SectionFrame title=t("settings.section_payment") icon="payment">
                        <div class="space-y-4">
                            <div class="space-y-1">
                                <strong>{t("settings.section_payment")}</strong>
                                <p class="text-sm" style="color: var(--lg-text-secondary);">
                                    "Add encrypted payment cards for auto-pay tasks and reservation checkout."
                                </p>
                            </div>

                            <CardAddIsland
                                label_label=t("payment.card_label").to_string()
                                number_label=t("payment.card_number").to_string()
                                password_label=t("payment.card_password").to_string()
                                birthday_label=t("payment.birthday").to_string()
                                expiry_label=t("payment.expiry").to_string()
                                card_type_label=t("settings.section_payment").to_string()
                                add_label=t("payment.add_card").to_string()
                                loading_label=t("common.loading").to_string()
                                credit_label=t("payment.credit_card").to_string()
                                debit_label=t("payment.debit_card").to_string()
                                error_fallback="Card submission failed".to_string()
                            />

                            <Suspense fallback=move || view! {
                                <div class="space-y-3">
                                    <Skeleton height="h-16" />
                                    <Skeleton height="h-16" />
                                </div>
                            }>
                                {move || cards.get().map(|result| match result {
                                    Ok(items) if items.is_empty() => {
                                        view! {
                                            <div class="lg-empty-state">
                                                <p>{t("payment.no_cards")}</p>
                                            </div>
                                        }.into_any()
                                    }
                                    Ok(items) => {
                                        view! {
                                            <div class="space-y-3">
                                                {items.into_iter().map(|card| {
                                                    view! {
                                                        <CardRow card=card />
                                                    }
                                                }).collect::<Vec<_>>()}
                                            </div>
                                        }.into_any()
                                    }
                                    Err(error) => {
                                        view! {
                                            <div class="lg-inline-alert lg-inline-alert--error">
                                                {error.to_string()}
                                            </div>
                                        }.into_any()
                                    }
                                })}
                            </Suspense>
                        </div>
                    </SectionFrame>

                    <AppearanceSection />

                    <GlassPanel class="lg-settings-logout" hover=true>
                        <div class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
                            <div class="space-y-1">
                                <div class="flex items-center gap-2">
                                    <Icon name="logout" class="h-4 w-4" />
                                    <strong>{t("auth.logout")}</strong>
                                </div>
                                <p class="text-sm" style="color: var(--lg-text-secondary);">
                                    "Sign out of Bominal Train on this device."
                                </p>
                            </div>

                            <ActionForm action=logout_action attr:class="inline">
                                <button type="submit" class="lg-btn-secondary">
                                    {t("auth.logout")}
                                </button>
                            </ActionForm>
                        </div>
                    </GlassPanel>
                </section>
            </section>
        </ProtectedPage>
    }
}
