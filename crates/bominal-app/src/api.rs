// =============================================================================
// API client — WASM-side HTTP calls to the Axum backend
// =============================================================================

use std::cell::RefCell;
use std::collections::VecDeque;

use serde::{Deserialize, de::DeserializeOwned};
use wasm_bindgen::JsCast;

/// Maximum number of cached entries.
const CACHE_MAX_ENTRIES: usize = 100;
/// Cache TTL in milliseconds (5 minutes).
const CACHE_TTL_MS: f64 = 5.0 * 60.0 * 1000.0;

struct CacheEntry {
    key: String,
    value: String,
    inserted_at: f64,
}

/// Bounded LRU cache with TTL.
struct BoundedCache {
    entries: VecDeque<CacheEntry>,
}

impl BoundedCache {
    fn new() -> Self {
        Self {
            entries: VecDeque::with_capacity(CACHE_MAX_ENTRIES),
        }
    }

    fn now_ms() -> f64 {
        js_sys::Date::now()
    }

    fn get(&mut self, key: &str) -> Option<String> {
        let now = Self::now_ms();
        self.entries
            .retain(|e| (now - e.inserted_at) < CACHE_TTL_MS);
        if let Some(pos) = self.entries.iter().position(|e| e.key == key) {
            let entry = self.entries.remove(pos)?;
            let value = entry.value.clone();
            self.entries.push_back(entry);
            Some(value)
        } else {
            None
        }
    }

    fn insert(&mut self, key: String, value: String) {
        let now = Self::now_ms();
        self.entries.retain(|e| e.key != key);
        while self.entries.len() >= CACHE_MAX_ENTRIES {
            self.entries.pop_front();
        }
        self.entries.push_back(CacheEntry {
            key,
            value,
            inserted_at: now,
        });
    }

    fn invalidate_prefix(&mut self, prefix: &str) {
        self.entries.retain(|e| !e.key.starts_with(prefix));
    }

    fn clear(&mut self) {
        self.entries.clear();
    }
}

thread_local! {
    static CACHE: RefCell<BoundedCache> = RefCell::new(BoundedCache::new());
}

/// Clear cached response for a specific path prefix.
fn invalidate_cache(path: &str) {
    let prefix = path.split('?').next().unwrap_or(path);
    CACHE.with(|cache| {
        cache.borrow_mut().invalidate_prefix(prefix);
    });
}

/// Clear entire cache (used on logout).
pub fn clear_cache() {
    CACHE.with(|cache| cache.borrow_mut().clear());
}

/// Base URL for API calls (same origin).
fn base_url() -> String {
    String::new()
}

/// Read the `csrf_token` cookie value from `document.cookie`.
fn read_csrf_cookie() -> Option<String> {
    let document = web_sys::window()?.document()?;
    let html_doc: &web_sys::HtmlDocument = document.unchecked_ref();
    let cookies = html_doc.cookie().ok()?;
    cookies
        .split(';')
        .map(str::trim)
        .find(|c| c.starts_with("csrf_token="))
        .and_then(|c| c.split_once('='))
        .map(|(_, v)| v.to_string())
}

/// Standard API response envelope.
#[derive(Debug, Clone, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

/// Redirect to sign-in page on 401.
fn redirect_to_signin() {
    clear_cache();
    if let Some(window) = leptos::web_sys::window() {
        let current = window.location().pathname().unwrap_or_default();
        if current != "/auth" {
            let _ = window.location().set_href("/auth");
        }
    }
}

/// Check response status and handle 401 redirects.
async fn check_response(
    resp: gloo_net::http::Response,
) -> Result<gloo_net::http::Response, String> {
    let status = resp.status();
    if status == 401 {
        redirect_to_signin();
        return Err("인증이 만료되었습니다. 다시 로그인해 주세요.".to_string());
    }
    if !(200..300).contains(&status) {
        return Err(format!("서버 오류 ({})", status));
    }
    Ok(resp)
}

/// GET request without 401 redirect (for auth checks).
pub async fn get_silent<T: DeserializeOwned>(path: &str) -> Result<ApiResponse<T>, String> {
    let url = format!("{}{}", base_url(), path);
    let resp = gloo_net::http::Request::get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let text = resp.text().await.map_err(|e| e.to_string())?;
    serde_json::from_str(&text).map_err(|e| e.to_string())
}

/// GET request to API endpoint.
pub async fn get<T: DeserializeOwned>(path: &str) -> Result<ApiResponse<T>, String> {
    let cached = CACHE.with(|cache| cache.borrow_mut().get(path));
    if let Some(json_str) = cached {
        return serde_json::from_str(&json_str).map_err(|e| e.to_string());
    }

    let url = format!("{}{}", base_url(), path);
    let resp = gloo_net::http::Request::get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let resp = check_response(resp).await?;
    let text = resp.text().await.map_err(|e| e.to_string())?;
    CACHE.with(|cache| {
        cache.borrow_mut().insert(path.to_string(), text.clone());
    });
    serde_json::from_str(&text).map_err(|e| e.to_string())
}

/// POST request with JSON body.
pub async fn post<T: DeserializeOwned, B: serde::Serialize>(
    path: &str,
    body: &B,
) -> Result<ApiResponse<T>, String> {
    let url = format!("{}{}", base_url(), path);
    let mut builder = gloo_net::http::Request::post(&url);
    if let Some(token) = read_csrf_cookie() {
        builder = builder.header("x-csrf-token", &token);
    }
    let resp = builder
        .json(body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    invalidate_cache(path);
    let text = resp.text().await.map_err(|e| e.to_string())?;
    serde_json::from_str(&text).map_err(|e| e.to_string())
}

/// POST request without caring about the response body (e.g. logout).
pub async fn post_no_body(path: &str) -> Result<(), String> {
    let url = format!("{}{}", base_url(), path);
    let mut req = gloo_net::http::Request::post(&url);
    if let Some(token) = read_csrf_cookie() {
        req = req.header("x-csrf-token", &token);
    }
    let resp = req.send().await.map_err(|e| e.to_string())?;
    let status = resp.status();
    if !(200..300).contains(&status) {
        return Err(format!("서버 오류 ({})", status));
    }
    clear_cache();
    Ok(())
}
