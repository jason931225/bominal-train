use super::{
    ParsedProviderExecution, PaymentExecutionPolicy, ReqwestKtxClient, ReqwestSrtClient,
    SrtOperationResponse, SrtProviderError, dispatch_ktx_with_client as dispatch_ktx_with_client_impl,
    dispatch_srt_with_client as dispatch_srt_with_client_impl,
    resolve_ktx_base_url as resolve_ktx_base_url_impl,
    resolve_srt_base_url as resolve_srt_base_url_impl,
    should_attempt_live_provider as should_attempt_live_provider_impl,
};

pub(super) fn dispatch_srt_with_client(
    parsed: &ParsedProviderExecution,
    client: ReqwestSrtClient,
) -> Result<SrtOperationResponse, SrtProviderError> {
    dispatch_srt_with_client_impl(parsed, client)
}

pub(super) fn dispatch_ktx_with_client(
    parsed: &ParsedProviderExecution,
    client: ReqwestKtxClient,
) -> Result<SrtOperationResponse, SrtProviderError> {
    dispatch_ktx_with_client_impl(parsed, client)
}

pub(super) fn should_attempt_live_provider(policy: &PaymentExecutionPolicy) -> bool {
    should_attempt_live_provider_impl(policy)
}

pub(super) fn resolve_srt_base_url() -> String {
    resolve_srt_base_url_impl()
}

pub(super) fn resolve_ktx_base_url() -> String {
    resolve_ktx_base_url_impl()
}
