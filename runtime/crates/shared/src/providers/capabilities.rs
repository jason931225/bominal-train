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

impl ProviderOperation {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Login => "login",
            Self::Logout => "logout",
            Self::SearchTrain => "search_train",
            Self::Reserve => "reserve",
            Self::ReserveStandby => "reserve_standby",
            Self::ReserveStandbyOptionSettings => "reserve_standby_option_settings",
            Self::GetReservations => "get_reservations",
            Self::TicketInfo => "ticket_info",
            Self::Cancel => "cancel",
            Self::PayWithCard => "pay_with_card",
            Self::ReserveInfo => "reserve_info",
            Self::Refund => "refund",
            Self::Clear => "clear",
        }
    }
}
