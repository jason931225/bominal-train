use std::sync::Arc;

use axum::{Router, routing::post};

use super::super::super::AppState;
use super::handlers::create_invite;

pub(super) fn register_invites(
    router: Router<Arc<AppState>>,
    mount_aliases: bool,
) -> Router<Arc<AppState>> {
    let router = router.route("/internal/v1/auth/invites", post(create_invite));

    if mount_aliases {
        return router.route("/api/internal/auth/invites", post(create_invite));
    }

    router
}
