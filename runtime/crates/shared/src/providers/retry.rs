use super::error::ProviderError;

pub fn is_retryable_provider_error(error: &ProviderError) -> bool {
    match error {
        ProviderError::Transport { .. } => true,
        ProviderError::OperationFailed { message } => {
            let normalized = message.to_ascii_lowercase();
            normalized.contains("timeout")
                || normalized.contains("temporar")
                || normalized.contains("connection reset")
                || normalized.contains("network")
        }
        ProviderError::SessionExpired
        | ProviderError::Unauthorized
        | ProviderError::NotLoggedIn
        | ProviderError::ReloginUnavailable => true,
        ProviderError::UnsupportedOperation { .. } => false,
    }
}
