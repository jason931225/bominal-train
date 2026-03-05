use super::{
    capabilities::ProviderKind,
    error::ProviderResult,
    model::{
        CancelRequest, CancelResponse, ClearRequest, ClearResponse, GetReservationsRequest,
        GetReservationsResponse, LoginRequest, LoginResponse, LogoutRequest, LogoutResponse,
        PayWithCardRequest, PayWithCardResponse, RefundRequest, RefundResponse,
        ReserveInfoRequest, ReserveInfoResponse, ReserveRequest, ReserveResponse,
        ReserveStandbyOptionSettingsRequest, ReserveStandbyOptionSettingsResponse,
        ReserveStandbyRequest, ReserveStandbyResponse, SearchTrainRequest, SearchTrainResponse,
        TicketInfoRequest, TicketInfoResponse,
    },
};

pub trait ProviderAdapter {
    fn provider_kind(&self) -> ProviderKind;

    fn login(&mut self, request: LoginRequest) -> ProviderResult<LoginResponse>;

    fn logout(&mut self, request: LogoutRequest) -> ProviderResult<LogoutResponse>;

    fn search_train(&mut self, request: SearchTrainRequest) -> ProviderResult<SearchTrainResponse>;

    fn reserve(&mut self, request: ReserveRequest) -> ProviderResult<ReserveResponse>;

    fn reserve_standby(
        &mut self,
        request: ReserveStandbyRequest,
    ) -> ProviderResult<ReserveStandbyResponse>;

    fn reserve_standby_option_settings(
        &mut self,
        request: ReserveStandbyOptionSettingsRequest,
    ) -> ProviderResult<ReserveStandbyOptionSettingsResponse>;

    fn get_reservations(
        &mut self,
        request: GetReservationsRequest,
    ) -> ProviderResult<GetReservationsResponse>;

    fn ticket_info(&mut self, request: TicketInfoRequest) -> ProviderResult<TicketInfoResponse>;

    fn cancel(&mut self, request: CancelRequest) -> ProviderResult<CancelResponse>;

    fn pay_with_card(&mut self, request: PayWithCardRequest) -> ProviderResult<PayWithCardResponse>;

    fn reserve_info(&mut self, request: ReserveInfoRequest) -> ProviderResult<ReserveInfoResponse>;

    fn refund(&mut self, request: RefundRequest) -> ProviderResult<RefundResponse>;

    fn clear(&mut self, request: ClearRequest) -> ProviderResult<ClearResponse>;
}
