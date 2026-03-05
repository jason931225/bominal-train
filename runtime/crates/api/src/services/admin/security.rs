use super::AdminServiceError;

pub(crate) fn validate_sensitive_confirmation(
    reason: &str,
    confirm_target: &str,
    expected_target: &str,
) -> Result<(), AdminServiceError> {
    let reason = reason.trim();
    let confirm_target = confirm_target.trim();
    if reason.len() < 8 {
        return Err(AdminServiceError::InvalidRequest(
            "reason must be at least 8 characters",
        ));
    }
    if reason.len() > 500 {
        return Err(AdminServiceError::InvalidRequest("reason too long"));
    }
    if confirm_target != expected_target {
        return Err(AdminServiceError::InvalidRequest(
            "typed confirmation target mismatch",
        ));
    }
    Ok(())
}
