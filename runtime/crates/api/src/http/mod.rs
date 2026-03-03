use std::sync::Arc;

use axum::{Router, middleware};
use tower_http::{
    compression::CompressionLayer, cors::CorsLayer, services::ServeDir, trace::TraceLayer,
};

use super::AppState;

mod auth;
mod internal;
mod internal_auth;
mod internal_provider_jobs;
mod internal_providers_srt;
mod modules;
#[path = "../services/payment_method_service.rs"]
mod payment_method_service;
#[path = "../services/provider_credentials_service.rs"]
mod provider_credentials_service;
#[path = "../services/provider_jobs_service.rs"]
mod provider_jobs_service;
mod runtime_queue;

pub(crate) fn build_router(state: Arc<AppState>) -> Router {
    let assets_dir =
        std::env::var("FRONTEND_ASSETS_DIR").unwrap_or_else(|_| "runtime/frontend/dist".to_string());

    let router = Router::<Arc<AppState>>::new();
    let router = internal::register(router);
    let router = register_internal_api(router, state.clone());
    let router = modules::register(router);
    let router = auth::register(router);
    let router = runtime_queue::register(router);

    router
        .nest_service("/assets", ServeDir::new(assets_dir))
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive())
        .with_state(state)
}

fn register_internal_api(
    router: Router<Arc<AppState>>,
    state: Arc<AppState>,
) -> Router<Arc<AppState>> {
    let mount_aliases = internal_auth::compatibility_aliases_enabled(state.as_ref());

    let internal_router = Router::<Arc<AppState>>::new();
    let internal_router = internal_providers_srt::register(internal_router, mount_aliases);
    let internal_router = internal_provider_jobs::register(internal_router, mount_aliases);
    let internal_router = internal_router.layer(middleware::from_fn_with_state(
        state,
        internal_auth::require_service_jwt,
    ));

    router.merge(internal_router)
}
