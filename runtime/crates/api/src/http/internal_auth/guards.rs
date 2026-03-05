use super::super::super::AppState;

pub(super) fn compatibility_aliases_enabled(state: &AppState) -> bool {
    super::service_identity::compatibility_aliases_enabled_impl(state)
}
