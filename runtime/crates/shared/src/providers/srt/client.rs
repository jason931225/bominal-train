use super::{
    errors::SrtResult,
    payment::{PayWithCardRequest, PayWithCardResponse},
    reservation::{
        CancelRequest, CancelResponse, GetReservationsRequest, GetReservationsResponse,
        RefundRequest, RefundResponse, ReserveInfoRequest, ReserveInfoResponse, ReserveRequest,
        ReserveResponse, ReserveStandbyOptionSettingsRequest, ReserveStandbyOptionSettingsResponse,
        ReserveStandbyRequest, ReserveStandbyResponse,
    },
    search::{SearchTrainRequest, SearchTrainResponse},
    session::{SessionMaterial, SessionSnapshot},
    ticket::{TicketInfoRequest, TicketInfoResponse},
    types::{
        ClearRequest, ClearResponse, LoginRequest, LoginResponse, LogoutRequest, LogoutResponse,
    },
};

#[derive(Debug, Clone)]
pub struct ClientCallOutput<T> {
    pub payload: T,
    pub session_update: Option<SessionMaterial>,
}

impl<T> ClientCallOutput<T> {
    pub fn success(payload: T) -> Self {
        Self {
            payload,
            session_update: None,
        }
    }

    pub fn with_session_update(mut self, session_update: SessionMaterial) -> Self {
        self.session_update = Some(session_update);
        self
    }
}

pub trait SrtClient {
    fn login(&mut self, request: &LoginRequest) -> SrtResult<ClientCallOutput<LoginResponse>>;

    fn logout(
        &mut self,
        session: &SessionSnapshot,
        request: &LogoutRequest,
    ) -> SrtResult<ClientCallOutput<LogoutResponse>>;

    fn search_train(
        &mut self,
        session: &SessionSnapshot,
        request: &SearchTrainRequest,
    ) -> SrtResult<ClientCallOutput<SearchTrainResponse>>;

    fn reserve(
        &mut self,
        session: &SessionSnapshot,
        request: &ReserveRequest,
    ) -> SrtResult<ClientCallOutput<ReserveResponse>>;

    fn reserve_standby(
        &mut self,
        session: &SessionSnapshot,
        request: &ReserveStandbyRequest,
    ) -> SrtResult<ClientCallOutput<ReserveStandbyResponse>>;

    fn reserve_standby_option_settings(
        &mut self,
        session: &SessionSnapshot,
        request: &ReserveStandbyOptionSettingsRequest,
    ) -> SrtResult<ClientCallOutput<ReserveStandbyOptionSettingsResponse>>;

    fn get_reservations(
        &mut self,
        session: &SessionSnapshot,
        request: &GetReservationsRequest,
    ) -> SrtResult<ClientCallOutput<GetReservationsResponse>>;

    fn ticket_info(
        &mut self,
        session: &SessionSnapshot,
        request: &TicketInfoRequest,
    ) -> SrtResult<ClientCallOutput<TicketInfoResponse>>;

    fn cancel(
        &mut self,
        session: &SessionSnapshot,
        request: &CancelRequest,
    ) -> SrtResult<ClientCallOutput<CancelResponse>>;

    fn pay_with_card(
        &mut self,
        session: &SessionSnapshot,
        request: &PayWithCardRequest,
    ) -> SrtResult<ClientCallOutput<PayWithCardResponse>>;

    fn reserve_info(
        &mut self,
        session: &SessionSnapshot,
        request: &ReserveInfoRequest,
    ) -> SrtResult<ClientCallOutput<ReserveInfoResponse>>;

    fn refund(
        &mut self,
        session: &SessionSnapshot,
        request: &RefundRequest,
    ) -> SrtResult<ClientCallOutput<RefundResponse>>;

    fn clear(&mut self, request: &ClearRequest) -> SrtResult<ClientCallOutput<ClearResponse>>;
}
