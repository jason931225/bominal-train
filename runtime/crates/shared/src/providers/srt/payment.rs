use secrecy::SecretString;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CardIdentityType {
    Personal,
    Corporate,
}

#[derive(Debug, Clone)]
pub struct PayWithCardRequest {
    pub reservation_id: String,
    pub card_identity_type: CardIdentityType,
    pub card_number: SecretString,
    pub card_password_two_digits: SecretString,
    pub card_validation_number: SecretString,
    pub card_expiry_yymm: SecretString,
    pub installment_months: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PayWithCardResponse {
    pub paid: bool,
    pub approval_code: Option<String>,
}
