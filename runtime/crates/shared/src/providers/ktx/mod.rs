pub mod reqwest_client;

use crate::providers::ProviderAdapter;
use crate::providers::srt::{
    LoginRequest, LoginResponse, SessionSnapshot, SrtOperationRequest, SrtOperationResponse,
    SrtProviderAdapter, SrtResult,
};

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

    pub fn login(&mut self, request: LoginRequest) -> SrtResult<LoginResponse> {
        self.inner.login(request)
    }

    pub fn dispatch(&mut self, request: SrtOperationRequest) -> SrtResult<SrtOperationResponse> {
        self.inner.dispatch(request)
    }

    pub fn session_snapshot(&self) -> Option<SessionSnapshot> {
        self.inner.session_snapshot()
    }
}
