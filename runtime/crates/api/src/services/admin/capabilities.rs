use bominal_shared::config::AdminRole;

use super::super::auth_service;

pub(crate) fn role_allows_admin_read(role: &AdminRole) -> bool {
    matches!(
        role,
        AdminRole::Admin | AdminRole::Operator | AdminRole::Viewer
    )
}

pub(crate) fn role_allows_admin_mutation(role: &AdminRole) -> bool {
    auth_service::admin_role_can_mutate(role)
}
