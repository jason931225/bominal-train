//! Typed Leptos server functions that proxy the existing Axum `/api/...` routes.

pub mod passkey;

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::types::{
    AuthResponse, CardInfo, CreateTaskInput, ProviderInfo, ReservationInfo, StationInfo,
    SuggestResult, TaskInfo, TicketInfo, TrainInfo, UpdateTaskInput,
};

#[derive(Clone, Debug)]
pub struct ApiBaseUrl(pub String);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AddProviderInput {
    pub provider: String,
    pub login_id: String,
    pub password: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AddCardInput {
    pub label: Option<String>,
    pub card_number: String,
    pub card_password: String,
    pub birthday: String,
    pub expire_date: String,
    pub expire_date_yymm: Option<String>,
    pub last_four: String,
    pub card_type: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateCardInput {
    pub label: String,
}

#[cfg(feature = "ssr")]
#[derive(Clone, Debug, Serialize, Deserialize)]
struct LoginInput {
    email: String,
    password: String,
}

#[cfg(feature = "ssr")]
#[derive(Clone, Debug, Serialize, Deserialize)]
struct RegisterInput {
    email: String,
    password: String,
    display_name: String,
}

#[cfg(feature = "ssr")]
#[derive(Clone, Debug, Serialize, Deserialize)]
struct ForgotPasswordInput {
    email: String,
}

#[cfg(feature = "ssr")]
#[derive(Clone, Debug, Serialize, Deserialize)]
struct ResetPasswordInput {
    token: String,
    new_password: String,
}

#[cfg(feature = "ssr")]
#[derive(Clone, Debug, Serialize, Deserialize)]
struct VerifyEmailInput {
    token: String,
}

#[cfg(feature = "ssr")]
#[derive(Clone, Debug, Serialize, Deserialize)]
struct SearchInput {
    provider: String,
    departure: String,
    arrival: String,
    date: Option<String>,
    time: Option<String>,
}

#[cfg(feature = "ssr")]
#[derive(Clone, Debug, Serialize, Deserialize)]
struct ProviderOnlyInput {
    provider: String,
}

#[cfg(feature = "ssr")]
#[derive(Clone, Debug, Serialize, Deserialize)]
struct PayReservationInput {
    provider: String,
    card_id: String,
}

#[cfg(feature = "ssr")]
mod ssr {
    use super::*;
    use reqwest::Method;
    use serde::de::DeserializeOwned;

    pub(super) fn api_base_url() -> String {
        use_context::<ApiBaseUrl>()
            .map(|value| value.0)
            .or_else(|| std::env::var("APP_BASE_URL").ok())
            .unwrap_or_else(|| "http://127.0.0.1:3000".to_string())
    }

    fn forwarded_cookie_header() -> Option<String> {
        use_context::<axum::http::request::Parts>().and_then(|parts| {
            parts
                .headers
                .get(axum::http::header::COOKIE)
                .and_then(|value| value.to_str().ok())
                .map(str::to_string)
        })
    }

    pub(super) async fn send_request<B>(
        method: Method,
        path: &str,
        body: Option<&B>,
    ) -> Result<reqwest::Response, ServerFnError>
    where
        B: Serialize + ?Sized,
    {
        let client = reqwest::Client::new();
        let url = format!("{}{}", api_base_url(), path);
        let mut request = client.request(method, url);

        if let Some(cookie) = forwarded_cookie_header() {
            request = request.header(reqwest::header::COOKIE, cookie);
        }

        if let Some(body) = body {
            request = request.json(body);
        }

        let response = request
            .send()
            .await
            .map_err(|error| ServerFnError::new(format!("API proxy request failed: {error}")))?;

        if let Some(response_options) = use_context::<leptos_axum::ResponseOptions>() {
            for value in response.headers().get_all(reqwest::header::SET_COOKIE) {
                if let Ok(value) = axum::http::HeaderValue::from_bytes(value.as_bytes()) {
                    response_options.append_header(axum::http::header::SET_COOKIE, value);
                }
            }
        }

        Ok(response)
    }

    pub(super) async fn request_json<T, B>(
        method: Method,
        path: &str,
        body: Option<&B>,
    ) -> Result<T, ServerFnError>
    where
        T: DeserializeOwned,
        B: Serialize + ?Sized,
    {
        let response = send_request(method, path, body).await?;
        if !response.status().is_success() {
            let status = response.status();
            let message = response.text().await.unwrap_or_default();
            return Err(ServerFnError::new(format!(
                "API proxy failed: {status} {message}"
            )));
        }

        response
            .json::<T>()
            .await
            .map_err(|error| ServerFnError::new(format!("API proxy decode failed: {error}")))
    }

    pub(super) async fn request_no_content<B>(
        method: Method,
        path: &str,
        body: Option<&B>,
    ) -> Result<(), ServerFnError>
    where
        B: Serialize + ?Sized,
    {
        let response = send_request(method, path, body).await?;
        if response.status().is_success() {
            return Ok(());
        }

        let status = response.status();
        let message = response.text().await.unwrap_or_default();
        Err(ServerFnError::new(format!(
            "API proxy failed: {status} {message}"
        )))
    }
}

#[cfg(feature = "ssr")]
fn redirect_with_query(path: &str, params: &[(&str, String)]) {
    let mut target = path.to_string();

    if !params.is_empty() {
        let mut serializer = url::form_urlencoded::Serializer::new(String::new());
        for (key, value) in params {
            serializer.append_pair(key, value);
        }

        let query = serializer.finish();
        if !query.is_empty() {
            target.push('?');
            target.push_str(&query);
        }
    }

    leptos_axum::redirect(&target);
}

#[cfg(feature = "ssr")]
fn cleaned_error_message(error: &ServerFnError) -> String {
    let message = error.to_string();
    let cleaned = message
        .strip_prefix("error running server function: ")
        .unwrap_or(&message)
        .trim();

    if cleaned.is_empty() {
        "Something went wrong".to_string()
    } else {
        cleaned.to_string()
    }
}

#[cfg(feature = "ssr")]
fn redirect_with_error(path: &str, error: &ServerFnError) {
    redirect_with_query(path, &[("error", cleaned_error_message(error))]);
}

#[cfg(feature = "ssr")]
fn append_cookie(name: &str, value: &str) -> Result<(), ServerFnError> {
    let response = use_context::<leptos_axum::ResponseOptions>()
        .ok_or_else(|| ServerFnError::new("ResponseOptions not available"))?;

    let cookie = format!("{name}={value}; Path=/; Max-Age=31536000; SameSite=Lax");
    let value = axum::http::HeaderValue::from_str(&cookie)
        .map_err(|_| ServerFnError::new("Invalid cookie value"))?;
    response.append_header(axum::http::header::SET_COOKIE, value);

    Ok(())
}

#[server(prefix = "/sfn")]
pub async fn get_me() -> Result<Option<AuthResponse>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let response = ssr::send_request::<()>(reqwest::Method::GET, "/api/auth/me", None).await?;
        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Ok(None);
        }
        if !response.status().is_success() {
            let status = response.status();
            let message = response.text().await.unwrap_or_default();
            return Err(ServerFnError::new(format!(
                "API proxy failed: {status} {message}"
            )));
        }
        let user = response
            .json::<AuthResponse>()
            .await
            .map_err(|error| ServerFnError::new(format!("API proxy decode failed: {error}")))?;
        return Ok(Some(user));
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new("get_me is only available during SSR"))
}

#[server(prefix = "/sfn")]
pub async fn login(email: String, password: String) -> Result<AuthResponse, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        return ssr::request_json(
            reqwest::Method::POST,
            "/api/auth/login",
            Some(&LoginInput { email, password }),
        )
        .await;
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new("login is only available during SSR"))
}

#[server(prefix = "/sfn")]
pub async fn login_submit(email: String, password: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        match login(email, password).await {
            Ok(_) => {
                leptos_axum::redirect("/home");
                return Ok(());
            }
            Err(error) => {
                redirect_with_error("/auth/login", &error);
                return Ok(());
            }
        }
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new(
        "login_submit is only available during SSR",
    ))
}

#[server(prefix = "/sfn")]
pub async fn register(
    email: String,
    password: String,
    display_name: String,
) -> Result<AuthResponse, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        return ssr::request_json(
            reqwest::Method::POST,
            "/api/auth/register",
            Some(&RegisterInput {
                email,
                password,
                display_name,
            }),
        )
        .await;
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new("register is only available during SSR"))
}

#[server(prefix = "/sfn")]
pub async fn register_submit(
    email: String,
    password: String,
    display_name: String,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        match register(email, password, display_name).await {
            Ok(_) => {
                leptos_axum::redirect("/auth/verify");
                return Ok(());
            }
            Err(error) => {
                redirect_with_error("/auth/signup", &error);
                return Ok(());
            }
        }
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new(
        "register_submit is only available during SSR",
    ))
}

#[server(prefix = "/sfn")]
pub async fn forgot_password(email: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        return ssr::request_no_content(
            reqwest::Method::POST,
            "/api/auth/forgot-password",
            Some(&ForgotPasswordInput { email }),
        )
        .await;
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new(
        "forgot_password is only available during SSR",
    ))
}

#[server(prefix = "/sfn")]
pub async fn forgot_password_submit(email: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        match forgot_password(email).await {
            Ok(()) => {
                redirect_with_query("/auth/forgot", &[("sent", "1".to_string())]);
                return Ok(());
            }
            Err(error) => {
                redirect_with_error("/auth/forgot", &error);
                return Ok(());
            }
        }
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new(
        "forgot_password_submit is only available during SSR",
    ))
}

#[server(prefix = "/sfn")]
pub async fn reset_password(token: String, new_password: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        return ssr::request_no_content(
            reqwest::Method::POST,
            "/api/auth/reset-password",
            Some(&ResetPasswordInput {
                token,
                new_password,
            }),
        )
        .await;
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new(
        "reset_password is only available during SSR",
    ))
}

#[server(prefix = "/sfn")]
pub async fn reset_password_submit(
    token: String,
    new_password: String,
    confirm_password: String,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        if new_password != confirm_password {
            redirect_with_query(
                "/reset-password",
                &[
                    ("token", token),
                    ("error", "Passwords do not match.".to_string()),
                ],
            );
            return Ok(());
        }

        let token_for_error = token.clone();
        match reset_password(token, new_password).await {
            Ok(()) => {
                redirect_with_query("/reset-password", &[("done", "1".to_string())]);
                return Ok(());
            }
            Err(error) => {
                redirect_with_query(
                    "/reset-password",
                    &[
                        ("token", token_for_error),
                        ("error", cleaned_error_message(&error)),
                    ],
                );
                return Ok(());
            }
        }
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new(
        "reset_password_submit is only available during SSR",
    ))
}

#[server(prefix = "/sfn")]
pub async fn verify_email(token: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        return ssr::request_no_content(
            reqwest::Method::POST,
            "/api/auth/verify-email",
            Some(&VerifyEmailInput { token }),
        )
        .await;
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new(
        "verify_email is only available during SSR",
    ))
}

#[server(prefix = "/sfn")]
pub async fn resend_verification() -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        return ssr::request_no_content::<()>(
            reqwest::Method::POST,
            "/api/auth/resend-verification",
            None,
        )
        .await;
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new(
        "resend_verification is only available during SSR",
    ))
}

#[server(prefix = "/sfn")]
pub async fn resend_verification_submit() -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        match resend_verification().await {
            Ok(()) => {
                redirect_with_query(
                    "/auth/verify",
                    &[("notice", "Verification email resent.".to_string())],
                );
                return Ok(());
            }
            Err(error) => {
                redirect_with_error("/auth/verify", &error);
                return Ok(());
            }
        }
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new(
        "resend_verification_submit is only available during SSR",
    ))
}

#[server(prefix = "/sfn")]
pub async fn logout() -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        ssr::request_no_content::<()>(reqwest::Method::POST, "/api/auth/logout", None).await?;
        leptos_axum::redirect("/auth");
        return Ok(());
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new("logout is only available during SSR"))
}

#[server(prefix = "/sfn")]
pub async fn list_providers() -> Result<Vec<ProviderInfo>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        return ssr::request_json::<Vec<ProviderInfo>, ()>(
            reqwest::Method::GET,
            "/api/providers",
            None,
        )
        .await;
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new(
        "list_providers is only available during SSR",
    ))
}

#[server(prefix = "/sfn")]
pub async fn add_provider(
    provider: String,
    login_id: String,
    password: String,
) -> Result<ProviderInfo, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        return ssr::request_json(
            reqwest::Method::POST,
            "/api/providers",
            Some(&AddProviderInput {
                provider,
                login_id,
                password,
            }),
        )
        .await;
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new(
        "add_provider is only available during SSR",
    ))
}

#[server(prefix = "/sfn")]
pub async fn add_provider_submit(
    provider: String,
    login_id: String,
    password: String,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        match add_provider(provider.clone(), login_id, password).await {
            Ok(_) => {
                redirect_with_query("/settings", &[("notice", format!("{provider} saved."))]);
                return Ok(());
            }
            Err(error) => {
                redirect_with_query(
                    "/settings",
                    &[(
                        "error",
                        format!("{provider}: {}", cleaned_error_message(&error)),
                    )],
                );
                return Ok(());
            }
        }
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new(
        "add_provider_submit is only available during SSR",
    ))
}

#[server(prefix = "/sfn")]
pub async fn delete_provider(provider: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        return ssr::request_no_content::<()>(
            reqwest::Method::DELETE,
            &format!("/api/providers/{provider}"),
            None,
        )
        .await;
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new(
        "delete_provider is only available during SSR",
    ))
}

#[server(prefix = "/sfn")]
pub async fn delete_provider_submit(provider: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        match delete_provider(provider.clone()).await {
            Ok(()) => {
                redirect_with_query("/settings", &[("notice", format!("{provider} removed."))]);
                return Ok(());
            }
            Err(error) => {
                redirect_with_query(
                    "/settings",
                    &[(
                        "error",
                        format!("{provider}: {}", cleaned_error_message(&error)),
                    )],
                );
                return Ok(());
            }
        }
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new(
        "delete_provider_submit is only available during SSR",
    ))
}

#[server(prefix = "/sfn")]
pub async fn list_cards() -> Result<Vec<CardInfo>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        return ssr::request_json::<Vec<CardInfo>, ()>(reqwest::Method::GET, "/api/cards", None)
            .await;
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new(
        "list_cards is only available during SSR",
    ))
}

#[server(prefix = "/sfn")]
pub async fn add_card(input: AddCardInput) -> Result<CardInfo, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        return ssr::request_json(reqwest::Method::POST, "/api/cards", Some(&input)).await;
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new("add_card is only available during SSR"))
}

#[server(prefix = "/sfn")]
pub async fn update_card(card_id: String, label: String) -> Result<CardInfo, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        return ssr::request_json(
            reqwest::Method::PATCH,
            &format!("/api/cards/{card_id}"),
            Some(&UpdateCardInput { label }),
        )
        .await;
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new(
        "update_card is only available during SSR",
    ))
}

#[server(prefix = "/sfn")]
pub async fn update_card_submit(card_id: String, label: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        match update_card(card_id, label).await {
            Ok(_) => {
                redirect_with_query("/settings", &[("notice", "Card updated.".to_string())]);
                return Ok(());
            }
            Err(error) => {
                redirect_with_error("/settings", &error);
                return Ok(());
            }
        }
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new(
        "update_card_submit is only available during SSR",
    ))
}

#[server(prefix = "/sfn")]
pub async fn delete_card(card_id: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        return ssr::request_no_content::<()>(
            reqwest::Method::DELETE,
            &format!("/api/cards/{card_id}"),
            None,
        )
        .await;
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new(
        "delete_card is only available during SSR",
    ))
}

#[server(prefix = "/sfn")]
pub async fn delete_card_submit(card_id: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        match delete_card(card_id).await {
            Ok(()) => {
                redirect_with_query("/settings", &[("notice", "Card removed.".to_string())]);
                return Ok(());
            }
            Err(error) => {
                redirect_with_error("/settings", &error);
                return Ok(());
            }
        }
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new(
        "delete_card_submit is only available during SSR",
    ))
}

#[server(prefix = "/sfn")]
pub async fn set_appearance(
    theme: Option<String>,
    mode: Option<String>,
    locale: Option<String>,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        if let Some(theme) = theme
            .as_deref()
            .and_then(crate::state::ThemeName::parse)
            .map(crate::state::ThemeName::as_str)
        {
            append_cookie(crate::state::THEME_COOKIE, theme)?;
        }

        if let Some(mode) = mode
            .as_deref()
            .and_then(crate::state::ThemeMode::parse)
            .map(crate::state::ThemeMode::as_str)
        {
            append_cookie(crate::state::MODE_COOKIE, mode)?;
        }

        if let Some(locale) = locale.as_deref() {
            let locale = bominal_domain::i18n::Locale::from_code(locale);
            append_cookie(crate::i18n::LOCALE_COOKIE, locale.code())?;
        }

        redirect_with_query(
            "/settings",
            &[("notice", "Appearance updated.".to_string())],
        );
        return Ok(());
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new(
        "set_appearance is only available during SSR",
    ))
}

#[server(prefix = "/sfn")]
pub async fn list_stations(provider: String) -> Result<Vec<StationInfo>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        return ssr::request_json::<Vec<StationInfo>, ()>(
            reqwest::Method::GET,
            &format!("/api/stations/{provider}"),
            None,
        )
        .await;
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new(
        "list_stations is only available during SSR",
    ))
}

#[server(prefix = "/sfn")]
pub async fn search_trains(
    provider: String,
    departure: String,
    arrival: String,
    date: Option<String>,
    time: Option<String>,
) -> Result<Vec<TrainInfo>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        return ssr::request_json(
            reqwest::Method::POST,
            "/api/search",
            Some(&SearchInput {
                provider,
                departure,
                arrival,
                date,
                time,
            }),
        )
        .await;
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new(
        "search_trains is only available during SSR",
    ))
}

#[server(prefix = "/sfn")]
pub async fn suggest_stations(
    provider: String,
    query: String,
    mode: Option<String>,
) -> Result<SuggestResult, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let query_string = {
            let mut serializer = url::form_urlencoded::Serializer::new(String::new());
            serializer.append_pair("q", &query);
            if let Some(mode) = mode.as_deref() {
                serializer.append_pair("mode", mode);
            }
            serializer.finish()
        };
        return ssr::request_json::<SuggestResult, ()>(
            reqwest::Method::GET,
            &format!("/api/stations/{provider}/suggest?{query_string}"),
            None,
        )
        .await;
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new(
        "suggest_stations is only available during SSR",
    ))
}

#[server(prefix = "/sfn")]
pub async fn list_tasks() -> Result<Vec<TaskInfo>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        return ssr::request_json::<Vec<TaskInfo>, ()>(reqwest::Method::GET, "/api/tasks", None)
            .await;
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new(
        "list_tasks is only available during SSR",
    ))
}

#[server(prefix = "/sfn")]
pub async fn get_task(task_id: String) -> Result<TaskInfo, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        return ssr::request_json::<TaskInfo, ()>(
            reqwest::Method::GET,
            &format!("/api/tasks/{task_id}"),
            None,
        )
        .await;
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new("get_task is only available during SSR"))
}

#[server(prefix = "/sfn")]
pub async fn create_task(input: CreateTaskInput) -> Result<TaskInfo, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        return ssr::request_json(reqwest::Method::POST, "/api/tasks", Some(&input)).await;
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new(
        "create_task is only available during SSR",
    ))
}

#[server(prefix = "/sfn")]
pub async fn update_task(
    task_id: String,
    input: UpdateTaskInput,
) -> Result<TaskInfo, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        return ssr::request_json(
            reqwest::Method::PATCH,
            &format!("/api/tasks/{task_id}"),
            Some(&input),
        )
        .await;
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new(
        "update_task is only available during SSR",
    ))
}

#[server(prefix = "/sfn")]
pub async fn delete_task(task_id: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        return ssr::request_no_content::<()>(
            reqwest::Method::DELETE,
            &format!("/api/tasks/{task_id}"),
            None,
        )
        .await;
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new(
        "delete_task is only available during SSR",
    ))
}

#[server(prefix = "/sfn")]
pub async fn list_reservations(provider: String) -> Result<Vec<ReservationInfo>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let path = format!("/api/reservations?provider={provider}");
        return ssr::request_json::<Vec<ReservationInfo>, ()>(reqwest::Method::GET, &path, None)
            .await;
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new(
        "list_reservations is only available during SSR",
    ))
}

#[server(prefix = "/sfn")]
pub async fn ticket_detail(
    provider: String,
    reservation_number: String,
) -> Result<Vec<TicketInfo>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let path = format!("/api/reservations/{reservation_number}/tickets?provider={provider}");
        return ssr::request_json::<Vec<TicketInfo>, ()>(reqwest::Method::GET, &path, None).await;
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new(
        "ticket_detail is only available during SSR",
    ))
}

#[server(prefix = "/sfn")]
pub async fn cancel_reservation(
    provider: String,
    reservation_number: String,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        return ssr::request_no_content(
            reqwest::Method::POST,
            &format!("/api/reservations/{reservation_number}/cancel"),
            Some(&ProviderOnlyInput { provider }),
        )
        .await;
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new(
        "cancel_reservation is only available during SSR",
    ))
}

#[server(prefix = "/sfn")]
pub async fn pay_reservation(
    provider: String,
    reservation_number: String,
    card_id: String,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        return ssr::request_no_content(
            reqwest::Method::POST,
            &format!("/api/reservations/{reservation_number}/pay"),
            Some(&PayReservationInput { provider, card_id }),
        )
        .await;
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new(
        "pay_reservation is only available during SSR",
    ))
}

#[server(prefix = "/sfn")]
pub async fn refund_reservation(
    provider: String,
    reservation_number: String,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        return ssr::request_no_content(
            reqwest::Method::POST,
            &format!("/api/reservations/{reservation_number}/refund"),
            Some(&ProviderOnlyInput { provider }),
        )
        .await;
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new(
        "refund_reservation is only available during SSR",
    ))
}
