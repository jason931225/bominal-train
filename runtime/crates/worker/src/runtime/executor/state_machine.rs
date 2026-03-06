use super::SrtOperationRequest;

pub(super) fn operation_requires_login_material(request: &SrtOperationRequest) -> bool {
    super::operation_requires_login_material(request)
}
