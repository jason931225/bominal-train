//! NetFunnel queue management for SRT and KTX.
//!
//! Both providers use NetFunnel at `nf.letskorail.com` but with different
//! protocols (HTTPS vs HTTP), headers, and cache TTLs.
//!
//! The SRT reference calls `NetFunnelHelper.run()` before search and reserve,
//! injecting the returned key as `netfunnelKey` into the form body.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use tracing::{debug, warn};

use crate::types::ProviderError;

/// Parse NetFunnel response.
/// Format: `NetFunnel.gControl.result='<code>:<status>:<key1>=<val1>&<key2>=<val2>...'`
///
/// # Examples
///
/// ```
/// use bominal_provider::netfunnel::parse_response;
/// let body = "NetFunnel.gControl.result='5002:200:key=abc123&ip=1.2.3.4'";
/// let result = parse_response(body).unwrap();
/// assert_eq!(result.code, 5002);
/// assert_eq!(result.status, "200");
/// assert!(result.should_pass());
/// ```
pub fn parse_response(body: &str) -> Option<NetFunnelResult> {
    let start = body.find("result='")?;
    let after = &body[start + 8..];
    let end = after.find('\'')?;
    let inner = &after[..end];

    let parts: Vec<&str> = inner.splitn(3, ':').collect();
    if parts.len() < 3 {
        return None;
    }

    let code: u16 = parts[0].parse().ok()?;
    let status = parts[1].to_string();

    let mut params = HashMap::new();
    for pair in parts[2].split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            params.insert(k.to_string(), v.to_string());
        }
    }

    Some(NetFunnelResult {
        code,
        status,
        params,
    })
}

#[derive(Debug, Clone)]
pub struct NetFunnelResult {
    pub code: u16,
    pub status: String,
    pub params: HashMap<String, String>,
}

impl NetFunnelResult {
    /// Pass: status field (2nd position) is "200" or "502" (already completed).
    ///
    /// The NetFunnel response format is `{code}:{status}:{params}`.
    /// The reference Python checks the **status** field (2nd position),
    /// not the code field (1st position).
    pub fn should_pass(&self) -> bool {
        self.status == "200" || self.status == "502"
    }

    /// Wait: status field is "201" (queue is full, poll again).
    pub fn should_wait(&self) -> bool {
        self.status == "201"
    }

    /// The numeric code from the first position (e.g. 5002 for chkEnter response).
    pub fn response_code(&self) -> u16 {
        self.code
    }
}

// ── NetFunnel Helper ──────────────────────────────────────────────────

/// Default NetFunnel host.
const NF_HOST: &str = "nf.letskorail.com";

/// NetFunnel key cache TTL (48 seconds, matching reference).
const KEY_TTL: Duration = Duration::from_secs(48);

/// Max poll iterations before giving up on a wait (201) response.
const MAX_WAIT_POLLS: u32 = 120;

/// Poll interval for wait (201) responses.
const POLL_INTERVAL: Duration = Duration::from_secs(1);

/// OpCodes matching the reference Python `NetFunnelHelper.OP_CODE`.
const OP_GET_TID_CHK_ENTER: &str = "5101";
const OP_CHK_ENTER: &str = "5002";
const OP_SET_COMPLETE: &str = "5004";

/// Cached NetFunnel key with expiry.
struct CachedKey {
    key: String,
    acquired_at: Instant,
}

impl CachedKey {
    fn is_valid(&self) -> bool {
        self.acquired_at.elapsed() < KEY_TTL
    }
}

/// Build the URL for a NetFunnel request.
/// Includes a timestamp parameter as a cache-buster (matches reference).
fn build_nf_url(host: &str, opcode: &str, key: Option<&str>) -> String {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis();

    let mut url = format!(
        "https://{host}/ts.wseq?opcode={opcode}&nfid=0\
         &prefix=NetFunnel.gRtype%3D{opcode}%3B\
         &js=true&{ts}="
    );

    // getTidchkEnter and chkEnter include sid/aid
    if opcode == OP_GET_TID_CHK_ENTER || opcode == OP_CHK_ENTER {
        url.push_str("&sid=service_1&aid=act_10");
        if opcode == OP_CHK_ENTER
            && let Some(k) = key
        {
            url.push_str(&format!("&key={k}&ttl=1"));
        }
    } else if opcode == OP_SET_COMPLETE
        && let Some(k) = key
    {
        url.push_str(&format!("&key={k}"));
    }

    url
}

/// NetFunnel helper that acquires and caches queue-bypass keys.
///
/// Follows the exact 3-step protocol from the reference:
/// 1. `getTidchkEnter` (5101) — start the queue
/// 2. `chkEnter` (5002) — poll while waiting (status 201)
/// 3. `setComplete` (5004) — complete the funnel
pub struct NetFunnelHelper {
    cached: Option<CachedKey>,
}

impl NetFunnelHelper {
    pub fn new() -> Self {
        Self { cached: None }
    }

    /// Acquire a NetFunnel key (returns cached key if still valid).
    pub async fn run(&mut self, client: &reqwest::Client) -> Result<String, ProviderError> {
        // Return cached key if valid
        if let Some(ref cached) = self.cached
            && cached.is_valid()
        {
            return Ok(cached.key.clone());
        }

        let key = self.acquire_key(client).await?;

        self.cached = Some(CachedKey {
            key: key.clone(),
            acquired_at: Instant::now(),
        });

        Ok(key)
    }

    /// Clear the cached key (called on NET000001 error before retry).
    pub fn clear(&mut self) {
        self.cached = None;
    }

    /// Acquire a fresh key via the 3-step NetFunnel protocol.
    async fn acquire_key(&self, client: &reqwest::Client) -> Result<String, ProviderError> {
        // Step 1: getTidchkEnter (start)
        let start_result = self
            .make_request(client, OP_GET_TID_CHK_ENTER, NF_HOST, None)
            .await?;
        let mut current_key = start_result.params.get("key").cloned();
        let host = start_result
            .params
            .get("ip")
            .map(String::as_str)
            .unwrap_or(NF_HOST);

        // Step 2: Poll chkEnter while waiting (status "201")
        if start_result.should_wait() {
            let mut attempts = 0u32;

            loop {
                attempts += 1;
                if attempts >= MAX_WAIT_POLLS {
                    warn!("NetFunnel max wait polls reached ({MAX_WAIT_POLLS})");
                    return Err(ProviderError::NetFunnelBlocked);
                }
                debug!(attempt = attempts, "NetFunnel waiting, polling chkEnter...");
                tokio::time::sleep(POLL_INTERVAL).await;

                let poll = self
                    .make_request(client, OP_CHK_ENTER, host, current_key.as_deref())
                    .await?;
                if let Some(new_key) = poll.params.get("key") {
                    current_key = Some(new_key.clone());
                }
                if !poll.should_wait() {
                    break;
                }
            }
        }

        // Step 3: setComplete
        let complete = self
            .make_request(client, OP_SET_COMPLETE, host, current_key.as_deref())
            .await?;

        if complete.should_pass() {
            return Ok(current_key.unwrap_or_default());
        }

        Err(ProviderError::UnexpectedResponse {
            status: complete.code,
            body: format!(
                "NetFunnel setComplete failed: {}:{}",
                complete.code, complete.status
            ),
        })
    }

    /// Make a single NetFunnel request and parse the response.
    /// Returns the parsed result.
    async fn make_request(
        &self,
        client: &reqwest::Client,
        opcode: &str,
        host: &str,
        key: Option<&str>,
    ) -> Result<NetFunnelResult, ProviderError> {
        let url = build_nf_url(host, opcode, key);

        let resp: reqwest::Response = client
            .get(&url)
            .header("Referer", "https://app.srail.or.kr/")
            .header("X-Requested-With", "kr.co.srail.newapp")
            .send()
            .await?;

        let body: String = resp.text().await?;
        debug!(opcode, body_len = body.len(), "NetFunnel response");

        parse_response(&body).ok_or_else(|| ProviderError::UnexpectedResponse {
            status: 200,
            body: format!("Failed to parse NetFunnel response: {body}"),
        })
    }
}

impl Default for NetFunnelHelper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_pass_response() {
        // Real API format: {response_code}:{status}:{params}
        // Status "200" means pass
        let body = "NetFunnel.gControl.result='5002:200:key=abc123&ip=1.2.3.4'";
        let result = parse_response(body).unwrap();
        assert_eq!(result.code, 5002);
        assert_eq!(result.status, "200");
        assert_eq!(result.params.get("key").unwrap(), "abc123");
        assert_eq!(result.params.get("ip").unwrap(), "1.2.3.4");
        assert!(result.should_pass());
        assert!(!result.should_wait());
    }

    #[test]
    fn parse_wait_response() {
        let body = "NetFunnel.gControl.result='5002:201:nwait=5&ttl=48'";
        let result = parse_response(body).unwrap();
        assert!(result.should_wait());
        assert!(!result.should_pass());
    }

    #[test]
    fn parse_already_completed() {
        let body = "NetFunnel.gControl.result='5004:502:key=xyz'";
        let result = parse_response(body).unwrap();
        assert!(result.should_pass());
    }

    #[test]
    fn parse_invalid() {
        assert!(parse_response("garbage").is_none());
        assert!(parse_response("result='bad'").is_none());
    }

    #[test]
    fn cached_key_valid_within_ttl() {
        let cached = CachedKey {
            key: "test_key".to_string(),
            acquired_at: Instant::now(),
        };
        assert!(cached.is_valid());
    }

    #[test]
    fn cached_key_expired_after_ttl() {
        let cached = CachedKey {
            key: "test_key".to_string(),
            acquired_at: Instant::now() - KEY_TTL - Duration::from_secs(1),
        };
        assert!(!cached.is_valid());
    }

    #[test]
    fn helper_clear_removes_cache() {
        let mut helper = NetFunnelHelper::new();
        helper.cached = Some(CachedKey {
            key: "test_key".to_string(),
            acquired_at: Instant::now(),
        });
        assert!(helper.cached.is_some());
        helper.clear();
        assert!(helper.cached.is_none());
    }

    #[test]
    fn helper_default() {
        let helper = NetFunnelHelper::default();
        assert!(helper.cached.is_none());
    }
}
