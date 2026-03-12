//! NetFunnel queue management for SRT and KTX.
//!
//! Both providers use NetFunnel at `nf.letskorail.com` but with different
//! protocols (HTTPS vs HTTP), headers, and cache TTLs.

use std::collections::HashMap;

/// Parse NetFunnel response.
/// Format: `NetFunnel.gControl.result='<code>:<status>:<key1>=<val1>&<key2>=<val2>...'`
///
/// # Examples
///
/// ```
/// use bominal_provider::netfunnel::parse_response;
/// let body = "NetFunnel.gControl.result='200:ok:key=abc123&ip=1.2.3.4'";
/// let result = parse_response(body).unwrap();
/// assert_eq!(result.code, 200);
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
    /// Status 200 = pass through, 502 = already completed.
    pub fn should_pass(&self) -> bool {
        self.code == 200 || self.code == 502
    }

    /// Status 201 = wait (1s poll loop).
    pub fn should_wait(&self) -> bool {
        self.code == 201
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_pass_response() {
        let body = "NetFunnel.gControl.result='200:ok:key=abc123&ip=1.2.3.4'";
        let result = parse_response(body).unwrap();
        assert_eq!(result.code, 200);
        assert_eq!(result.status, "ok");
        assert_eq!(result.params.get("key").unwrap(), "abc123");
        assert_eq!(result.params.get("ip").unwrap(), "1.2.3.4");
        assert!(result.should_pass());
        assert!(!result.should_wait());
    }

    #[test]
    fn parse_wait_response() {
        let body = "NetFunnel.gControl.result='201:wait:nwait=5&ttl=48'";
        let result = parse_response(body).unwrap();
        assert!(result.should_wait());
        assert!(!result.should_pass());
    }

    #[test]
    fn parse_already_completed() {
        let body = "NetFunnel.gControl.result='502:already:key=xyz'";
        let result = parse_response(body).unwrap();
        assert!(result.should_pass());
    }

    #[test]
    fn parse_invalid() {
        assert!(parse_response("garbage").is_none());
        assert!(parse_response("result='bad'").is_none());
    }
}
