use secrecy::SecretString;
use serde::{Deserialize, Serialize};

use super::session::SessionMaterial;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoginAccountType {
    MembershipNumber,
    Email,
    PhoneNumber,
}

#[derive(Debug, Clone)]
pub struct LoginRequest {
    pub account_type: LoginAccountType,
    pub account_identifier: String,
    pub password: SecretString,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub membership_number: String,
    pub membership_name: String,
    pub phone_number: Option<String>,
    pub session: SessionMaterial,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogoutRequest;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogoutResponse {
    pub logged_out: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PassengerKind {
    Adult,
    Child,
    Senior,
    Disability1To3,
    Disability4To6,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Passenger {
    pub kind: PassengerKind,
    pub count: u8,
}

impl Passenger {
    pub fn adult(count: u8) -> Self {
        Self {
            kind: PassengerKind::Adult,
            count,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SeatClassPreference {
    GeneralFirst,
    GeneralOnly,
    SpecialFirst,
    SpecialOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClearRequest;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClearResponse {
    pub cleared: bool,
}
