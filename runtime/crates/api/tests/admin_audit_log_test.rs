use bominal_shared::config::AdminRole;

#[allow(dead_code)]
#[path = "../src/main.rs"]
mod api_main;

#[test]
fn admin_audit_migration_enforces_append_only_log_contract() {
    let migration = include_str!("../../../migrations/202603030004_admin_ops.sql");

    assert!(migration.contains("create table if not exists admin_audit_log"));
    assert!(migration.contains("reason text not null"));
    assert!(migration.contains("request_id text not null"));
    assert!(migration.contains("action text not null"));
    assert!(migration.contains("create or replace function admin_audit_log_prevent_mutation"));
    assert!(migration.contains("raise exception 'admin_audit_log is append-only'"));
    assert!(migration.contains("create trigger tr_admin_audit_log_immutable"));
}

#[test]
fn admin_sensitive_confirmation_is_strictly_enforced() {
    let result = api_main::services::admin_service::validate_sensitive_confirmation(
        "", "job-123", "job-123",
    );
    assert!(result.is_err());

    let result = api_main::services::admin_service::validate_sensitive_confirmation(
        "manual intervention",
        "",
        "job-123",
    );
    assert!(result.is_err());

    let result = api_main::services::admin_service::validate_sensitive_confirmation(
        "manual intervention",
        "different-target",
        "job-123",
    );
    assert!(result.is_err());

    let ok = api_main::services::admin_service::validate_sensitive_confirmation(
        "manual intervention",
        "job-123",
        "job-123",
    );
    assert!(ok.is_ok());
}

#[test]
fn admin_role_matrix_is_fail_closed_for_mutations() {
    assert!(api_main::services::admin_service::role_allows_admin_read(
        &AdminRole::Admin
    ));
    assert!(api_main::services::admin_service::role_allows_admin_read(
        &AdminRole::Operator
    ));
    assert!(api_main::services::admin_service::role_allows_admin_read(
        &AdminRole::Viewer
    ));
    assert!(!api_main::services::admin_service::role_allows_admin_read(
        &AdminRole::User
    ));

    assert!(api_main::services::admin_service::role_allows_admin_mutation(&AdminRole::Admin));
    assert!(api_main::services::admin_service::role_allows_admin_mutation(&AdminRole::Operator));
    assert!(!api_main::services::admin_service::role_allows_admin_mutation(&AdminRole::Viewer));
    assert!(!api_main::services::admin_service::role_allows_admin_mutation(&AdminRole::User));
}
