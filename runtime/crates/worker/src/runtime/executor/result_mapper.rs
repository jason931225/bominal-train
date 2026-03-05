use serde_json::Value;

use super::{
    ClaimedRuntimeJob, ExecutionError, SrtOperationResponse, SrtProviderError,
    build_redacted_result as build_redacted_result_impl,
    map_provider_error as map_provider_error_impl, map_srt_error as map_srt_error_impl,
    should_fallback_to_deterministic as should_fallback_to_deterministic_impl,
};

pub(super) fn build_redacted_result(
    job: &ClaimedRuntimeJob,
    response: &SrtOperationResponse,
) -> Value {
    build_redacted_result_impl(job, response)
}

pub(super) fn map_provider_error(
    error: SrtProviderError,
    provider: &str,
    operation_name: &str,
) -> ExecutionError {
    map_provider_error_impl(error, provider, operation_name)
}

pub(super) fn map_srt_error(error: SrtProviderError, operation_name: &str) -> ExecutionError {
    map_srt_error_impl(error, operation_name)
}

pub(super) fn should_fallback_to_deterministic(error: &SrtProviderError) -> bool {
    should_fallback_to_deterministic_impl(error)
}
