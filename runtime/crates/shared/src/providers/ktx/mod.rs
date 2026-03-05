pub mod reqwest_client;

use crate::providers::{ProviderAdapter, ProviderResult};
use crate::providers::model::{
    LoginRequest, LoginResponse, ProviderOperationRequest, ProviderOperationResponse,
    SessionSnapshot,
};
use crate::providers::srt::SrtProviderAdapter;

pub use reqwest_client::{KtxClientFailureKind, ReqwestKtxClient};

#[derive(Debug)]
pub struct KtxProviderAdapter {
    inner: SrtProviderAdapter<ReqwestKtxClient>,
}

impl KtxProviderAdapter {
    pub fn new(client: ReqwestKtxClient) -> Self {
        Self {
            inner: SrtProviderAdapter::new(client),
        }
    }

    pub fn login(&mut self, request: LoginRequest) -> ProviderResult<LoginResponse> {
        self.inner.login(request)
    }

    pub fn dispatch(
        &mut self,
        request: ProviderOperationRequest,
    ) -> ProviderResult<ProviderOperationResponse> {
        self.inner.dispatch(request)
    }

    pub fn session_snapshot(&self) -> Option<SessionSnapshot> {
        self.inner.session_snapshot()
    }
}
