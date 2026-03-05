use super::{ClaimedRuntimeJob, ExecutionError, ParsedProviderExecution};

pub(super) fn parse_provider_execution(
    job: &ClaimedRuntimeJob,
    provider: &str,
) -> Result<ParsedProviderExecution, ExecutionError> {
    super::parse_provider_execution(job, provider)
}
