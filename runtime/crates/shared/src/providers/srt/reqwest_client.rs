use std::collections::HashMap;

use chrono::{Duration, Utc};
use secrecy::SecretString;

use crate::providers::ProviderOperation;

use super::{
    ClientCallOutput, SrtClient, SrtProviderError, SrtResult,
    netfunnel::NetfunnelStatus,
    payment::{PayWithCardRequest, PayWithCardResponse},
    reservation::{
        CancelRequest, CancelResponse, GetReservationsRequest, GetReservationsResponse,
        RefundRequest, RefundResponse, ReserveInfoRequest, ReserveInfoResponse, ReserveRequest,
        ReserveResponse, ReserveStandbyOptionSettingsRequest, ReserveStandbyOptionSettingsResponse,
        ReserveStandbyRequest, ReserveStandbyResponse, SrtReservation,
    },
    search::{SearchTrainRequest, SearchTrainResponse, SrtTrain},
    session::{SessionCookie, SessionMaterial, SessionSnapshot},
    ticket::{SrtTicket, TicketInfoRequest, TicketInfoResponse},
    types::{
        ClearRequest, ClearResponse, LoginRequest, LoginResponse, LogoutRequest, LogoutResponse,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SrtClientFailureKind {
    Transient,
    RateLimited,
    Fatal,
    SessionExpired,
    Unauthorized,
}

impl SrtClientFailureKind {
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
    kind: SrtClientFailureKind,
    remaining: usize,
}

#[derive(Debug, Clone, Default)]
pub struct ReqwestSrtClient {
    failures: HashMap<ProviderOperation, PlannedFailure>,
}

impl ReqwestSrtClient {
    pub fn deterministic() -> Self {
        Self::default()
    }

    pub fn with_failure(
        mut self,
        operation: ProviderOperation,
        kind: SrtClientFailureKind,
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

    fn canned_session() -> SessionMaterial {
        SessionMaterial {
            cookies: vec![SessionCookie::new(
                "JSESSIONID",
                SecretString::new("deterministic-cookie".into()),
            )],
            expires_at: Some(Utc::now() + Duration::minutes(30)),
        }
    }

    fn canned_login_response() -> LoginResponse {
        LoginResponse {
            membership_number: "MEM-DET-1".to_string(),
            membership_name: "Deterministic User".to_string(),
            phone_number: Some("01012341234".to_string()),
            session: Self::canned_session(),
        }
    }

    fn canned_train() -> SrtTrain {
        SrtTrain {
            train_code: "17".to_string(),
            train_number: "301".to_string(),
            dep_station_code: "0551".to_string(),
            arr_station_code: "0020".to_string(),
            dep_date: "20260305".to_string(),
            dep_time: "080000".to_string(),
            arr_date: "20260305".to_string(),
            arr_time: "103000".to_string(),
            general_seat_available: true,
            special_seat_available: false,
            standby_available: true,
        }
    }

    fn canned_reservation() -> SrtReservation {
        SrtReservation {
            reservation_id: "PNR-DET-1".to_string(),
            train_number: "301".to_string(),
            dep_station_code: "0551".to_string(),
            arr_station_code: "0020".to_string(),
            dep_date: "20260305".to_string(),
            dep_time: "080000".to_string(),
            arr_time: "103000".to_string(),
            seat_count: 1,
            total_cost: 55_000,
            paid: false,
            waiting: false,
        }
    }

    fn canned_ticket() -> SrtTicket {
        SrtTicket {
            reservation_id: "PNR-DET-1".to_string(),
            car: Some("4".to_string()),
            seat: Some("8A".to_string()),
            seat_class: "general".to_string(),
            passenger_type: "adult".to_string(),
            price: 55_000,
            discount: 0,
            waiting: false,
        }
    }
}

impl SrtClient for ReqwestSrtClient {
    fn login(&mut self, _request: &LoginRequest) -> SrtResult<ClientCallOutput<LoginResponse>> {
        self.maybe_fail(ProviderOperation::Login)?;
        Ok(ClientCallOutput::success(Self::canned_login_response()))
    }

    fn logout(
        &mut self,
        _session: &SessionSnapshot,
        _request: &LogoutRequest,
    ) -> SrtResult<ClientCallOutput<LogoutResponse>> {
        self.maybe_fail(ProviderOperation::Logout)?;
        Ok(ClientCallOutput::success(LogoutResponse {
            logged_out: true,
        }))
    }

    fn search_train(
        &mut self,
        _session: &SessionSnapshot,
        _request: &SearchTrainRequest,
    ) -> SrtResult<ClientCallOutput<SearchTrainResponse>> {
        self.maybe_fail(ProviderOperation::SearchTrain)?;
        Ok(ClientCallOutput::success(SearchTrainResponse {
            trains: vec![Self::canned_train()],
            netfunnel_status: NetfunnelStatus::Pass,
        }))
    }

    fn reserve(
        &mut self,
        _session: &SessionSnapshot,
        _request: &ReserveRequest,
    ) -> SrtResult<ClientCallOutput<ReserveResponse>> {
        self.maybe_fail(ProviderOperation::Reserve)?;
        Ok(ClientCallOutput::success(ReserveResponse {
            reservation: Self::canned_reservation(),
        }))
    }

    fn reserve_standby(
        &mut self,
        _session: &SessionSnapshot,
        _request: &ReserveStandbyRequest,
    ) -> SrtResult<ClientCallOutput<ReserveStandbyResponse>> {
        self.maybe_fail(ProviderOperation::ReserveStandby)?;
        Ok(ClientCallOutput::success(ReserveStandbyResponse {
            reservation: Self::canned_reservation(),
        }))
    }

    fn reserve_standby_option_settings(
        &mut self,
        _session: &SessionSnapshot,
        _request: &ReserveStandbyOptionSettingsRequest,
    ) -> SrtResult<ClientCallOutput<ReserveStandbyOptionSettingsResponse>> {
        self.maybe_fail(ProviderOperation::ReserveStandbyOptionSettings)?;
        Ok(ClientCallOutput::success(
            ReserveStandbyOptionSettingsResponse { updated: true },
        ))
    }

    fn get_reservations(
        &mut self,
        _session: &SessionSnapshot,
        _request: &GetReservationsRequest,
    ) -> SrtResult<ClientCallOutput<GetReservationsResponse>> {
        self.maybe_fail(ProviderOperation::GetReservations)?;
        Ok(ClientCallOutput::success(GetReservationsResponse {
            reservations: vec![Self::canned_reservation()],
        }))
    }

    fn ticket_info(
        &mut self,
        _session: &SessionSnapshot,
        _request: &TicketInfoRequest,
    ) -> SrtResult<ClientCallOutput<TicketInfoResponse>> {
        self.maybe_fail(ProviderOperation::TicketInfo)?;
        Ok(ClientCallOutput::success(TicketInfoResponse {
            tickets: vec![Self::canned_ticket()],
        }))
    }

    fn cancel(
        &mut self,
        _session: &SessionSnapshot,
        _request: &CancelRequest,
    ) -> SrtResult<ClientCallOutput<CancelResponse>> {
        self.maybe_fail(ProviderOperation::Cancel)?;
        Ok(ClientCallOutput::success(CancelResponse { canceled: true }))
    }

    fn pay_with_card(
        &mut self,
        _session: &SessionSnapshot,
        _request: &PayWithCardRequest,
    ) -> SrtResult<ClientCallOutput<PayWithCardResponse>> {
        self.maybe_fail(ProviderOperation::PayWithCard)?;
        Ok(ClientCallOutput::success(PayWithCardResponse {
            paid: true,
            approval_code: Some("APR-DET-1".to_string()),
        }))
    }

    fn reserve_info(
        &mut self,
        _session: &SessionSnapshot,
        _request: &ReserveInfoRequest,
    ) -> SrtResult<ClientCallOutput<ReserveInfoResponse>> {
        self.maybe_fail(ProviderOperation::ReserveInfo)?;
        Ok(ClientCallOutput::success(ReserveInfoResponse {
            reservation: Some(Self::canned_reservation()),
            refundable: true,
        }))
    }

    fn refund(
        &mut self,
        _session: &SessionSnapshot,
        _request: &RefundRequest,
    ) -> SrtResult<ClientCallOutput<RefundResponse>> {
        self.maybe_fail(ProviderOperation::Refund)?;
        Ok(ClientCallOutput::success(RefundResponse { refunded: true }))
    }

    fn clear(&mut self, _request: &ClearRequest) -> SrtResult<ClientCallOutput<ClearResponse>> {
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
