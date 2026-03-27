//! SRT API response parsing.
//!
//! SRT API responses follow a consistent pattern:
//! - JSON body with `resultMap[0]` containing status
//! - `strResult`: "SUCC" or "FAIL"
//! - `msgTxt`: human-readable message
//! - `msgCd`: machine-readable message code
//!
//! Payment responses use a different path: `outDataSets.dsOutput0[0].strResult`.

use serde_json::Value;

use super::super::types::ProviderError;

const RESULT_SUCCESS: &str = "SUCC";
const RESULT_FAIL: &str = "FAIL";

/// Parsed SRT API response.
#[derive(Debug, Clone)]
pub struct SrtResponse {
    json: Value,
    status: Value,
}

impl SrtResponse {
    /// Parse an SRT API response body.
    ///
    /// # Examples
    ///
    /// ```
    /// use bominal_service::providers::srt::response::SrtResponse;
    /// let body = r#"{"resultMap":[{"strResult":"SUCC","msgTxt":"ok","msgCd":""}]}"#;
    /// let resp = SrtResponse::parse(body).unwrap();
    /// assert!(resp.is_success());
    /// ```
    pub fn parse(body: &str) -> Result<Self, ProviderError> {
        let json: Value =
            serde_json::from_str(body).map_err(|_| ProviderError::UnexpectedResponse {
                status: 200,
                body: body.to_string(),
            })?;

        if let Some(result_map) = json.get("resultMap").and_then(|v| v.get(0)) {
            return Ok(Self {
                json: json.clone(),
                status: result_map.clone(),
            });
        }

        if let (Some(code), Some(msg)) = (json.get("ErrorCode"), json.get("ErrorMsg")) {
            let code_str = code.as_str().unwrap_or("");
            let msg_str = msg.as_str().unwrap_or("");
            return Err(ProviderError::UnexpectedResponse {
                status: 200,
                body: format!("[{code_str}]: {msg_str}"),
            });
        }

        Err(ProviderError::UnexpectedResponse {
            status: 200,
            body: body.to_string(),
        })
    }

    /// Whether the response indicates success.
    pub fn is_success(&self) -> bool {
        self.status
            .get("strResult")
            .and_then(|v| v.as_str())
            .map(|s| s == RESULT_SUCCESS)
            .unwrap_or(false)
    }

    /// The human-readable message.
    pub fn message(&self) -> &str {
        self.status
            .get("msgTxt")
            .and_then(|v| v.as_str())
            .unwrap_or("")
    }

    /// The machine-readable message code.
    pub fn message_code(&self) -> &str {
        self.status
            .get("msgCd")
            .and_then(|v| v.as_str())
            .unwrap_or("")
    }

    /// Access the full parsed JSON.
    pub fn json(&self) -> &Value {
        &self.json
    }

    /// Get a nested value from the JSON.
    pub fn get(&self, pointer: &str) -> Option<&Value> {
        self.json.pointer(pointer)
    }
}

/// Parse an SRT payment response.
/// Payment uses a different JSON path: `outDataSets.dsOutput0[0].strResult`.
pub fn parse_payment_response(body: &str) -> Result<(), ProviderError> {
    let json: Value =
        serde_json::from_str(body).map_err(|_| ProviderError::UnexpectedResponse {
            status: 200,
            body: body.to_string(),
        })?;

    let result = json
        .pointer("/outDataSets/dsOutput0/0/strResult")
        .and_then(|v| v.as_str());

    match result {
        Some(RESULT_SUCCESS) => Ok(()),
        Some(RESULT_FAIL) => {
            let msg = json
                .pointer("/outDataSets/dsOutput0/0/msgTxt")
                .and_then(|v| v.as_str())
                .unwrap_or("Payment failed");
            Err(ProviderError::UnexpectedResponse {
                status: 200,
                body: msg.to_string(),
            })
        }
        _ => Err(ProviderError::UnexpectedResponse {
            status: 200,
            body: body.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_success_response() {
        let body = r#"{"resultMap":[{"strResult":"SUCC","msgTxt":"조회 되었습니다","msgCd":""}],"outDataSets":{"dsOutput1":[]}}"#;
        let resp = SrtResponse::parse(body).unwrap();
        assert!(resp.is_success());
        assert_eq!(resp.message(), "조회 되었습니다");
        assert_eq!(resp.message_code(), "");
    }

    #[test]
    fn parse_fail_response() {
        let body =
            r#"{"resultMap":[{"strResult":"FAIL","msgTxt":"잔여석없음","msgCd":"WRG000000"}]}"#;
        let resp = SrtResponse::parse(body).unwrap();
        assert!(!resp.is_success());
        assert_eq!(resp.message(), "잔여석없음");
        assert_eq!(resp.message_code(), "WRG000000");
    }

    #[test]
    fn parse_error_code_response() {
        let body = r#"{"ErrorCode":"NET000001","ErrorMsg":"Invalid netfunnel key"}"#;
        let result = SrtResponse::parse(body);
        assert!(result.is_err());
    }

    #[test]
    fn parse_invalid_json() {
        let result = SrtResponse::parse("not json");
        assert!(result.is_err());
    }

    #[test]
    fn parse_netfunnel_error_code() {
        let body = r#"{"resultMap":[{"strResult":"FAIL","msgTxt":"정상적인 경로로 접근 부탁드립니다","msgCd":"NET000001"}]}"#;
        let resp = SrtResponse::parse(body).unwrap();
        assert!(!resp.is_success());
        assert_eq!(resp.message_code(), "NET000001");
    }

    #[test]
    fn payment_success() {
        let body = r#"{"outDataSets":{"dsOutput0":[{"strResult":"SUCC","msgTxt":"결제 완료"}]}}"#;
        assert!(parse_payment_response(body).is_ok());
    }

    #[test]
    fn payment_fail() {
        let body = r#"{"outDataSets":{"dsOutput0":[{"strResult":"FAIL","msgTxt":"카드 오류"}]}}"#;
        let result = parse_payment_response(body);
        assert!(result.is_err());
    }
}
