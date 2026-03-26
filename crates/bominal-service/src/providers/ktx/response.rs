//! KTX/Korail API response parsing.
//!
//! KTX responses use a flat JSON structure:
//! - `strResult`: "SUCC" or "FAIL"
//! - `h_msg_cd`: machine-readable code
//! - `h_msg_txt`: human-readable message

use serde_json::Value;

use super::super::types::ProviderError;

const RESULT_SUCCESS: &str = "SUCC";

/// Parsed KTX API response.
#[derive(Debug, Clone)]
pub struct KtxResponse {
    json: Value,
}

impl KtxResponse {
    /// Parse a KTX API response body.
    ///
    /// # Examples
    ///
    /// ```
    /// use bominal_service::providers::ktx::response::KtxResponse;
    /// let body = r#"{"strResult":"SUCC","h_msg_cd":"IRZ000001","h_msg_txt":"조회 성공"}"#;
    /// let resp = KtxResponse::parse(body).unwrap();
    /// assert!(resp.is_success());
    /// ```
    pub fn parse(body: &str) -> Result<Self, ProviderError> {
        let json: Value =
            serde_json::from_str(body).map_err(|_| ProviderError::UnexpectedResponse {
                status: 200,
                body: body.to_string(),
            })?;

        Ok(Self { json })
    }

    /// Whether the response indicates success.
    pub fn is_success(&self) -> bool {
        self.json
            .get("strResult")
            .and_then(|v| v.as_str())
            .map(|s| s == RESULT_SUCCESS)
            .unwrap_or(false)
    }

    /// The human-readable message.
    pub fn message(&self) -> &str {
        self.json
            .get("h_msg_txt")
            .and_then(|v| v.as_str())
            .unwrap_or("")
    }

    /// The machine-readable message code.
    pub fn message_code(&self) -> &str {
        self.json
            .get("h_msg_cd")
            .and_then(|v| v.as_str())
            .unwrap_or("")
    }

    /// Access the full parsed JSON.
    pub fn json(&self) -> &Value {
        &self.json
    }

    /// Get a nested value using JSON pointer syntax.
    pub fn get(&self, pointer: &str) -> Option<&Value> {
        self.json.pointer(pointer)
    }

    /// Get a string value from a top-level field.
    pub fn str_field(&self, key: &str) -> &str {
        self.json.get(key).and_then(|v| v.as_str()).unwrap_or("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_success_response() {
        let body = r#"{"strResult":"SUCC","h_msg_cd":"IRZ000001","h_msg_txt":"조회 성공","trn_infos":{"trn_info":[]}}"#;
        let resp = KtxResponse::parse(body).unwrap();
        assert!(resp.is_success());
        assert_eq!(resp.message(), "조회 성공");
        assert_eq!(resp.message_code(), "IRZ000001");
    }

    #[test]
    fn parse_fail_response() {
        let body =
            r#"{"strResult":"FAIL","h_msg_cd":"P058","h_msg_txt":"운행하는 열차가 없습니다"}"#;
        let resp = KtxResponse::parse(body).unwrap();
        assert!(!resp.is_success());
        assert_eq!(resp.message(), "운행하는 열차가 없습니다");
    }

    #[test]
    fn parse_invalid_json() {
        assert!(KtxResponse::parse("not json").is_err());
    }

    #[test]
    fn str_field_access() {
        let body = r#"{"strResult":"SUCC","strMbCrdNo":"1234567890","strCustNm":"홍길동"}"#;
        let resp = KtxResponse::parse(body).unwrap();
        assert_eq!(resp.str_field("strMbCrdNo"), "1234567890");
        assert_eq!(resp.str_field("strCustNm"), "홍길동");
        assert_eq!(resp.str_field("missing"), "");
    }
}
