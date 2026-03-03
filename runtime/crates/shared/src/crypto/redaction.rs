use std::collections::BTreeMap;

use serde_json::{Map, Value};

pub const REDACTED_VALUE: &str = "[REDACTED]";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedactionMode {
    Mask,
    Remove,
}

pub fn redact_json(input: &Value, mode: RedactionMode) -> Value {
    redact_value(input, mode)
}

pub fn redact_pairs(
    input: &BTreeMap<String, String>,
    mode: RedactionMode,
) -> BTreeMap<String, String> {
    let mut redacted = BTreeMap::new();
    for (key, value) in input {
        if is_sensitive_key(key) {
            if mode == RedactionMode::Mask {
                redacted.insert(key.clone(), REDACTED_VALUE.to_string());
            }
            continue;
        }

        redacted.insert(key.clone(), value.clone());
    }
    redacted
}

fn redact_value(input: &Value, mode: RedactionMode) -> Value {
    match input {
        Value::Object(map) => Value::Object(redact_object(map, mode)),
        Value::Array(values) => {
            let redacted = values
                .iter()
                .map(|value| redact_value(value, mode))
                .collect();
            Value::Array(redacted)
        }
        _ => input.clone(),
    }
}

fn redact_object(map: &Map<String, Value>, mode: RedactionMode) -> Map<String, Value> {
    let mut redacted = Map::new();

    for (key, value) in map {
        if is_sensitive_key(key) {
            if mode == RedactionMode::Mask {
                redacted.insert(key.clone(), Value::String(REDACTED_VALUE.to_string()));
            }
            continue;
        }

        redacted.insert(key.clone(), redact_value(value, mode));
    }

    redacted
}

fn is_sensitive_key(key: &str) -> bool {
    let normalized = key.to_ascii_lowercase();
    const SENSITIVE_TOKENS: &[&str] = &[
        "password",
        "passcode",
        "secret",
        "token",
        "authorization",
        "auth_header",
        "cookie",
        "session",
        "api_key",
        "apikey",
        "private_key",
        "card_number",
        "pan",
        "cvv",
        "cvc",
        "iban",
        "account_number",
    ];

    SENSITIVE_TOKENS
        .iter()
        .any(|token| normalized == *token || normalized.contains(token))
}

#[cfg(test)]
mod tests {
    use super::{REDACTED_VALUE, RedactionMode, redact_json};

    #[test]
    fn masks_sensitive_keys() {
        let value = serde_json::json!({
            "token": "abc",
            "nested": {"password": "secret", "safe": "ok"}
        });

        let redacted = redact_json(&value, RedactionMode::Mask);
        assert_eq!(redacted["token"], REDACTED_VALUE);
        assert_eq!(redacted["nested"]["password"], REDACTED_VALUE);
        assert_eq!(redacted["nested"]["safe"], "ok");
    }

    #[test]
    fn removes_sensitive_keys() {
        let value = serde_json::json!({
            "token": "abc",
            "nested": {"password": "secret", "safe": "ok"}
        });

        let redacted = redact_json(&value, RedactionMode::Remove);
        assert!(redacted.get("token").is_none());
        assert!(redacted["nested"].get("password").is_none());
        assert_eq!(redacted["nested"]["safe"], "ok");
    }
}
