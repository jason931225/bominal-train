use serde_json::Value;

use super::ProviderJobsError;

pub(super) fn canonical_provider(raw: &str) -> Option<&'static str> {
    match normalize_token(raw).as_str() {
        "srt" => Some("srt"),
        "ktx" => Some("ktx"),
        _ => None,
    }
}

pub(super) fn canonical_operation_name(raw: &str) -> Option<&'static str> {
    let normalized = normalize_token(raw);
    match normalized.as_str() {
        "login" => return Some("login"),
        "logout" => return Some("logout"),
        "search" | "search_train" | "train_search" => return Some("search_train"),
        "reserve" | "book" => return Some("reserve"),
        "reserve_standby" | "standby" | "waitlist" => return Some("reserve_standby"),
        "reserve_standby_option_settings" | "standby_option_settings" => {
            return Some("reserve_standby_option_settings");
        }
        "get_reservations" | "reservations" | "list_reservations" | "reservation_list" => {
            return Some("get_reservations");
        }
        "ticket_info" | "ticket" | "tickets" => return Some("ticket_info"),
        "cancel" => return Some("cancel"),
        "pay" | "payment" | "pay_with_card" | "train_pay" => return Some("pay_with_card"),
        "reserve_info" => return Some("reserve_info"),
        "refund" => return Some("refund"),
        "clear" => return Some("clear"),
        _ => {}
    }

    if normalized.contains("standby") && normalized.contains("option") {
        return Some("reserve_standby_option_settings");
    }
    if normalized.contains("standby") {
        return Some("reserve_standby");
    }
    if normalized.contains("search") {
        return Some("search_train");
    }
    if normalized.contains("reservation") && normalized.contains("list") {
        return Some("get_reservations");
    }
    if normalized.contains("ticket") {
        return Some("ticket_info");
    }
    if normalized.contains("pay") {
        return Some("pay_with_card");
    }
    if normalized.contains("reserve") && normalized.contains("info") {
        return Some("reserve_info");
    }
    if normalized.contains("refund") {
        return Some("refund");
    }
    if normalized.contains("reserve") {
        return Some("reserve");
    }
    if normalized.contains("login") {
        return Some("login");
    }
    if normalized.contains("logout") {
        return Some("logout");
    }
    if normalized.contains("cancel") {
        return Some("cancel");
    }
    if normalized.contains("clear") {
        return Some("clear");
    }

    None
}

pub(super) fn validate_runtime_payload_contract(
    operation: &str,
    payload: &Value,
) -> Result<(), ProviderJobsError> {
    let Value::Object(_) = payload else {
        return Err(ProviderJobsError::ValidationFailed);
    };

    if contains_inline_secret_material(payload) {
        return Err(ProviderJobsError::ValidationFailed);
    }

    if operation != "clear" && ref_field(payload, "subject_ref").is_none() {
        return Err(ProviderJobsError::ValidationFailed);
    }

    if operation == "pay_with_card"
        && (ref_field(payload, "owner_ref").is_none()
            || ref_field(payload, "payment_method_ref").is_none())
    {
        return Err(ProviderJobsError::ValidationFailed);
    }

    Ok(())
}

fn ref_field<'a>(payload: &'a Value, key: &str) -> Option<&'a str> {
    payload
        .get(key)
        .and_then(Value::as_str)
        .or_else(|| {
            payload
                .get("refs")
                .and_then(Value::as_object)
                .and_then(|refs| refs.get(key))
                .and_then(Value::as_str)
        })
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

pub(super) fn payload_user_id(payload: &Value) -> Option<&str> {
    payload
        .get("user_id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .or_else(|| ref_field(payload, "subject_ref"))
}

fn contains_inline_secret_material(value: &Value) -> bool {
    match value {
        Value::Object(map) => map.iter().any(|(key, nested)| {
            if is_inline_secret_key(key) && !nested.is_null() {
                return true;
            }
            contains_inline_secret_material(nested)
        }),
        Value::Array(items) => items.iter().any(contains_inline_secret_material),
        _ => false,
    }
}

fn is_inline_secret_key(key: &str) -> bool {
    const INLINE_SECRET_KEYS: &[&str] = &[
        "identityciphertext",
        "passwordciphertext",
        "password",
        "panciphertext",
        "expirymonthciphertext",
        "expiryyearciphertext",
        "birthorbusinessnumberciphertext",
        "cardpasswordtwodigitsciphertext",
        "cardnumber",
        "cardpasswordtwodigits",
        "cardvalidationnumber",
        "cardexpiryyymm",
        "txtpwd",
        "hidstlcrcrdno1",
        "hidvanpwd1",
        "hidathnval1",
        "hidcrdvlidtrm1",
    ];

    let normalized = normalize_token(key).replace('_', "");
    INLINE_SECRET_KEYS.contains(&normalized.as_str())
}

pub(super) fn validate_job_id(job_id: &str) -> Result<(), ProviderJobsError> {
    if job_id.trim().is_empty() {
        return Err(ProviderJobsError::ValidationFailed);
    }

    Ok(())
}

fn normalize_token(raw: &str) -> String {
    raw.trim()
        .to_ascii_lowercase()
        .replace(['.', '-', ' '], "_")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn canonical_provider_accepts_srt_and_ktx_only() {
        assert_eq!(canonical_provider("srt"), Some("srt"));
        assert_eq!(canonical_provider("KTX"), Some("ktx"));
        assert_eq!(canonical_provider(" srt "), Some("srt"));
        assert_eq!(canonical_provider("korail"), None);
    }

    #[test]
    fn canonical_operation_name_maps_aliases() {
        assert_eq!(canonical_operation_name("search"), Some("search_train"));
        assert_eq!(canonical_operation_name("train-pay"), Some("pay_with_card"));
        assert_eq!(
            canonical_operation_name("reservation list"),
            Some("get_reservations")
        );
        assert_eq!(canonical_operation_name("unknown-op"), None);
    }

    #[test]
    fn payload_contract_requires_subject_ref_for_non_clear_operations() {
        let payload = json!({
            "request": {"dep_station_code": "0551", "arr_station_code": "0020"}
        });
        assert!(matches!(
            validate_runtime_payload_contract("search_train", &payload),
            Err(ProviderJobsError::ValidationFailed)
        ));

        let payload_with_ref = json!({
            "subject_ref": "subject-1",
            "request": {"dep_station_code": "0551", "arr_station_code": "0020"}
        });
        assert!(validate_runtime_payload_contract("search_train", &payload_with_ref).is_ok());

        let clear_payload = json!({});
        assert!(validate_runtime_payload_contract("clear", &clear_payload).is_ok());
    }

    #[test]
    fn payload_contract_requires_payment_refs_for_pay_with_card() {
        let missing_refs = json!({
            "subject_ref": "subject-1",
            "request": {"reservation_id": "PNR-1"}
        });
        assert!(matches!(
            validate_runtime_payload_contract("pay_with_card", &missing_refs),
            Err(ProviderJobsError::ValidationFailed)
        ));

        let with_refs = json!({
            "subject_ref": "subject-1",
            "owner_ref": "owner-1",
            "payment_method_ref": "pm-1",
            "request": {"reservation_id": "PNR-1"}
        });
        assert!(validate_runtime_payload_contract("pay_with_card", &with_refs).is_ok());
    }

    #[test]
    fn payload_contract_rejects_inline_secret_material_recursively() {
        let payload = json!({
            "subject_ref": "subject-1",
            "request": {
                "card_number": "4111111111111111"
            }
        });
        assert!(matches!(
            validate_runtime_payload_contract("search_train", &payload),
            Err(ProviderJobsError::ValidationFailed)
        ));

        let nested_secret_payload = json!({
            "subject_ref": "subject-1",
            "request": {
                "nested": {
                    "pan_ciphertext": "ciphertext"
                }
            }
        });
        assert!(matches!(
            validate_runtime_payload_contract("search_train", &nested_secret_payload),
            Err(ProviderJobsError::ValidationFailed)
        ));
    }

    #[test]
    fn payload_user_id_prefers_explicit_user_id_then_subject_ref() {
        let with_user_id = json!({
            "user_id": "user-123",
            "subject_ref": "subject-456"
        });
        assert_eq!(payload_user_id(&with_user_id), Some("user-123"));

        let with_subject_ref_only = json!({
            "subject_ref": "subject-456"
        });
        assert_eq!(payload_user_id(&with_subject_ref_only), Some("subject-456"));
    }
}
