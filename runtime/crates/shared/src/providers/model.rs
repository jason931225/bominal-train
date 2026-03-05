pub use super::srt::{
    CancelRequest, CancelResponse, CardIdentityType, ClearRequest, ClearResponse,
    GetReservationsRequest, GetReservationsResponse, LoginAccountType, LoginRequest,
    LoginResponse, LogoutRequest, LogoutResponse, Passenger, PassengerKind, PayWithCardRequest,
    PayWithCardResponse, RefundRequest, RefundResponse, ReserveInfoRequest, ReserveInfoResponse,
    ReserveRequest, ReserveResponse, ReserveStandbyOptionSettingsRequest,
    ReserveStandbyOptionSettingsResponse, ReserveStandbyRequest, ReserveStandbyResponse,
    SearchTrainRequest, SearchTrainResponse, SecretString, SeatClassPreference, SessionSnapshot,
    SrtClientFailureKind, SrtOperationRequest, SrtOperationResponse, TicketInfoRequest,
    TicketInfoResponse,
};

pub type ProviderOperationRequest = SrtOperationRequest;
pub type ProviderOperationResponse = SrtOperationResponse;
