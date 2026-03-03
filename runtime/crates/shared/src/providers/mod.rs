pub mod srt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProviderKind {
    Srt,
    Ktx,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProviderOperation {
    Login,
    Logout,
    SearchTrain,
    Reserve,
    ReserveStandby,
    ReserveStandbyOptionSettings,
    GetReservations,
    TicketInfo,
    Cancel,
    PayWithCard,
    ReserveInfo,
    Refund,
    Clear,
}

pub trait ProviderAdapter {
    fn provider_kind(&self) -> ProviderKind;

    fn login(&mut self, request: srt::LoginRequest) -> srt::SrtResult<srt::LoginResponse>;

    fn logout(&mut self, request: srt::LogoutRequest) -> srt::SrtResult<srt::LogoutResponse>;

    fn search_train(
        &mut self,
        request: srt::SearchTrainRequest,
    ) -> srt::SrtResult<srt::SearchTrainResponse>;

    fn reserve(&mut self, request: srt::ReserveRequest) -> srt::SrtResult<srt::ReserveResponse>;

    fn reserve_standby(
        &mut self,
        request: srt::ReserveStandbyRequest,
    ) -> srt::SrtResult<srt::ReserveStandbyResponse>;

    fn reserve_standby_option_settings(
        &mut self,
        request: srt::ReserveStandbyOptionSettingsRequest,
    ) -> srt::SrtResult<srt::ReserveStandbyOptionSettingsResponse>;

    fn get_reservations(
        &mut self,
        request: srt::GetReservationsRequest,
    ) -> srt::SrtResult<srt::GetReservationsResponse>;

    fn ticket_info(
        &mut self,
        request: srt::TicketInfoRequest,
    ) -> srt::SrtResult<srt::TicketInfoResponse>;

    fn cancel(&mut self, request: srt::CancelRequest) -> srt::SrtResult<srt::CancelResponse>;

    fn pay_with_card(
        &mut self,
        request: srt::PayWithCardRequest,
    ) -> srt::SrtResult<srt::PayWithCardResponse>;

    fn reserve_info(
        &mut self,
        request: srt::ReserveInfoRequest,
    ) -> srt::SrtResult<srt::ReserveInfoResponse>;

    fn refund(&mut self, request: srt::RefundRequest) -> srt::SrtResult<srt::RefundResponse>;

    fn clear(&mut self, request: srt::ClearRequest) -> srt::SrtResult<srt::ClearResponse>;
}
