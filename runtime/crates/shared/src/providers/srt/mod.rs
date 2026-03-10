pub mod client;
pub mod errors;
pub mod netfunnel;
pub mod payment;
pub mod reqwest_client;
pub mod reservation;
pub mod search;
pub mod session;
pub mod ticket;
pub mod types;

use chrono::Utc;
use serde::Serialize;

use super::{ProviderAdapter, ProviderKind, ProviderOperation};

pub use client::ClientCallOutput;
pub use client::SrtClient;
pub use errors::{SrtProviderError, SrtResult};
pub use netfunnel::{NetfunnelSnapshot, NetfunnelStatus};
pub use payment::{CardIdentityType, PayWithCardRequest, PayWithCardResponse};
pub use reqwest_client::{ReqwestSrtClient, SrtClientFailureKind};
pub use reservation::{
    CancelRequest, CancelResponse, GetReservationsRequest, GetReservationsResponse, RefundRequest,
    RefundResponse, ReserveInfoRequest, ReserveInfoResponse, ReserveRequest, ReserveResponse,
    ReserveStandbyOptionSettingsRequest, ReserveStandbyOptionSettingsResponse,
    ReserveStandbyRequest, ReserveStandbyResponse, SrtReservation,
};
pub use search::{SearchTrainRequest, SearchTrainResponse, SrtTrain};
pub use secrecy::SecretString;
pub use session::{SessionCookie, SessionMaterial, SessionSnapshot, SrtRuntimeSession};
pub use ticket::{SrtTicket, TicketInfoRequest, TicketInfoResponse};
pub use types::{
    ClearRequest, ClearResponse, LoginAccountType, LoginRequest, LoginResponse, LogoutRequest,
    LogoutResponse, Passenger, PassengerKind, SeatClassPreference,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReloginPolicy {
    pub retry_once_on_auth_failure: bool,
}

impl Default for ReloginPolicy {
    fn default() -> Self {
        Self {
            retry_once_on_auth_failure: true,
        }
    }
}

#[derive(Debug, Clone)]
pub enum SrtOperationRequest {
    Login(LoginRequest),
    Logout(LogoutRequest),
    SearchTrain(SearchTrainRequest),
    Reserve(ReserveRequest),
    ReserveStandby(ReserveStandbyRequest),
    ReserveStandbyOptionSettings(ReserveStandbyOptionSettingsRequest),
    GetReservations(GetReservationsRequest),
    TicketInfo(TicketInfoRequest),
    Cancel(CancelRequest),
    PayWithCard(PayWithCardRequest),
    ReserveInfo(ReserveInfoRequest),
    Refund(RefundRequest),
    Clear(ClearRequest),
}

impl SrtOperationRequest {
    pub fn operation(&self) -> ProviderOperation {
        match self {
            Self::Login(_) => ProviderOperation::Login,
            Self::Logout(_) => ProviderOperation::Logout,
            Self::SearchTrain(_) => ProviderOperation::SearchTrain,
            Self::Reserve(_) => ProviderOperation::Reserve,
            Self::ReserveStandby(_) => ProviderOperation::ReserveStandby,
            Self::ReserveStandbyOptionSettings(_) => {
                ProviderOperation::ReserveStandbyOptionSettings
            }
            Self::GetReservations(_) => ProviderOperation::GetReservations,
            Self::TicketInfo(_) => ProviderOperation::TicketInfo,
            Self::Cancel(_) => ProviderOperation::Cancel,
            Self::PayWithCard(_) => ProviderOperation::PayWithCard,
            Self::ReserveInfo(_) => ProviderOperation::ReserveInfo,
            Self::Refund(_) => ProviderOperation::Refund,
            Self::Clear(_) => ProviderOperation::Clear,
        }
    }

    pub fn operation_name(&self) -> &'static str {
        operation_name(self.operation())
    }

    pub fn requires_authentication(&self) -> bool {
        !matches!(self, Self::Login(_) | Self::Clear(_))
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "operation", content = "payload", rename_all = "snake_case")]
pub enum SrtOperationResponse {
    Login(LoginResponse),
    Logout(LogoutResponse),
    SearchTrain(SearchTrainResponse),
    Reserve(ReserveResponse),
    ReserveStandby(ReserveStandbyResponse),
    ReserveStandbyOptionSettings(ReserveStandbyOptionSettingsResponse),
    GetReservations(GetReservationsResponse),
    TicketInfo(TicketInfoResponse),
    Cancel(CancelResponse),
    PayWithCard(PayWithCardResponse),
    ReserveInfo(ReserveInfoResponse),
    Refund(RefundResponse),
    Clear(ClearResponse),
}

impl SrtOperationResponse {
    pub fn operation_name(&self) -> &'static str {
        match self {
            Self::Login(_) => "login",
            Self::Logout(_) => "logout",
            Self::SearchTrain(_) => "search_train",
            Self::Reserve(_) => "reserve",
            Self::ReserveStandby(_) => "reserve_standby",
            Self::ReserveStandbyOptionSettings(_) => "reserve_standby_option_settings",
            Self::GetReservations(_) => "get_reservations",
            Self::TicketInfo(_) => "ticket_info",
            Self::Cancel(_) => "cancel",
            Self::PayWithCard(_) => "pay_with_card",
            Self::ReserveInfo(_) => "reserve_info",
            Self::Refund(_) => "refund",
            Self::Clear(_) => "clear",
        }
    }
}

#[derive(Debug)]
pub struct SrtProviderAdapter<C: SrtClient> {
    client: C,
    session: SrtRuntimeSession,
    relogin_policy: ReloginPolicy,
    cached_login_request: Option<LoginRequest>,
}

macro_rules! authenticated_call {
    ($adapter:expr, |$client:ident, $session:ident| $call:expr) => {{
        let adapter = &mut *$adapter;
        let mut relogin_attempted = false;

        loop {
            let session_snapshot = adapter.current_session().await?;
            match {
                let $client = &mut adapter.client;
                let $session = &session_snapshot;
                $call
            }
            .await
            {
                Ok(output) => {
                    adapter.apply_output_session(&output);
                    break Ok(output.payload);
                }
                Err(error)
                    if adapter.relogin_policy.retry_once_on_auth_failure
                        && !relogin_attempted
                        && error.is_auth_failure() =>
                {
                    adapter.try_relogin().await?;
                    relogin_attempted = true;
                }
                Err(error) => break Err(error),
            }
        }
    }};
}

impl<C: SrtClient> SrtProviderAdapter<C> {
    pub fn new(client: C) -> Self {
        Self {
            client,
            session: SrtRuntimeSession::default(),
            relogin_policy: ReloginPolicy::default(),
            cached_login_request: None,
        }
    }

    pub fn with_relogin_policy(mut self, relogin_policy: ReloginPolicy) -> Self {
        self.relogin_policy = relogin_policy;
        self
    }

    pub fn session_snapshot(&self) -> Option<SessionSnapshot> {
        self.session.snapshot()
    }

    pub fn into_client(self) -> C {
        self.client
    }

    fn apply_output_session<T>(&mut self, output: &ClientCallOutput<T>) {
        if let Some(update) = &output.session_update {
            self.session.apply_update(update.clone(), Utc::now());
        }
    }

    async fn perform_login(
        &mut self,
        request: &LoginRequest,
        cache_request: bool,
    ) -> SrtResult<LoginResponse> {
        let output = self.client.login(request).await?;
        self.apply_output_session(&output);

        let response = output.payload;
        self.session.activate(response.session.clone(), Utc::now());

        if cache_request {
            self.cached_login_request = Some(request.clone());
        }

        Ok(response)
    }

    async fn try_relogin(&mut self) -> SrtResult<()> {
        let cached_request = self
            .cached_login_request
            .clone()
            .ok_or(SrtProviderError::ReloginUnavailable)?;

        self.perform_login(&cached_request, false).await.map(|_| ())
    }

    async fn current_session(&mut self) -> SrtResult<SessionSnapshot> {
        let now = Utc::now();
        if let Some(snapshot) = self.session.snapshot()
            && !snapshot.is_expired_at(now)
        {
            return Ok(snapshot);
        }

        self.try_relogin().await?;
        self.session.snapshot().ok_or(SrtProviderError::NotLoggedIn)
    }

    pub async fn dispatch(
        &mut self,
        request: SrtOperationRequest,
    ) -> SrtResult<SrtOperationResponse> {
        match request {
            SrtOperationRequest::Login(request) => {
                self.login(request).await.map(SrtOperationResponse::Login)
            }
            SrtOperationRequest::Logout(request) => {
                self.logout(request).await.map(SrtOperationResponse::Logout)
            }
            SrtOperationRequest::SearchTrain(request) => self
                .search_train(request)
                .await
                .map(SrtOperationResponse::SearchTrain),
            SrtOperationRequest::Reserve(request) => self
                .reserve(request)
                .await
                .map(SrtOperationResponse::Reserve),
            SrtOperationRequest::ReserveStandby(request) => self
                .reserve_standby(request)
                .await
                .map(SrtOperationResponse::ReserveStandby),
            SrtOperationRequest::ReserveStandbyOptionSettings(request) => self
                .reserve_standby_option_settings(request)
                .await
                .map(SrtOperationResponse::ReserveStandbyOptionSettings),
            SrtOperationRequest::GetReservations(request) => self
                .get_reservations(request)
                .await
                .map(SrtOperationResponse::GetReservations),
            SrtOperationRequest::TicketInfo(request) => self
                .ticket_info(request)
                .await
                .map(SrtOperationResponse::TicketInfo),
            SrtOperationRequest::Cancel(request) => {
                self.cancel(request).await.map(SrtOperationResponse::Cancel)
            }
            SrtOperationRequest::PayWithCard(request) => self
                .pay_with_card(request)
                .await
                .map(SrtOperationResponse::PayWithCard),
            SrtOperationRequest::ReserveInfo(request) => self
                .reserve_info(request)
                .await
                .map(SrtOperationResponse::ReserveInfo),
            SrtOperationRequest::Refund(request) => {
                self.refund(request).await.map(SrtOperationResponse::Refund)
            }
            SrtOperationRequest::Clear(request) => {
                self.clear(request).await.map(SrtOperationResponse::Clear)
            }
        }
    }
}

impl<C: SrtClient> ProviderAdapter for SrtProviderAdapter<C> {
    fn provider_kind(&self) -> ProviderKind {
        ProviderKind::Srt
    }

    async fn login(&mut self, request: LoginRequest) -> SrtResult<LoginResponse> {
        self.perform_login(&request, true).await
    }

    async fn logout(&mut self, request: LogoutRequest) -> SrtResult<LogoutResponse> {
        let response = if self.session.is_active() {
            authenticated_call!(self, |client, session| client.logout(session, &request))?
        } else {
            LogoutResponse { logged_out: true }
        };

        self.session.clear();
        self.cached_login_request = None;
        Ok(response)
    }

    async fn search_train(
        &mut self,
        request: SearchTrainRequest,
    ) -> SrtResult<SearchTrainResponse> {
        authenticated_call!(self, |client, session| client
            .search_train(session, &request))
    }

    async fn reserve(&mut self, request: ReserveRequest) -> SrtResult<ReserveResponse> {
        authenticated_call!(self, |client, session| client.reserve(session, &request))
    }

    async fn reserve_standby(
        &mut self,
        request: ReserveStandbyRequest,
    ) -> SrtResult<ReserveStandbyResponse> {
        authenticated_call!(self, |client, session| client
            .reserve_standby(session, &request))
    }

    async fn reserve_standby_option_settings(
        &mut self,
        request: ReserveStandbyOptionSettingsRequest,
    ) -> SrtResult<ReserveStandbyOptionSettingsResponse> {
        authenticated_call!(self, |client, session| {
            client.reserve_standby_option_settings(session, &request)
        })
    }

    async fn get_reservations(
        &mut self,
        request: GetReservationsRequest,
    ) -> SrtResult<GetReservationsResponse> {
        authenticated_call!(self, |client, session| client
            .get_reservations(session, &request))
    }

    async fn ticket_info(&mut self, request: TicketInfoRequest) -> SrtResult<TicketInfoResponse> {
        authenticated_call!(self, |client, session| client
            .ticket_info(session, &request))
    }

    async fn cancel(&mut self, request: CancelRequest) -> SrtResult<CancelResponse> {
        authenticated_call!(self, |client, session| client.cancel(session, &request))
    }

    async fn pay_with_card(
        &mut self,
        request: PayWithCardRequest,
    ) -> SrtResult<PayWithCardResponse> {
        authenticated_call!(self, |client, session| client
            .pay_with_card(session, &request))
    }

    async fn reserve_info(
        &mut self,
        request: ReserveInfoRequest,
    ) -> SrtResult<ReserveInfoResponse> {
        authenticated_call!(self, |client, session| client
            .reserve_info(session, &request))
    }

    async fn refund(&mut self, request: RefundRequest) -> SrtResult<RefundResponse> {
        authenticated_call!(self, |client, session| client.refund(session, &request))
    }

    async fn clear(&mut self, request: ClearRequest) -> SrtResult<ClearResponse> {
        let output = self.client.clear(&request).await?;
        self.apply_output_session(&output);
        Ok(output.payload)
    }
}

fn operation_name(operation: ProviderOperation) -> &'static str {
    match operation {
        ProviderOperation::Login => "login",
        ProviderOperation::Logout => "logout",
        ProviderOperation::SearchTrain => "search_train",
        ProviderOperation::Reserve => "reserve",
        ProviderOperation::ReserveStandby => "reserve_standby",
        ProviderOperation::ReserveStandbyOptionSettings => "reserve_standby_option_settings",
        ProviderOperation::GetReservations => "get_reservations",
        ProviderOperation::TicketInfo => "ticket_info",
        ProviderOperation::Cancel => "cancel",
        ProviderOperation::PayWithCard => "pay_with_card",
        ProviderOperation::ReserveInfo => "reserve_info",
        ProviderOperation::Refund => "refund",
        ProviderOperation::Clear => "clear",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_operation_names_match_srtgo_contract() {
        let observed = [
            ProviderOperation::Login,
            ProviderOperation::Logout,
            ProviderOperation::SearchTrain,
            ProviderOperation::Reserve,
            ProviderOperation::ReserveStandby,
            ProviderOperation::ReserveStandbyOptionSettings,
            ProviderOperation::GetReservations,
            ProviderOperation::TicketInfo,
            ProviderOperation::Cancel,
            ProviderOperation::PayWithCard,
            ProviderOperation::ReserveInfo,
            ProviderOperation::Refund,
            ProviderOperation::Clear,
        ]
        .into_iter()
        .map(operation_name)
        .collect::<Vec<_>>();

        assert_eq!(
            observed,
            vec![
                "login",
                "logout",
                "search_train",
                "reserve",
                "reserve_standby",
                "reserve_standby_option_settings",
                "get_reservations",
                "ticket_info",
                "cancel",
                "pay_with_card",
                "reserve_info",
                "refund",
                "clear",
            ]
        );
    }

    #[test]
    fn requires_authentication_is_fail_closed() {
        let login = SrtOperationRequest::Login(LoginRequest {
            account_type: LoginAccountType::MembershipNumber,
            account_identifier: "member-1".to_string(),
            password: SecretString::new("password".to_string().into_boxed_str()),
        });
        let clear = SrtOperationRequest::Clear(ClearRequest);
        let logout = SrtOperationRequest::Logout(LogoutRequest);

        assert!(!login.requires_authentication());
        assert!(!clear.requires_authentication());
        assert!(logout.requires_authentication());
    }

    #[tokio::test]
    async fn async_provider_adapter_dispatch_keeps_existing_result_shapes() {
        let mut adapter = SrtProviderAdapter::new(ReqwestSrtClient::deterministic());
        let login = adapter
            .login(LoginRequest {
                account_type: LoginAccountType::MembershipNumber,
                account_identifier: "member-1".to_string(),
                password: SecretString::new("password".to_string().into_boxed_str()),
            })
            .await
            .expect("deterministic login should succeed");

        assert_eq!(login.membership_number, "MEM-DET-1");

        let response = adapter
            .dispatch(SrtOperationRequest::SearchTrain(SearchTrainRequest {
                dep_station_code: "0551".to_string(),
                arr_station_code: "0020".to_string(),
                dep_date: "20260305".to_string(),
                dep_time: "080000".to_string(),
                time_limit: None,
                passengers: vec![Passenger::adult(1)],
                available_only: true,
            }))
            .await
            .expect("deterministic dispatch should succeed");

        match response {
            SrtOperationResponse::SearchTrain(payload) => {
                assert_eq!(payload.trains.len(), 1);
                assert_eq!(payload.netfunnel_status, NetfunnelStatus::Pass);
            }
            other => panic!(
                "expected search_train response shape, got {}",
                other.operation_name()
            ),
        }
    }
}
