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

        request
            .send()
            .await
            .map_err(|error| ServerFnError::new(format!("API proxy request failed: {error}")))
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
pub async fn logout() -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        return ssr::request_no_content::<()>(reqwest::Method::POST, "/api/auth/logout", None)
            .await;
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
