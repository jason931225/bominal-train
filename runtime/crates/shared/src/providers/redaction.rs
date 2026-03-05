const SENSITIVE_PROVIDER_FIELDS: &[&str] = &[
    "password",
    "password_ciphertext",
    "identity_ciphertext",
    "card_number",
    "card_number_ciphertext",
    "card_cvc",
    "card_cvc_ciphertext",
    "pan",
    "cvv",
    "token",
    "access_token",
    "refresh_token",
    "session",
    "session_cookie",
];

pub fn sensitive_provider_fields() -> &'static [&'static str] {
    SENSITIVE_PROVIDER_FIELDS
}

pub fn should_redact_provider_field(field_name: &str) -> bool {
    let normalized = field_name.trim().to_ascii_lowercase();
    SENSITIVE_PROVIDER_FIELDS
        .iter()
        .any(|value| normalized == *value || normalized.ends_with(value))
}
