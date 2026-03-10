use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use chrono::{Duration as ChronoDuration, Utc};
use reqwest::blocking::Client as BlockingClient;
use reqwest::header::{ACCEPT, CONTENT_TYPE, HeaderMap, HeaderValue, REFERER, USER_AGENT};
use secrecy::{ExposeSecret, SecretString};
use url::form_urlencoded;

use crate::providers::ProviderOperation;

use super::{
    ClientCallOutput, SrtClient, SrtProviderError, SrtResult,
    netfunnel::NetfunnelStatus,
    payment::{PayWithCardRequest, PayWithCardResponse},
    reservation::{
        CancelRequest, CancelResponse, GetReservationsRequest, GetReservationsResponse,
        RefundRequest, RefundResponse, ReserveInfoRequest, ReserveInfoResponse, ReserveRequest,
        ReserveResponse, ReserveStandbyOptionSettingsRequest, ReserveStandbyOptionSettingsResponse,
        ReserveStandbyRequest, ReserveStandbyResponse, SrtReservation,
    },
    search::{SearchTrainRequest, SearchTrainResponse, SrtTrain},
    session::{SessionCookie, SessionMaterial, SessionSnapshot},
    ticket::{SrtTicket, TicketInfoRequest, TicketInfoResponse},
    types::{
        ClearRequest, ClearResponse, LoginAccountType, LoginRequest, LoginResponse, LogoutRequest,
        LogoutResponse, Passenger, PassengerKind, SeatClassPreference,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SrtClientFailureKind {
    Transient,
    RateLimited,
    Fatal,
    SessionExpired,
    Unauthorized,
}

const SRT_MOBILE_USER_AGENT: &str = "Mozilla/5.0 (Linux; Android 15; SM-S912N Build/AP3A.240905.015.A2; wv) AppleWebKit/537.36(KHTML, like Gecko) Version/4.0 Chrome/136.0.7103.125 Mobile Safari/537.36SRT-APP-Android V.2.0.38";
const NETFUNNEL_WAIT_STATUS_PASS: &str = "200";
const NETFUNNEL_WAIT_STATUS_FAIL: &str = "201";
const NETFUNNEL_ALREADY_COMPLETED: &str = "502";
const NETFUNNEL_OP_START: &str = "5101";
const NETFUNNEL_OP_CHECK: &str = "5002";
const NETFUNNEL_OP_COMPLETE: &str = "5004";
const NETFUNNEL_CACHE_TTL: Duration = Duration::from_secs(48);
const NETFUNNEL_WAIT_POLL_INTERVAL: Duration = Duration::from_secs(1);

impl SrtClientFailureKind {
    fn into_provider_error(self, operation: ProviderOperation) -> SrtProviderError {
        match self {
            Self::Transient => SrtProviderError::Transport {
                message: format!(
                    "deterministic transport failure for {}",
                    operation_name(operation)
                ),
            },
            Self::RateLimited => SrtProviderError::OperationFailed {
                message: format!(
                    "rate_limited deterministic failure for {}",
                    operation_name(operation)
                ),
            },
            Self::Fatal => SrtProviderError::OperationFailed {
                message: format!(
                    "deterministic provider failure for {}",
                    operation_name(operation)
                ),
            },
            Self::SessionExpired => SrtProviderError::SessionExpired,
            Self::Unauthorized => SrtProviderError::Unauthorized,
        }
    }
}

#[derive(Debug, Clone)]
struct PlannedFailure {
    kind: SrtClientFailureKind,
    remaining: usize,
}

#[derive(Debug, Clone, Default)]
struct NetfunnelCache {
    key: Option<String>,
    fetched_at: Option<Instant>,
}

impl NetfunnelCache {
    fn get(&self) -> Option<String> {
        match (&self.key, self.fetched_at) {
            (Some(key), Some(fetched_at)) if fetched_at.elapsed() < NETFUNNEL_CACHE_TTL => {
                Some(key.clone())
            }
            _ => None,
        }
    }

    fn store(&mut self, key: String) {
        self.key = Some(key);
        self.fetched_at = Some(Instant::now());
    }

    fn clear(&mut self) {
        self.key = None;
        self.fetched_at = None;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NetfunnelResponse {
    status: String,
    key: Option<String>,
    waiting_count: Option<u32>,
    ip: Option<String>,
}

#[derive(Debug, Clone)]
enum SrtTransportMode {
    Deterministic,
    Live {
        base_url: String,
        client: BlockingClient,
    },
}

#[derive(Debug, Clone)]
pub struct ReqwestSrtClient {
    mode: SrtTransportMode,
    failures: HashMap<ProviderOperation, PlannedFailure>,
    netfunnel_cache: NetfunnelCache,
}

impl Default for ReqwestSrtClient {
    fn default() -> Self {
        Self::deterministic()
    }
}

impl ReqwestSrtClient {
    pub fn deterministic() -> Self {
        Self {
            mode: SrtTransportMode::Deterministic,
            failures: HashMap::new(),
            netfunnel_cache: NetfunnelCache::default(),
        }
    }

    pub fn live(base_url: impl Into<String>) -> Self {
        Self::live_with_timeout(base_url, Duration::from_secs(15))
    }

    pub fn live_with_timeout(base_url: impl Into<String>, timeout: Duration) -> Self {
        let client = BlockingClient::builder()
            .timeout(timeout)
            .default_headers(srt_default_headers())
            .cookie_store(true)
            .build()
            .unwrap_or_else(|_| {
                BlockingClient::builder()
                    .timeout(timeout)
                    .default_headers(srt_default_headers())
                    .build()
                    .unwrap_or_else(|_| BlockingClient::new())
            });
        Self {
            mode: SrtTransportMode::Live {
                base_url: trim_trailing_slash(base_url.into()),
                client,
            },
            failures: HashMap::new(),
            netfunnel_cache: NetfunnelCache::default(),
        }
    }

    pub fn live_default() -> Self {
        Self::live("https://app.srail.or.kr")
    }

    pub fn with_failure(
        mut self,
        operation: ProviderOperation,
        kind: SrtClientFailureKind,
        times: usize,
    ) -> Self {
        if times > 0 {
            self.failures.insert(
                operation,
                PlannedFailure {
                    kind,
                    remaining: times,
                },
            );
        }
        self
    }

    fn maybe_fail(&mut self, operation: ProviderOperation) -> SrtResult<()> {
        let mut clear = false;
        let failure = self.failures.get_mut(&operation).and_then(|planned| {
            if planned.remaining == 0 {
                return None;
            }

            planned.remaining -= 1;
            if planned.remaining == 0 {
                clear = true;
            }
            Some(planned.kind)
        });

        if clear {
            self.failures.remove(&operation);
        }

        if let Some(kind) = failure {
            return Err(kind.into_provider_error(operation));
        }

        Ok(())
    }

    fn maybe_live_form(
        &self,
        operation: ProviderOperation,
        endpoint: &str,
        form: Vec<(String, String)>,
    ) -> SrtResult<()> {
        let SrtTransportMode::Live { base_url, client } = &self.mode else {
            return Ok(());
        };

        let url = format!("{base_url}{endpoint}");
        let encoded_form = encode_form_pairs(&form);
        let mut request = client
            .post(url)
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(encoded_form);
        if matches!(operation, ProviderOperation::Login) {
            request = request.header(REFERER, format!("{base_url}/main/main.do"));
        }
        let response = request.send().map_err(|err| SrtProviderError::Transport {
            message: format!(
                "srt transport failed for {}: {err}",
                operation_name(operation)
            ),
        })?;
        let status = response.status();

        if status.is_success() {
            return Ok(());
        }
        if status.as_u16() == 429 {
            return Err(SrtProviderError::OperationFailed {
                message: format!("rate_limited for {}", operation_name(operation)),
            });
        }
        if status.as_u16() == 401 || status.as_u16() == 403 {
            return Err(SrtProviderError::Unauthorized);
        }
        if status.is_server_error() {
            return Err(SrtProviderError::Transport {
                message: format!(
                    "srt upstream status {} for {}",
                    status.as_u16(),
                    operation_name(operation)
                ),
            });
        }

        Err(SrtProviderError::OperationFailed {
            message: format!(
                "srt upstream rejected {} with status {}",
                operation_name(operation),
                status.as_u16()
            ),
        })
    }

    fn prime_login_session(&self) -> SrtResult<()> {
        let SrtTransportMode::Live { base_url, client } = &self.mode else {
            return Ok(());
        };

        let response = client
            .get(format!("{base_url}/main/main.do"))
            .send()
            .map_err(|err| SrtProviderError::Transport {
                message: format!("srt transport failed for login bootstrap: {err}"),
            })?;
        let status = response.status();
        if status.is_success() {
            Ok(())
        } else if status.is_server_error() {
            Err(SrtProviderError::Transport {
                message: format!(
                    "srt upstream status {} for login bootstrap",
                    status.as_u16()
                ),
            })
        } else {
            Err(SrtProviderError::OperationFailed {
                message: format!(
                    "srt upstream rejected login bootstrap with status {}",
                    status.as_u16()
                ),
            })
        }
    }

    fn ensure_netfunnel_key(&mut self) -> SrtResult<Option<String>> {
        let SrtTransportMode::Live { .. } = &self.mode else {
            return Ok(None);
        };

        if let Some(key) = self.netfunnel_cache.get() {
            return Ok(Some(key));
        }

        let mut response = self.netfunnel_request(NETFUNNEL_OP_START, None, None)?;
        while response.status == NETFUNNEL_WAIT_STATUS_FAIL {
            thread::sleep(NETFUNNEL_WAIT_POLL_INTERVAL);
            response = self.netfunnel_request(
                NETFUNNEL_OP_CHECK,
                response.ip.as_deref(),
                response.key.as_deref(),
            )?;
        }

        if response.status != NETFUNNEL_WAIT_STATUS_PASS {
            return Err(SrtProviderError::OperationFailed {
                message: format!(
                    "srt netfunnel returned unexpected status {}",
                    response.status
                ),
            });
        }

        let key = response
            .key
            .ok_or_else(|| SrtProviderError::OperationFailed {
                message: "srt netfunnel did not return a key".to_string(),
            })?;
        let complete = self.netfunnel_request(
            NETFUNNEL_OP_COMPLETE,
            response.ip.as_deref(),
            Some(key.as_str()),
        )?;
        if complete.status != NETFUNNEL_WAIT_STATUS_PASS
            && complete.status != NETFUNNEL_ALREADY_COMPLETED
        {
            return Err(SrtProviderError::OperationFailed {
                message: format!(
                    "srt netfunnel completion returned unexpected status {}",
                    complete.status
                ),
            });
        }

        self.netfunnel_cache.store(key.clone());
        Ok(Some(key))
    }

    fn netfunnel_request(
        &self,
        opcode: &str,
        ip: Option<&str>,
        key: Option<&str>,
    ) -> SrtResult<NetfunnelResponse> {
        let SrtTransportMode::Live { client, .. } = &self.mode else {
            return Err(SrtProviderError::OperationFailed {
                message: "srt netfunnel is unavailable in deterministic mode".to_string(),
            });
        };

        let query = encode_form_pairs(&netfunnel_query_params(opcode, key));
        let url = format!(
            "https://{}/ts.wseq?{query}",
            ip.unwrap_or("nf.letskorail.com")
        );
        let response = client
            .get(url)
            .headers(netfunnel_default_headers())
            .send()
            .map_err(|err| SrtProviderError::Transport {
                message: format!("srt transport failed for netfunnel: {err}"),
            })?;
        let status = response.status();
        if !status.is_success() {
            return Err(if status.is_server_error() {
                SrtProviderError::Transport {
                    message: format!("srt upstream status {} for netfunnel", status.as_u16()),
                }
            } else {
                SrtProviderError::OperationFailed {
                    message: format!(
                        "srt upstream rejected netfunnel with status {}",
                        status.as_u16()
                    ),
                }
            });
        }
        let body = response.text().map_err(|err| SrtProviderError::Transport {
            message: format!("srt transport failed reading netfunnel response: {err}"),
        })?;
        parse_netfunnel_response(&body)
    }

    fn canned_session() -> SessionMaterial {
        SessionMaterial {
            cookies: vec![SessionCookie::new(
                "JSESSIONID",
                SecretString::new("deterministic-cookie".into()),
            )],
            expires_at: Some(Utc::now() + ChronoDuration::minutes(30)),
        }
    }

    fn canned_login_response() -> LoginResponse {
        LoginResponse {
            membership_number: "MEM-DET-1".to_string(),
            membership_name: "Deterministic User".to_string(),
            phone_number: Some("01012341234".to_string()),
            session: Self::canned_session(),
        }
    }

    fn canned_train() -> SrtTrain {
        SrtTrain {
            train_code: "17".to_string(),
            train_number: "301".to_string(),
            dep_station_code: "0551".to_string(),
            arr_station_code: "0020".to_string(),
            dep_date: "20260305".to_string(),
            dep_time: "080000".to_string(),
            arr_date: "20260305".to_string(),
            arr_time: "103000".to_string(),
            general_seat_available: true,
            special_seat_available: false,
            standby_available: true,
        }
    }

    fn canned_reservation() -> SrtReservation {
        SrtReservation {
            reservation_id: "PNR-DET-1".to_string(),
            train_number: "301".to_string(),
            dep_station_code: "0551".to_string(),
            arr_station_code: "0020".to_string(),
            dep_date: "20260305".to_string(),
            dep_time: "080000".to_string(),
            arr_time: "103000".to_string(),
            seat_count: 1,
            total_cost: 55_000,
            paid: false,
            waiting: false,
        }
    }

    fn canned_ticket() -> SrtTicket {
        SrtTicket {
            reservation_id: "PNR-DET-1".to_string(),
            car: Some("4".to_string()),
            seat: Some("8A".to_string()),
            seat_class: "general".to_string(),
            passenger_type: "adult".to_string(),
            price: 55_000,
            discount: 0,
            waiting: false,
        }
    }
}

impl SrtClient for ReqwestSrtClient {
    fn login(&mut self, request: &LoginRequest) -> SrtResult<ClientCallOutput<LoginResponse>> {
        self.maybe_fail(ProviderOperation::Login)?;
        self.netfunnel_cache.clear();
        self.prime_login_session()?;
        let base_url = match &self.mode {
            SrtTransportMode::Live { base_url, .. } => base_url.as_str(),
            SrtTransportMode::Deterministic => "https://app.srail.or.kr",
        };
        self.maybe_live_form(
            ProviderOperation::Login,
            "/apb/selectListApb01080_n.do",
            login_form_fields(request, base_url),
        )?;
        Ok(ClientCallOutput::success(Self::canned_login_response()))
    }

    fn logout(
        &mut self,
        _session: &SessionSnapshot,
        _request: &LogoutRequest,
    ) -> SrtResult<ClientCallOutput<LogoutResponse>> {
        self.maybe_fail(ProviderOperation::Logout)?;
        self.netfunnel_cache.clear();
        self.maybe_live_form(ProviderOperation::Logout, "/common/logout.do", Vec::new())?;
        Ok(ClientCallOutput::success(LogoutResponse {
            logged_out: true,
        }))
    }

    fn search_train(
        &mut self,
        _session: &SessionSnapshot,
        request: &SearchTrainRequest,
    ) -> SrtResult<ClientCallOutput<SearchTrainResponse>> {
        self.maybe_fail(ProviderOperation::SearchTrain)?;
        let netfunnel_key = self.ensure_netfunnel_key()?.unwrap_or_default();
        self.maybe_live_form(
            ProviderOperation::SearchTrain,
            "/ara/selectListAra10007_n.do",
            search_form_fields(request, netfunnel_key.as_str()),
        )?;
        Ok(ClientCallOutput::success(SearchTrainResponse {
            trains: vec![Self::canned_train()],
            netfunnel_status: NetfunnelStatus::Pass,
        }))
    }

    fn reserve(
        &mut self,
        _session: &SessionSnapshot,
        request: &ReserveRequest,
    ) -> SrtResult<ClientCallOutput<ReserveResponse>> {
        self.maybe_fail(ProviderOperation::Reserve)?;
        let netfunnel_key = self.ensure_netfunnel_key()?.unwrap_or_default();
        self.maybe_live_form(
            ProviderOperation::Reserve,
            "/arc/selectListArc05013_n.do",
            reserve_form_fields(
                reserve_form_context(request, "1101", None),
                netfunnel_key.as_str(),
            ),
        )?;
        Ok(ClientCallOutput::success(ReserveResponse {
            reservation: Self::canned_reservation(),
        }))
    }

    fn reserve_standby(
        &mut self,
        _session: &SessionSnapshot,
        request: &ReserveStandbyRequest,
    ) -> SrtResult<ClientCallOutput<ReserveStandbyResponse>> {
        self.maybe_fail(ProviderOperation::ReserveStandby)?;
        let netfunnel_key = self.ensure_netfunnel_key()?.unwrap_or_default();
        self.maybe_live_form(
            ProviderOperation::ReserveStandby,
            "/arc/selectListArc05013_n.do",
            reserve_form_fields(
                reserve_standby_form_context(request, "1102"),
                netfunnel_key.as_str(),
            ),
        )?;
        Ok(ClientCallOutput::success(ReserveStandbyResponse {
            reservation: Self::canned_reservation(),
        }))
    }

    fn reserve_standby_option_settings(
        &mut self,
        _session: &SessionSnapshot,
        request: &ReserveStandbyOptionSettingsRequest,
    ) -> SrtResult<ClientCallOutput<ReserveStandbyOptionSettingsResponse>> {
        self.maybe_fail(ProviderOperation::ReserveStandbyOptionSettings)?;
        self.maybe_live_form(
            ProviderOperation::ReserveStandbyOptionSettings,
            "/ata/selectListAta01135_n.do",
            vec![
                ("pnrNo".to_string(), request.reservation_id.clone()),
                (
                    "smsRecvYn".to_string(),
                    if request.agree_sms { "Y" } else { "N" }.to_string(),
                ),
            ],
        )?;
        Ok(ClientCallOutput::success(
            ReserveStandbyOptionSettingsResponse { updated: true },
        ))
    }

    fn get_reservations(
        &mut self,
        _session: &SessionSnapshot,
        _request: &GetReservationsRequest,
    ) -> SrtResult<ClientCallOutput<GetReservationsResponse>> {
        self.maybe_fail(ProviderOperation::GetReservations)?;
        self.maybe_live_form(
            ProviderOperation::GetReservations,
            "/atc/selectListAtc14016_n.do",
            Vec::new(),
        )?;
        Ok(ClientCallOutput::success(GetReservationsResponse {
            reservations: vec![Self::canned_reservation()],
        }))
    }

    fn ticket_info(
        &mut self,
        _session: &SessionSnapshot,
        request: &TicketInfoRequest,
    ) -> SrtResult<ClientCallOutput<TicketInfoResponse>> {
        self.maybe_fail(ProviderOperation::TicketInfo)?;
        self.maybe_live_form(
            ProviderOperation::TicketInfo,
            "/atc/getListAtc14087.do",
            vec![("pnrNo".to_string(), request.reservation_id.clone())],
        )?;
        Ok(ClientCallOutput::success(TicketInfoResponse {
            tickets: vec![Self::canned_ticket()],
        }))
    }

    fn cancel(
        &mut self,
        _session: &SessionSnapshot,
        request: &CancelRequest,
    ) -> SrtResult<ClientCallOutput<CancelResponse>> {
        self.maybe_fail(ProviderOperation::Cancel)?;
        self.maybe_live_form(
            ProviderOperation::Cancel,
            "/ard/selectListArd02045_n.do",
            vec![("pnrNo".to_string(), request.reservation_id.clone())],
        )?;
        Ok(ClientCallOutput::success(CancelResponse { canceled: true }))
    }

    fn pay_with_card(
        &mut self,
        _session: &SessionSnapshot,
        request: &PayWithCardRequest,
    ) -> SrtResult<ClientCallOutput<PayWithCardResponse>> {
        self.maybe_fail(ProviderOperation::PayWithCard)?;
        self.maybe_live_form(
            ProviderOperation::PayWithCard,
            "/ata/selectListAta09036_n.do",
            vec![
                ("pnrNo".to_string(), request.reservation_id.clone()),
                (
                    "stlCrCrdNo".to_string(),
                    request.card_number.expose_secret().to_string(),
                ),
                (
                    "vanPwd".to_string(),
                    request.card_password_two_digits.expose_secret().to_string(),
                ),
                (
                    "athnVal".to_string(),
                    request.card_validation_number.expose_secret().to_string(),
                ),
                (
                    "crdVlidTrm".to_string(),
                    request.card_expiry_yymm.expose_secret().to_string(),
                ),
            ],
        )?;
        Ok(ClientCallOutput::success(PayWithCardResponse {
            paid: true,
            approval_code: Some("APR-DET-1".to_string()),
        }))
    }

    fn reserve_info(
        &mut self,
        _session: &SessionSnapshot,
        request: &ReserveInfoRequest,
    ) -> SrtResult<ClientCallOutput<ReserveInfoResponse>> {
        self.maybe_fail(ProviderOperation::ReserveInfo)?;
        self.maybe_live_form(
            ProviderOperation::ReserveInfo,
            "/ard/selectListArd02019_n.do",
            vec![("pnrNo".to_string(), request.reservation_id.clone())],
        )?;
        Ok(ClientCallOutput::success(ReserveInfoResponse {
            reservation: Some(Self::canned_reservation()),
            refundable: true,
        }))
    }

    fn refund(
        &mut self,
        _session: &SessionSnapshot,
        request: &RefundRequest,
    ) -> SrtResult<ClientCallOutput<RefundResponse>> {
        self.maybe_fail(ProviderOperation::Refund)?;
        self.maybe_live_form(
            ProviderOperation::Refund,
            "/atc/selectListAtc02063_n.do",
            vec![("pnrNo".to_string(), request.reservation_id.clone())],
        )?;
        Ok(ClientCallOutput::success(RefundResponse { refunded: true }))
    }

    fn clear(&mut self, _request: &ClearRequest) -> SrtResult<ClientCallOutput<ClearResponse>> {
        self.maybe_fail(ProviderOperation::Clear)?;
        self.netfunnel_cache.clear();
        Ok(ClientCallOutput::success(ClearResponse { cleared: true }))
    }
}

fn operation_name(operation: ProviderOperation) -> &'static str {
    match operation {
        ProviderOperation::Login => "login",
        ProviderOperation::Logout => "logout",
        ProviderOperation::SearchTrain => "search_train",
        ProviderOperation::Reserve => "reserve",
        ProviderOperation::ReserveStandby => "reserve_standby",
        ProviderOperation::ReserveStandbyOptionSettings => "reserve_standby_option_settings",
        ProviderOperation::GetReservations => "get_reservations",
        ProviderOperation::TicketInfo => "ticket_info",
        ProviderOperation::Cancel => "cancel",
        ProviderOperation::PayWithCard => "pay_with_card",
        ProviderOperation::ReserveInfo => "reserve_info",
        ProviderOperation::Refund => "refund",
        ProviderOperation::Clear => "clear",
    }
}

fn trim_trailing_slash(raw: String) -> String {
    raw.trim_end_matches('/').to_string()
}

fn encode_form_pairs(form: &[(String, String)]) -> String {
    let mut serializer = form_urlencoded::Serializer::new(String::new());
    for (key, value) in form {
        serializer.append_pair(key.as_str(), value.as_str());
    }
    serializer.finish()
}

fn srt_default_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static(SRT_MOBILE_USER_AGENT));
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    headers
}

fn netfunnel_default_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static(SRT_MOBILE_USER_AGENT));
    headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
    headers.insert(
        "sec-ch-ua",
        HeaderValue::from_static(
            "\"Chromium\";v=\"136\", \"Android WebView\";v=\"136\", \"Not=A/Brand\";v=\"99\"",
        ),
    );
    headers.insert("sec-ch-ua-mobile", HeaderValue::from_static("?1"));
    headers.insert("sec-ch-ua-platform", HeaderValue::from_static("Android"));
    headers.insert(
        "x-requested-with",
        HeaderValue::from_static("kr.co.srail.newapp"),
    );
    headers.insert("sec-fetch-site", HeaderValue::from_static("cross-site"));
    headers.insert("sec-fetch-mode", HeaderValue::from_static("no-cors"));
    headers.insert("sec-fetch-dest", HeaderValue::from_static("script"));
    headers.insert(
        "sec-fetch-storage-access",
        HeaderValue::from_static("active"),
    );
    headers.insert(
        REFERER,
        HeaderValue::from_static("https://app.srail.or.kr/"),
    );
    headers.insert(
        "accept-language",
        HeaderValue::from_static("en-US,en;q=0.9,ko-KR;q=0.8,ko;q=0.7"),
    );
    headers
}

fn login_form_fields(request: &LoginRequest, base_url: &str) -> Vec<(String, String)> {
    let (login_type, account_identifier) = login_identity_fields(request);
    vec![
        ("auto".to_string(), "Y".to_string()),
        ("check".to_string(), "Y".to_string()),
        ("page".to_string(), "menu".to_string()),
        ("deviceKey".to_string(), "-".to_string()),
        ("customerYn".to_string(), String::new()),
        (
            "login_referer".to_string(),
            format!("{}/main/main.do", trim_trailing_slash(base_url.to_string())),
        ),
        ("srchDvCd".to_string(), login_type.to_string()),
        ("srchDvNm".to_string(), account_identifier),
        (
            "hmpgPwdCphd".to_string(),
            request.password.expose_secret().to_string(),
        ),
    ]
}

fn login_identity_fields(request: &LoginRequest) -> (&'static str, String) {
    let account_identifier = request.account_identifier.trim();
    match request.account_type {
        LoginAccountType::MembershipNumber => ("1", account_identifier.to_string()),
        LoginAccountType::Email => ("2", account_identifier.to_string()),
        LoginAccountType::PhoneNumber => ("3", account_identifier.replace('-', "")),
    }
}

fn search_form_fields(request: &SearchTrainRequest, netfunnel_key: &str) -> Vec<(String, String)> {
    vec![
        ("chtnDvCd".to_string(), "1".to_string()),
        ("dptDt".to_string(), request.dep_date.clone()),
        ("dptTm".to_string(), request.dep_time.clone()),
        ("dptDt1".to_string(), request.dep_date.clone()),
        (
            "dptTm1".to_string(),
            format!("{}0000", request.dep_time.get(..2).unwrap_or("00")),
        ),
        ("dptRsStnCd".to_string(), request.dep_station_code.clone()),
        ("arvRsStnCd".to_string(), request.arr_station_code.clone()),
        ("stlbTrnClsfCd".to_string(), "05".to_string()),
        ("trnGpCd".to_string(), "109".to_string()),
        ("trnNo".to_string(), String::new()),
        (
            "psgNum".to_string(),
            total_passenger_count(request.passengers.as_slice()).to_string(),
        ),
        ("seatAttCd".to_string(), "015".to_string()),
        ("arriveTime".to_string(), "N".to_string()),
        ("tkDptDt".to_string(), String::new()),
        ("tkDptTm".to_string(), String::new()),
        ("tkTrnNo".to_string(), String::new()),
        ("tkTripChgFlg".to_string(), String::new()),
        ("dlayTnumAplFlg".to_string(), "Y".to_string()),
        ("netfunnelKey".to_string(), netfunnel_key.to_string()),
    ]
}

struct ReserveFormContext<'a> {
    train: &'a SrtTrain,
    passengers: &'a [Passenger],
    seat_preference: SeatClassPreference,
    window_seat: Option<bool>,
    notification_phone: Option<&'a str>,
    job_id: &'static str,
}

fn reserve_form_context<'a>(
    request: &'a ReserveRequest,
    job_id: &'static str,
    notification_phone: Option<&'a str>,
) -> ReserveFormContext<'a> {
    ReserveFormContext {
        train: &request.train,
        passengers: request.passengers.as_slice(),
        seat_preference: request.seat_preference,
        window_seat: request.window_seat,
        notification_phone,
        job_id,
    }
}

fn reserve_standby_form_context<'a>(
    request: &'a ReserveStandbyRequest,
    job_id: &'static str,
) -> ReserveFormContext<'a> {
    ReserveFormContext {
        train: &request.train,
        passengers: request.passengers.as_slice(),
        seat_preference: request.seat_preference,
        window_seat: None,
        notification_phone: request.notification_phone.as_deref(),
        job_id,
    }
}

fn reserve_form_fields(
    context: ReserveFormContext<'_>,
    netfunnel_key: &str,
) -> Vec<(String, String)> {
    let mut form = vec![
        ("jobId".to_string(), context.job_id.to_string()),
        ("jrnyCnt".to_string(), "1".to_string()),
        ("jrnyTpCd".to_string(), "11".to_string()),
        ("jrnySqno1".to_string(), "001".to_string()),
        ("stndFlg".to_string(), "N".to_string()),
        ("trnGpCd1".to_string(), "300".to_string()),
        ("trnGpCd".to_string(), "109".to_string()),
        ("grpDv".to_string(), "0".to_string()),
        ("rtnDv".to_string(), "0".to_string()),
        (
            "stlbTrnClsfCd1".to_string(),
            context.train.train_code.clone(),
        ),
        (
            "dptRsStnCd1".to_string(),
            context.train.dep_station_code.clone(),
        ),
        (
            "arvRsStnCd1".to_string(),
            context.train.arr_station_code.clone(),
        ),
        ("dptDt1".to_string(), context.train.dep_date.clone()),
        ("dptTm1".to_string(), context.train.dep_time.clone()),
        ("arvTm1".to_string(), context.train.arr_time.clone()),
        (
            "trnNo1".to_string(),
            padded_train_number(context.train.train_number.as_str()),
        ),
        ("runDt1".to_string(), context.train.dep_date.clone()),
        (
            "mblPhone".to_string(),
            context.notification_phone.unwrap_or_default().to_string(),
        ),
        ("netfunnelKey".to_string(), netfunnel_key.to_string()),
    ];
    if context.job_id == "1101" {
        form.push(("reserveType".to_string(), "11".to_string()));
    }
    form.extend(passenger_form_fields(
        context.passengers,
        uses_special_seat(context.seat_preference, context.train),
        context.window_seat,
    ));
    form
}

fn passenger_form_fields(
    passengers: &[Passenger],
    special_seat: bool,
    window_seat: Option<bool>,
) -> Vec<(String, String)> {
    let grouped = grouped_passengers(passengers);
    let mut fields = vec![
        (
            "totPrnb".to_string(),
            total_passenger_count(passengers).to_string(),
        ),
        ("psgGridcnt".to_string(), grouped.len().to_string()),
        (
            "locSeatAttCd1".to_string(),
            match window_seat {
                Some(true) => "012",
                Some(false) => "013",
                None => "000",
            }
            .to_string(),
        ),
        ("rqSeatAttCd1".to_string(), "015".to_string()),
        ("dirSeatAttCd1".to_string(), "009".to_string()),
        ("smkSeatAttCd1".to_string(), "000".to_string()),
        ("etcSeatAttCd1".to_string(), "000".to_string()),
        (
            "psrmClCd1".to_string(),
            if special_seat { "2" } else { "1" }.to_string(),
        ),
    ];

    for (index, (kind, count)) in grouped.iter().enumerate() {
        let ordinal = index + 1;
        fields.push((
            format!("psgTpCd{ordinal}"),
            passenger_type_code(*kind).to_string(),
        ));
        fields.push((format!("psgInfoPerPrnb{ordinal}"), count.to_string()));
    }

    fields
}

fn grouped_passengers(passengers: &[Passenger]) -> Vec<(PassengerKind, u8)> {
    let order = [
        PassengerKind::Adult,
        PassengerKind::Disability1To3,
        PassengerKind::Disability4To6,
        PassengerKind::Senior,
        PassengerKind::Child,
    ];
    order
        .iter()
        .filter_map(|kind| {
            let count = passengers
                .iter()
                .filter(|passenger| passenger.kind == *kind)
                .map(|passenger| passenger.count)
                .sum::<u8>();
            (count > 0).then_some((*kind, count))
        })
        .collect()
}

fn passenger_type_code(kind: PassengerKind) -> &'static str {
    match kind {
        PassengerKind::Adult => "1",
        PassengerKind::Disability1To3 => "2",
        PassengerKind::Disability4To6 => "3",
        PassengerKind::Senior => "4",
        PassengerKind::Child => "5",
    }
}

fn total_passenger_count(passengers: &[Passenger]) -> u32 {
    passengers
        .iter()
        .map(|passenger| u32::from(passenger.count))
        .sum()
}

fn uses_special_seat(preference: SeatClassPreference, train: &SrtTrain) -> bool {
    match preference {
        SeatClassPreference::GeneralFirst => !train.general_seat_available,
        SeatClassPreference::GeneralOnly => false,
        SeatClassPreference::SpecialFirst => train.special_seat_available,
        SeatClassPreference::SpecialOnly => true,
    }
}

fn padded_train_number(raw: &str) -> String {
    raw.trim()
        .parse::<u32>()
        .map(|value| format!("{value:05}"))
        .unwrap_or_else(|_| raw.trim().to_string())
}

fn netfunnel_query_params(opcode: &str, key: Option<&str>) -> Vec<(String, String)> {
    let mut params = vec![
        ("opcode".to_string(), opcode.to_string()),
        ("nfid".to_string(), "0".to_string()),
        ("prefix".to_string(), format!("NetFunnel.gRtype={opcode};")),
        ("js".to_string(), "true".to_string()),
        (timestamp_millis().to_string(), String::new()),
    ];
    if opcode == NETFUNNEL_OP_START || opcode == NETFUNNEL_OP_CHECK {
        params.push(("sid".to_string(), "service_1".to_string()));
        params.push(("aid".to_string(), "act_10".to_string()));
        if opcode == NETFUNNEL_OP_CHECK {
            params.push(("key".to_string(), key.unwrap_or_default().to_string()));
            params.push(("ttl".to_string(), "1".to_string()));
        }
    } else if opcode == NETFUNNEL_OP_COMPLETE {
        params.push(("key".to_string(), key.unwrap_or_default().to_string()));
    }
    params
}

fn timestamp_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

fn parse_netfunnel_response(raw: &str) -> SrtResult<NetfunnelResponse> {
    let marker = "NetFunnel.gControl.result='";
    let start = raw
        .find(marker)
        .ok_or_else(|| SrtProviderError::OperationFailed {
            message: "failed to parse netfunnel response".to_string(),
        })?;
    let payload = &raw[start + marker.len()..];
    let end = payload
        .find('\'')
        .ok_or_else(|| SrtProviderError::OperationFailed {
            message: "failed to parse netfunnel response".to_string(),
        })?;
    let result = &payload[..end];
    let mut parts = result.splitn(3, ':');
    let _code = parts.next().unwrap_or_default();
    let status = parts.next().unwrap_or_default();
    let params = parts.next().unwrap_or_default();
    if status.is_empty() || params.is_empty() {
        return Err(SrtProviderError::OperationFailed {
            message: "failed to parse netfunnel response".to_string(),
        });
    }

    let mut key = None;
    let mut waiting_count = None;
    let mut ip = None;
    for param in params.split('&') {
        let Some((name, value)) = param.split_once('=') else {
            continue;
        };
        match name {
            "key" if !value.is_empty() => key = Some(value.to_string()),
            "nwait" => waiting_count = value.parse::<u32>().ok(),
            "ip" if !value.is_empty() => ip = Some(value.to_string()),
            _ => {}
        }
    }

    Ok(NetfunnelResponse {
        status: status.to_string(),
        key,
        waiting_count,
        ip,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::{ACCEPT, USER_AGENT};

    fn read_field<'a>(form: &'a [(String, String)], key: &str) -> Option<&'a str> {
        form.iter()
            .find(|(candidate, _)| candidate == key)
            .map(|(_, value)| value.as_str())
    }

    #[test]
    fn login_form_matches_srtgo_membership_contract() {
        let request = LoginRequest {
            account_type: LoginAccountType::MembershipNumber,
            account_identifier: "1234567890".to_string(),
            password: SecretString::new("pw".to_string().into_boxed_str()),
        };

        let form = login_form_fields(&request, "https://app.srail.or.kr");

        assert_eq!(read_field(form.as_slice(), "auto"), Some("Y"));
        assert_eq!(read_field(form.as_slice(), "check"), Some("Y"));
        assert_eq!(read_field(form.as_slice(), "page"), Some("menu"));
        assert_eq!(read_field(form.as_slice(), "deviceKey"), Some("-"));
        assert_eq!(read_field(form.as_slice(), "customerYn"), Some(""));
        assert_eq!(
            read_field(form.as_slice(), "login_referer"),
            Some("https://app.srail.or.kr/main/main.do")
        );
        assert_eq!(read_field(form.as_slice(), "srchDvCd"), Some("1"));
        assert_eq!(read_field(form.as_slice(), "srchDvNm"), Some("1234567890"));
        assert_eq!(read_field(form.as_slice(), "hmpgPwdCphd"), Some("pw"));
    }

    #[test]
    fn login_form_uses_email_login_type() {
        let request = LoginRequest {
            account_type: LoginAccountType::Email,
            account_identifier: "user@example.com".to_string(),
            password: SecretString::new("pw".to_string().into_boxed_str()),
        };

        let form = login_form_fields(&request, "https://app.srail.or.kr");

        assert_eq!(read_field(form.as_slice(), "srchDvCd"), Some("2"));
        assert_eq!(
            read_field(form.as_slice(), "srchDvNm"),
            Some("user@example.com")
        );
    }

    #[test]
    fn login_form_uses_phone_login_type_and_normalizes_hyphen() {
        let request = LoginRequest {
            account_type: LoginAccountType::PhoneNumber,
            account_identifier: "010-1234-5678".to_string(),
            password: SecretString::new("pw".to_string().into_boxed_str()),
        };

        let form = login_form_fields(&request, "https://app.srail.or.kr");

        assert_eq!(read_field(form.as_slice(), "srchDvCd"), Some("3"));
        assert_eq!(read_field(form.as_slice(), "srchDvNm"), Some("01012345678"));
    }

    #[test]
    fn default_headers_match_srtgo_profile() {
        let headers = srt_default_headers();
        assert_eq!(
            headers
                .get(USER_AGENT)
                .and_then(|value| value.to_str().ok()),
            Some(SRT_MOBILE_USER_AGENT)
        );
        assert_eq!(
            headers.get(ACCEPT).and_then(|value| value.to_str().ok()),
            Some("application/json")
        );
    }

    #[test]
    fn parse_netfunnel_response_extracts_status_key_and_ip() {
        let response = parse_netfunnel_response(
            "NetFunnel.gRtype=4999;NetFunnel.gControl.result='5002:200:key=abc123&nwait=7&ip=1.2.3.4';",
        )
        .expect("netfunnel response should parse");

        assert_eq!(response.status, "200");
        assert_eq!(response.key.as_deref(), Some("abc123"));
        assert_eq!(response.waiting_count, Some(7));
        assert_eq!(response.ip.as_deref(), Some("1.2.3.4"));
    }

    #[test]
    fn search_form_includes_netfunnel_key_and_srtgo_fields() {
        let form = search_form_fields(
            &SearchTrainRequest {
                dep_station_code: "0551".to_string(),
                arr_station_code: "0020".to_string(),
                dep_date: "20260305".to_string(),
                dep_time: "080000".to_string(),
                time_limit: None,
                passengers: vec![Passenger::adult(2)],
                available_only: true,
            },
            "nf-key",
        );

        assert_eq!(read_field(form.as_slice(), "netfunnelKey"), Some("nf-key"));
        assert_eq!(read_field(form.as_slice(), "dptRsStnCd"), Some("0551"));
        assert_eq!(read_field(form.as_slice(), "arvRsStnCd"), Some("0020"));
        assert_eq!(read_field(form.as_slice(), "psgNum"), Some("2"));
        assert_eq!(read_field(form.as_slice(), "stlbTrnClsfCd"), Some("05"));
        assert_eq!(read_field(form.as_slice(), "trnGpCd"), Some("109"));
    }

    #[test]
    fn reserve_form_includes_netfunnel_key_and_passenger_fields() {
        let form = reserve_form_fields(
            reserve_form_context(
                &ReserveRequest {
                    train: SrtTrain {
                        train_code: "17".to_string(),
                        train_number: "301".to_string(),
                        dep_station_code: "0551".to_string(),
                        arr_station_code: "0020".to_string(),
                        dep_date: "20260305".to_string(),
                        dep_time: "080000".to_string(),
                        arr_date: "20260305".to_string(),
                        arr_time: "103000".to_string(),
                        general_seat_available: true,
                        special_seat_available: false,
                        standby_available: true,
                    },
                    passengers: vec![Passenger::adult(1)],
                    seat_preference: SeatClassPreference::GeneralOnly,
                    window_seat: Some(true),
                },
                "1101",
                None,
            ),
            "nf-key",
        );

        assert_eq!(read_field(form.as_slice(), "netfunnelKey"), Some("nf-key"));
        assert_eq!(read_field(form.as_slice(), "jobId"), Some("1101"));
        assert_eq!(read_field(form.as_slice(), "reserveType"), Some("11"));
        assert_eq!(read_field(form.as_slice(), "psgTpCd1"), Some("1"));
        assert_eq!(read_field(form.as_slice(), "psgInfoPerPrnb1"), Some("1"));
        assert_eq!(read_field(form.as_slice(), "locSeatAttCd1"), Some("012"));
    }
}
