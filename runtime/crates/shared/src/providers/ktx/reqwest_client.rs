use std::collections::HashMap;
use std::time::Duration;

use chrono::{Duration as ChronoDuration, Utc};
use reqwest::Client;
use reqwest::header::CONTENT_TYPE;
use secrecy::{ExposeSecret, SecretString};
use url::form_urlencoded;

use crate::providers::ProviderOperation;
use crate::providers::srt::{
    CancelRequest, CancelResponse, ClearRequest, ClearResponse, ClientCallOutput,
    GetReservationsRequest, GetReservationsResponse, LoginRequest, LoginResponse, LogoutRequest,
    LogoutResponse, NetfunnelStatus, PayWithCardRequest, PayWithCardResponse, RefundRequest,
    RefundResponse, ReserveInfoRequest, ReserveInfoResponse, ReserveRequest, ReserveResponse,
    ReserveStandbyOptionSettingsRequest, ReserveStandbyOptionSettingsResponse,
    ReserveStandbyRequest, ReserveStandbyResponse, SearchTrainRequest, SearchTrainResponse,
    SessionCookie, SessionMaterial, SessionSnapshot, SrtClient, SrtProviderError, SrtReservation,
    SrtResult, SrtTicket, SrtTrain, TicketInfoRequest, TicketInfoResponse,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KtxClientFailureKind {
    Transient,
    RateLimited,
    Fatal,
    SessionExpired,
    Unauthorized,
}

impl KtxClientFailureKind {
    fn into_provider_error(self, operation: ProviderOperation) -> SrtProviderError {
        match self {
            Self::Transient => SrtProviderError::Transport {
                message: format!(
                    "deterministic transport failure for {}",
                    operation_name(operation)
                ),
            },
            Self::RateLimited => SrtProviderError::OperationFailed {
                message: format!(
                    "rate_limited deterministic failure for {}",
                    operation_name(operation)
                ),
            },
            Self::Fatal => SrtProviderError::OperationFailed {
                message: format!(
                    "deterministic provider failure for {}",
                    operation_name(operation)
                ),
            },
            Self::SessionExpired => SrtProviderError::SessionExpired,
            Self::Unauthorized => SrtProviderError::Unauthorized,
        }
    }
}

#[derive(Debug, Clone)]
struct PlannedFailure {
    kind: KtxClientFailureKind,
    remaining: usize,
}

#[derive(Debug, Clone)]
enum KtxTransportMode {
    Deterministic,
    Live { base_url: String, client: Client },
}

#[derive(Debug, Clone)]
pub struct ReqwestKtxClient {
    mode: KtxTransportMode,
    failures: HashMap<ProviderOperation, PlannedFailure>,
}

impl Default for ReqwestKtxClient {
    fn default() -> Self {
        Self::deterministic()
    }
}

impl ReqwestKtxClient {
    pub fn deterministic() -> Self {
        Self {
            mode: KtxTransportMode::Deterministic,
            failures: HashMap::new(),
        }
    }

    pub fn live(base_url: impl Into<String>) -> Self {
        Self::live_with_timeout(base_url, Duration::from_secs(15))
    }

    pub fn live_with_timeout(base_url: impl Into<String>, timeout: Duration) -> Self {
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .unwrap_or_else(|_| Client::new());
        Self {
            mode: KtxTransportMode::Live {
                base_url: trim_trailing_slash(base_url.into()),
                client,
            },
            failures: HashMap::new(),
        }
    }

    pub fn live_default() -> Self {
        Self::live("https://smart.letskorail.com")
    }

    pub fn with_failure(
        mut self,
        operation: ProviderOperation,
        kind: KtxClientFailureKind,
        times: usize,
    ) -> Self {
        if times > 0 {
            self.failures.insert(
                operation,
                PlannedFailure {
                    kind,
                    remaining: times,
                },
            );
        }
        self
    }

    pub fn with_srt_failure(
        self,
        operation: ProviderOperation,
        kind: crate::providers::srt::SrtClientFailureKind,
        times: usize,
    ) -> Self {
        let mapped = match kind {
            crate::providers::srt::SrtClientFailureKind::Transient => {
                KtxClientFailureKind::Transient
            }
            crate::providers::srt::SrtClientFailureKind::RateLimited => {
                KtxClientFailureKind::RateLimited
            }
            crate::providers::srt::SrtClientFailureKind::Fatal => KtxClientFailureKind::Fatal,
            crate::providers::srt::SrtClientFailureKind::SessionExpired => {
                KtxClientFailureKind::SessionExpired
            }
            crate::providers::srt::SrtClientFailureKind::Unauthorized => {
                KtxClientFailureKind::Unauthorized
            }
        };
        self.with_failure(operation, mapped, times)
    }

    fn maybe_fail(&mut self, operation: ProviderOperation) -> SrtResult<()> {
        let mut clear = false;
        let failure = self.failures.get_mut(&operation).and_then(|planned| {
            if planned.remaining == 0 {
                return None;
            }

            planned.remaining -= 1;
            if planned.remaining == 0 {
                clear = true;
            }
            Some(planned.kind)
        });

        if clear {
            self.failures.remove(&operation);
        }

        if let Some(kind) = failure {
            return Err(kind.into_provider_error(operation));
        }

        Ok(())
    }

    async fn maybe_live_form(
        &self,
        operation: ProviderOperation,
        endpoint: &str,
        form: Vec<(String, String)>,
    ) -> SrtResult<()> {
        let KtxTransportMode::Live { base_url, client } = &self.mode else {
            return Ok(());
        };

        let url = format!("{base_url}{endpoint}");
        let encoded_form = encode_form_pairs(&form);
        let response = client
            .post(url)
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(encoded_form)
            .send()
            .await
            .map_err(|err| SrtProviderError::Transport {
                message: format!(
                    "ktx transport failed for {}: {err}",
                    operation_name(operation)
                ),
            })?;
        let status = response.status();

        if status.is_success() {
            return Ok(());
        }
        if status.as_u16() == 429 {
            return Err(SrtProviderError::OperationFailed {
                message: format!("rate_limited for {}", operation_name(operation)),
            });
        }
        if status.as_u16() == 401 || status.as_u16() == 403 {
            return Err(SrtProviderError::Unauthorized);
        }
        if status.is_server_error() {
            return Err(SrtProviderError::Transport {
                message: format!(
                    "ktx upstream status {} for {}",
                    status.as_u16(),
                    operation_name(operation)
                ),
            });
        }

        Err(SrtProviderError::OperationFailed {
            message: format!(
                "ktx upstream rejected {} with status {}",
                operation_name(operation),
                status.as_u16()
            ),
        })
    }

    fn canned_session() -> SessionMaterial {
        SessionMaterial {
            cookies: vec![SessionCookie::new(
                "JSESSIONID",
                SecretString::new("ktx-deterministic-cookie".into()),
            )],
            expires_at: Some(Utc::now() + ChronoDuration::minutes(20)),
        }
    }

    fn canned_login_response() -> LoginResponse {
        LoginResponse {
            membership_number: "KTX-DET-1".to_string(),
            membership_name: "KTX Deterministic User".to_string(),
            phone_number: Some("01000000000".to_string()),
            session: Self::canned_session(),
        }
    }

    fn canned_train() -> SrtTrain {
        SrtTrain {
            train_code: "100".to_string(),
            train_number: "123".to_string(),
            dep_station_code: "0001".to_string(),
            arr_station_code: "0020".to_string(),
            dep_date: "20260305".to_string(),
            dep_time: "081500".to_string(),
            arr_date: "20260305".to_string(),
            arr_time: "102500".to_string(),
            general_seat_available: true,
            special_seat_available: true,
            standby_available: false,
        }
    }

    fn canned_reservation() -> SrtReservation {
        SrtReservation {
            reservation_id: "KTX-PNR-1".to_string(),
            train_number: "123".to_string(),
            dep_station_code: "0001".to_string(),
            arr_station_code: "0020".to_string(),
            dep_date: "20260305".to_string(),
            dep_time: "081500".to_string(),
            arr_time: "102500".to_string(),
            seat_count: 1,
            total_cost: 62_000,
            paid: false,
            waiting: false,
        }
    }

    fn canned_ticket() -> SrtTicket {
        SrtTicket {
            reservation_id: "KTX-PNR-1".to_string(),
            car: Some("8".to_string()),
            seat: Some("12C".to_string()),
            seat_class: "general".to_string(),
            passenger_type: "adult".to_string(),
            price: 62_000,
            discount: 0,
            waiting: false,
        }
    }
}

impl SrtClient for ReqwestKtxClient {
    async fn login(
        &mut self,
        request: &LoginRequest,
    ) -> SrtResult<ClientCallOutput<LoginResponse>> {
        self.maybe_fail(ProviderOperation::Login)?;
        self.maybe_live_form(
            ProviderOperation::Login,
            "/classes/com.korail.mobile.login.Login",
            vec![
                (
                    "txtMemberNo".to_string(),
                    request.account_identifier.clone(),
                ),
                (
                    "txtPwd".to_string(),
                    request.password.expose_secret().to_string(),
                ),
            ],
        )
        .await?;
        Ok(ClientCallOutput::success(Self::canned_login_response()))
    }

    async fn logout(
        &mut self,
        _session: &SessionSnapshot,
        _request: &LogoutRequest,
    ) -> SrtResult<ClientCallOutput<LogoutResponse>> {
        self.maybe_fail(ProviderOperation::Logout)?;
        self.maybe_live_form(
            ProviderOperation::Logout,
            "/classes/com.korail.mobile.common.logout",
            Vec::new(),
        )
        .await?;
        Ok(ClientCallOutput::success(LogoutResponse {
            logged_out: true,
        }))
    }

    async fn search_train(
        &mut self,
        _session: &SessionSnapshot,
        request: &SearchTrainRequest,
    ) -> SrtResult<ClientCallOutput<SearchTrainResponse>> {
        self.maybe_fail(ProviderOperation::SearchTrain)?;
        self.maybe_live_form(
            ProviderOperation::SearchTrain,
            "/classes/com.korail.mobile.seatMovie.ScheduleView",
            vec![
                ("txtGoStart".to_string(), request.dep_station_code.clone()),
                ("txtGoEnd".to_string(), request.arr_station_code.clone()),
                ("txtGoAbrdDt".to_string(), request.dep_date.clone()),
                ("txtGoHour".to_string(), request.dep_time.clone()),
            ],
        )
        .await?;
        Ok(ClientCallOutput::success(SearchTrainResponse {
            trains: vec![Self::canned_train()],
            netfunnel_status: NetfunnelStatus::Pass,
        }))
    }

    async fn reserve(
        &mut self,
        _session: &SessionSnapshot,
        request: &ReserveRequest,
    ) -> SrtResult<ClientCallOutput<ReserveResponse>> {
        self.maybe_fail(ProviderOperation::Reserve)?;
        self.maybe_live_form(
            ProviderOperation::Reserve,
            "/classes/com.korail.mobile.certification.TicketReservation",
            vec![
                ("txtTrnNo1".to_string(), request.train.train_number.clone()),
                ("txtRunDt1".to_string(), request.train.dep_date.clone()),
            ],
        )
        .await?;
        Ok(ClientCallOutput::success(ReserveResponse {
            reservation: Self::canned_reservation(),
        }))
    }

    async fn reserve_standby(
        &mut self,
        _session: &SessionSnapshot,
        _request: &ReserveStandbyRequest,
    ) -> SrtResult<ClientCallOutput<ReserveStandbyResponse>> {
        Err(SrtProviderError::UnsupportedOperation {
            operation: "reserve_standby",
        })
    }

    async fn reserve_standby_option_settings(
        &mut self,
        _session: &SessionSnapshot,
        _request: &ReserveStandbyOptionSettingsRequest,
    ) -> SrtResult<ClientCallOutput<ReserveStandbyOptionSettingsResponse>> {
        Err(SrtProviderError::UnsupportedOperation {
            operation: "reserve_standby_option_settings",
        })
    }

    async fn get_reservations(
        &mut self,
        _session: &SessionSnapshot,
        _request: &GetReservationsRequest,
    ) -> SrtResult<ClientCallOutput<GetReservationsResponse>> {
        self.maybe_fail(ProviderOperation::GetReservations)?;
        self.maybe_live_form(
            ProviderOperation::GetReservations,
            "/classes/com.korail.mobile.certification.ReservationList",
            Vec::new(),
        )
        .await?;
        Ok(ClientCallOutput::success(GetReservationsResponse {
            reservations: vec![Self::canned_reservation()],
        }))
    }

    async fn ticket_info(
        &mut self,
        _session: &SessionSnapshot,
        request: &TicketInfoRequest,
    ) -> SrtResult<ClientCallOutput<TicketInfoResponse>> {
        self.maybe_fail(ProviderOperation::TicketInfo)?;
        self.maybe_live_form(
            ProviderOperation::TicketInfo,
            "/classes/com.korail.mobile.refunds.SelTicketInfo",
            vec![("txtPnrNo".to_string(), request.reservation_id.clone())],
        )
        .await?;
        Ok(ClientCallOutput::success(TicketInfoResponse {
            tickets: vec![Self::canned_ticket()],
        }))
    }

    async fn cancel(
        &mut self,
        _session: &SessionSnapshot,
        request: &CancelRequest,
    ) -> SrtResult<ClientCallOutput<CancelResponse>> {
        self.maybe_fail(ProviderOperation::Cancel)?;
        self.maybe_live_form(
            ProviderOperation::Cancel,
            "/classes/com.korail.mobile.reservationCancel.ReservationCancelChk",
            vec![("txtPnrNo".to_string(), request.reservation_id.clone())],
        )
        .await?;
        Ok(ClientCallOutput::success(CancelResponse { canceled: true }))
    }

    async fn pay_with_card(
        &mut self,
        _session: &SessionSnapshot,
        request: &PayWithCardRequest,
    ) -> SrtResult<ClientCallOutput<PayWithCardResponse>> {
        self.maybe_fail(ProviderOperation::PayWithCard)?;
        self.maybe_live_form(
            ProviderOperation::PayWithCard,
            "/classes/com.korail.mobile.payment.ReservationPayment",
            vec![
                ("txtPnrNo".to_string(), request.reservation_id.clone()),
                (
                    "hidStlCrCrdNo1".to_string(),
                    request.card_number.expose_secret().to_string(),
                ),
                (
                    "hidVanPwd1".to_string(),
                    request.card_password_two_digits.expose_secret().to_string(),
                ),
                (
                    "hidAthnVal1".to_string(),
                    request.card_validation_number.expose_secret().to_string(),
                ),
                (
                    "hidCrdVlidTrm1".to_string(),
                    request.card_expiry_yymm.expose_secret().to_string(),
                ),
            ],
        )
        .await?;
        Ok(ClientCallOutput::success(PayWithCardResponse {
            paid: true,
            approval_code: Some("KTX-APR-1".to_string()),
        }))
    }

    async fn reserve_info(
        &mut self,
        _session: &SessionSnapshot,
        _request: &ReserveInfoRequest,
    ) -> SrtResult<ClientCallOutput<ReserveInfoResponse>> {
        Err(SrtProviderError::UnsupportedOperation {
            operation: "reserve_info",
        })
    }

    async fn refund(
        &mut self,
        _session: &SessionSnapshot,
        request: &RefundRequest,
    ) -> SrtResult<ClientCallOutput<RefundResponse>> {
        self.maybe_fail(ProviderOperation::Refund)?;
        self.maybe_live_form(
            ProviderOperation::Refund,
            "/classes/com.korail.mobile.refunds.RefundsRequest",
            vec![("txtPnrNo".to_string(), request.reservation_id.clone())],
        )
        .await?;
        Ok(ClientCallOutput::success(RefundResponse { refunded: true }))
    }

    async fn clear(
        &mut self,
        _request: &ClearRequest,
    ) -> SrtResult<ClientCallOutput<ClearResponse>> {
        self.maybe_fail(ProviderOperation::Clear)?;
        Ok(ClientCallOutput::success(ClearResponse { cleared: true }))
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

fn trim_trailing_slash(raw: String) -> String {
    raw.trim_end_matches('/').to_string()
}

fn encode_form_pairs(form: &[(String, String)]) -> String {
    let mut serializer = form_urlencoded::Serializer::new(String::new());
    for (key, value) in form {
        serializer.append_pair(key.as_str(), value.as_str());
    }
    serializer.finish()
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::providers::srt::{LoginAccountType, LoginRequest};

    #[tokio::test]
    async fn ktx_live_provider_client_use_inside_tokio_does_not_require_blocking_reqwest() {
        let join = tokio::spawn(async move {
            let mut client =
                ReqwestKtxClient::live_with_timeout("http://127.0.0.1:1", Duration::from_millis(5));

            client
                .login(&LoginRequest {
                    account_type: LoginAccountType::MembershipNumber,
                    account_identifier: "member-1".to_string(),
                    password: SecretString::new("password".to_string().into_boxed_str()),
                })
                .await
        });

        let result = join.await.expect("client task should not panic");
        assert!(matches!(result, Err(SrtProviderError::Transport { .. })));
    }
}
