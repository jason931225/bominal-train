use url::form_urlencoded;

const CURSOR_VERSION: u64 = 1;
const DEFAULT_EVENT_LIMIT: usize = 100;
const MAX_EVENT_LIMIT: usize = 200;
const MAX_CURSOR_LEN: usize = 512;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RuntimeEventRequest {
    pub(super) after_id: i64,
    pub(super) limit: usize,
}

pub(super) fn parse_runtime_event_request(
    raw_query: Option<&str>,
    job_id: &str,
) -> Result<RuntimeEventRequest, &'static str> {
    let mut cursor: Option<String> = None;
    let mut limit_raw: Option<String> = None;

    if let Some(query) = raw_query {
        for (key, value) in form_urlencoded::parse(query.as_bytes()) {
            let key = key.as_ref();
            let value = value.into_owned();
            match key {
                "cursor" => {
                    if cursor.is_some() {
                        return Err("duplicate query parameter");
                    }
                    cursor = Some(value);
                }
                "limit" => {
                    if limit_raw.is_some() {
                        return Err("duplicate query parameter");
                    }
                    limit_raw = Some(value);
                }
                _ => return Err("unknown query parameter"),
            }
        }
    }

    let limit = parse_limit(limit_raw.as_deref())?;
    let after_id = decode_cursor(cursor.as_deref(), job_id)?;
    Ok(RuntimeEventRequest { after_id, limit })
}

pub(super) fn encode_runtime_event_cursor(
    job_id: &str,
    after_id: i64,
) -> Result<String, &'static str> {
    if after_id < 0 {
        return Err("invalid cursor token payload");
    }
    let payload = serde_json::json!({
        "v": CURSOR_VERSION,
        "job_id": job_id.trim(),
        "after_id": after_id,
    });
    Ok(encode_base64url(payload.to_string().as_bytes()))
}

fn parse_limit(raw_limit: Option<&str>) -> Result<usize, &'static str> {
    let Some(raw_limit) = raw_limit else {
        return Ok(DEFAULT_EVENT_LIMIT);
    };
    let trimmed = raw_limit.trim();
    if trimmed.is_empty() {
        return Err("invalid limit query parameter");
    }
    let parsed = trimmed
        .parse::<usize>()
        .map_err(|_| "invalid limit query parameter")?;
    Ok(parsed.clamp(1, MAX_EVENT_LIMIT))
}

fn decode_cursor(raw_cursor: Option<&str>, job_id: &str) -> Result<i64, &'static str> {
    let Some(raw_cursor) = raw_cursor.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(0);
    };

    if raw_cursor.len() > MAX_CURSOR_LEN {
        return Err("invalid cursor token");
    }

    let bytes = decode_base64url(raw_cursor)?;
    let payload =
        serde_json::from_slice::<serde_json::Value>(&bytes).map_err(|_| "invalid cursor token")?;
    let version = payload
        .get("v")
        .and_then(serde_json::Value::as_u64)
        .ok_or("invalid cursor token payload")?;
    if version != CURSOR_VERSION {
        return Err("invalid cursor token version");
    }

    let payload_job_id = payload
        .get("job_id")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or("invalid cursor token payload")?;
    if payload_job_id != job_id.trim() {
        return Err("cursor token job mismatch");
    }

    let after_id = payload
        .get("after_id")
        .and_then(serde_json::Value::as_i64)
        .ok_or("invalid cursor token payload")?;
    if after_id < 0 {
        return Err("invalid cursor token payload");
    }
    Ok(after_id)
}

fn encode_base64url(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::with_capacity((bytes.len() * 4).div_ceil(3));
    let mut index = 0usize;
    while index + 3 <= bytes.len() {
        let chunk = ((bytes[index] as u32) << 16)
            | ((bytes[index + 1] as u32) << 8)
            | bytes[index + 2] as u32;
        out.push(ALPHABET[((chunk >> 18) & 0x3f) as usize] as char);
        out.push(ALPHABET[((chunk >> 12) & 0x3f) as usize] as char);
        out.push(ALPHABET[((chunk >> 6) & 0x3f) as usize] as char);
        out.push(ALPHABET[(chunk & 0x3f) as usize] as char);
        index += 3;
    }

    let remaining = bytes.len() - index;
    if remaining == 1 {
        let chunk = (bytes[index] as u32) << 16;
        out.push(ALPHABET[((chunk >> 18) & 0x3f) as usize] as char);
        out.push(ALPHABET[((chunk >> 12) & 0x3f) as usize] as char);
    } else if remaining == 2 {
        let chunk = ((bytes[index] as u32) << 16) | ((bytes[index + 1] as u32) << 8);
        out.push(ALPHABET[((chunk >> 18) & 0x3f) as usize] as char);
        out.push(ALPHABET[((chunk >> 12) & 0x3f) as usize] as char);
        out.push(ALPHABET[((chunk >> 6) & 0x3f) as usize] as char);
    }
    out
}

fn decode_base64url(input: &str) -> Result<Vec<u8>, &'static str> {
    let mut out = Vec::with_capacity((input.len() * 3) / 4 + 3);
    let mut buffer = 0u32;
    let mut bits = 0usize;

    for byte in input.bytes() {
        let sextet = match byte {
            b'A'..=b'Z' => (byte - b'A') as u32,
            b'a'..=b'z' => (byte - b'a' + 26) as u32,
            b'0'..=b'9' => (byte - b'0' + 52) as u32,
            b'-' => 62,
            b'_' => 63,
            _ => return Err("invalid cursor token"),
        };

        buffer = (buffer << 6) | sextet;
        bits += 6;

        if bits >= 8 {
            bits -= 8;
            out.push(((buffer >> bits) & 0xff) as u8);
            buffer &= (1u32 << bits) - 1;
        }
    }

    if bits > 0 && (buffer & ((1u32 << bits) - 1)) != 0 {
        return Err("invalid cursor token");
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_runtime_event_request_defaults() {
        let parsed = parse_runtime_event_request(None, "job-1").expect("parse should succeed");
        assert_eq!(
            parsed,
            RuntimeEventRequest {
                after_id: 0,
                limit: 100
            }
        );
    }

    #[test]
    fn parse_runtime_event_request_rejects_unknown_and_duplicate_keys() {
        let unknown = parse_runtime_event_request(Some("since_id=10"), "job-1");
        assert!(matches!(unknown, Err("unknown query parameter")));

        let dup = parse_runtime_event_request(Some("limit=10&limit=20"), "job-1");
        assert!(matches!(dup, Err("duplicate query parameter")));
    }

    #[test]
    fn parse_runtime_event_request_rejects_invalid_limit() {
        let result = parse_runtime_event_request(Some("limit=abc"), "job-1");
        assert!(matches!(result, Err("invalid limit query parameter")));
    }

    #[test]
    fn cursor_roundtrip_and_limits_work() {
        let cursor = encode_runtime_event_cursor("job-9", 42).expect("cursor encoding should work");
        let parsed = parse_runtime_event_request(
            Some(format!("cursor={cursor}&limit=9999").as_str()),
            "job-9",
        )
        .expect("cursor parsing should work");
        assert_eq!(parsed.after_id, 42);
        assert_eq!(parsed.limit, 200);
    }

    #[test]
    fn cursor_decode_rejects_job_mismatch() {
        let cursor = encode_runtime_event_cursor("job-9", 99).expect("cursor encoding should work");
        let result =
            parse_runtime_event_request(Some(format!("cursor={cursor}").as_str()), "job-10");
        assert!(matches!(result, Err("cursor token job mismatch")));
    }
}
