use super::{
    capabilities::ProviderKind,
    error::ProviderResult,
    model::{
        CancelRequest, CancelResponse, ClearRequest, ClearResponse, GetReservationsRequest,
        GetReservationsResponse, LoginRequest, LoginResponse, LogoutRequest, LogoutResponse,
        PayWithCardRequest, PayWithCardResponse, RefundRequest, RefundResponse, ReserveInfoRequest,
        ReserveInfoResponse, ReserveRequest, ReserveResponse, ReserveStandbyOptionSettingsRequest,
        ReserveStandbyOptionSettingsResponse, ReserveStandbyRequest, ReserveStandbyResponse,
        SearchTrainRequest, SearchTrainResponse, TicketInfoRequest, TicketInfoResponse,
    },
};

#[allow(async_fn_in_trait)]
pub trait ProviderAdapter {
    fn provider_kind(&self) -> ProviderKind;

    async fn login(&mut self, request: LoginRequest) -> ProviderResult<LoginResponse>;

    async fn logout(&mut self, request: LogoutRequest) -> ProviderResult<LogoutResponse>;

    async fn search_train(
        &mut self,
        request: SearchTrainRequest,
    ) -> ProviderResult<SearchTrainResponse>;

    async fn reserve(&mut self, request: ReserveRequest) -> ProviderResult<ReserveResponse>;

    async fn reserve_standby(
        &mut self,
        request: ReserveStandbyRequest,
    ) -> ProviderResult<ReserveStandbyResponse>;

    async fn reserve_standby_option_settings(
        &mut self,
        request: ReserveStandbyOptionSettingsRequest,
    ) -> ProviderResult<ReserveStandbyOptionSettingsResponse>;

    async fn get_reservations(
        &mut self,
        request: GetReservationsRequest,
    ) -> ProviderResult<GetReservationsResponse>;

    async fn ticket_info(
        &mut self,
        request: TicketInfoRequest,
    ) -> ProviderResult<TicketInfoResponse>;

    async fn cancel(&mut self, request: CancelRequest) -> ProviderResult<CancelResponse>;

    async fn pay_with_card(
        &mut self,
        request: PayWithCardRequest,
    ) -> ProviderResult<PayWithCardResponse>;

    async fn reserve_info(
        &mut self,
        request: ReserveInfoRequest,
    ) -> ProviderResult<ReserveInfoResponse>;

    async fn refund(&mut self, request: RefundRequest) -> ProviderResult<RefundResponse>;

    async fn clear(&mut self, request: ClearRequest) -> ProviderResult<ClearResponse>;
}
