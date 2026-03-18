//! KTX/Korail HTTP client implementation.
//!
//! Faithfully ported from `third_party/srtgo/srtgo/ktx.py`.
//! - Most requests are GET with query params (search, reserve, tickets)
//! - Cancel, pay, refund are POST with form body
//! - DynaPath token required for protected endpoints

use rand::Rng;
use tracing::{debug, instrument};

use crate::types::{AuthType, ProviderError, SeatPreference, classify_auth};

use crate::srt::passenger::{PassengerGroup, PassengerType, combine_passengers, total_count};

use super::crypto::{encrypt_password, generate_sid};
use super::dynapath::{DynaPathEngine, requires_token};
use super::reservation::{KtxReservation, KtxTicket};
use super::response::KtxResponse;
use super::stations::station_code;
use super::train::KtxTrain;

// ── Constants ────────────────────────────────────────────────────────

const BASE_URL: &str = "https://smart.letskorail.com:443/classes/com.korail.mobile";

const USER_AGENT: &str = "Dalvik/2.1.0 (Linux; U; Android 14; SM-S928N Build/UP1A.231005.007)";

const DEVICE: &str = "AD";
const VERSION: &str = "250601002";
/// Public/shared API key sent as a form parameter in every request.
/// This is a protocol constant (not a secret) — same value in the official Korail app.
const KEY: &str = "korail1234567890";

/// Device ID for DynaPath (fixed per client instance in Python reference).
const DEVICE_ID: &str = "558a4f02041657ea";

/// API endpoint paths (appended to BASE_URL).
struct Endpoints;

impl Endpoints {
    const CODE: &str = ".common.code.do";
    const LOGIN: &str = ".login.Login";
    const LOGOUT: &str = ".common.logout";
    const SEARCH: &str = ".seatMovie.ScheduleView";
    const RESERVE: &str = ".certification.TicketReservation";
    const CANCEL: &str = ".reservationCancel.ReservationCancelChk";
    const _TICKETS: &str = ".myTicket.MyTicketList";
    const _TICKET_INFO: &str = ".refunds.SelTicketInfo";
    const RESERVATION_VIEW: &str = ".reservation.ReservationView";
    const RESERVATION_LIST: &str = ".certification.ReservationList";
    const PAYMENT: &str = ".payment.ReservationPayment";
    const REFUND: &str = ".refunds.RefundsRequest";

    fn url(path: &str) -> String {
        format!("{BASE_URL}{path}")
    }
}

// ── Client ───────────────────────────────────────────────────────────

/// KTX/Korail provider client with persistent session.
pub struct KtxClient {
    client: reqwest::Client,
    dynapath: DynaPathEngine,
    user_info: Option<KtxUserInfo>,
    is_logged_in: bool,
    /// Set once at client creation, NOT regenerated per call.
    _app_start_ts: u64,
}

/// User info extracted after successful login.
#[derive(Debug, Clone)]
pub struct KtxUserInfo {
    pub membership_number: String,
    pub name: String,
    pub email: String,
    pub phone: String,
}

impl KtxClient {
    /// Create a new KTX client with fresh session.
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .cookie_store(true)
            .user_agent(USER_AGENT)
            .default_headers({
                let mut h = reqwest::header::HeaderMap::new();
                h.insert(
                    reqwest::header::CONTENT_TYPE,
                    "application/x-www-form-urlencoded; charset=UTF-8"
                        .parse()
                        .unwrap(),
                );
                h.insert(
                    reqwest::header::HOST,
                    "smart.letskorail.com".parse().unwrap(),
                );
                h
            })
            .build()
            .expect("Failed to build KTX client");

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as u64;

        Self {
            client,
            dynapath: DynaPathEngine::new(now),
            user_info: None,
            is_logged_in: false,
            _app_start_ts: now,
        }
    }

    /// Create a new KTX client that proxies through an Evervault Relay.
    ///
    /// All requests go through the relay, which transparently decrypts
    /// `ev:`-prefixed card fields in-flight. Cookies, Host, and UA are
    /// preserved because the target URL stays `smart.letskorail.com`.
    pub fn with_relay(relay_domain: &str) -> Self {
        let proxy =
            reqwest::Proxy::all(format!("https://{relay_domain}")).expect("Invalid relay domain");

        let client = reqwest::Client::builder()
            .cookie_store(true)
            .user_agent(USER_AGENT)
            .proxy(proxy)
            .default_headers({
                let mut h = reqwest::header::HeaderMap::new();
                h.insert(
                    reqwest::header::CONTENT_TYPE,
                    "application/x-www-form-urlencoded; charset=UTF-8"
                        .parse()
                        .unwrap(),
                );
                h.insert(
                    reqwest::header::HOST,
                    "smart.letskorail.com".parse().unwrap(),
                );
                h
            })
            .build()
            .expect("Failed to build KTX client with relay");

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as u64;

        Self {
            client,
            dynapath: DynaPathEngine::new(now),
            user_info: None,
            is_logged_in: false,
            _app_start_ts: now,
        }
    }

    /// Whether the client is currently logged in.
    pub fn is_logged_in(&self) -> bool {
        self.is_logged_in
    }

    /// User info (available after login).
    pub fn user_info(&self) -> Option<&KtxUserInfo> {
        self.user_info.as_ref()
    }

    /// KTX auth code: Email -> "5", Phone -> "4", Membership -> "2".
    fn auth_code(login_id: &str) -> &'static str {
        match classify_auth(login_id) {
            AuthType::Email => "5",
            AuthType::Phone => "4",
            AuthType::Membership => "2",
        }
    }

    /// Common form params included in every KTX request.
    fn base_params() -> Vec<(&'static str, &'static str)> {
        vec![("Device", DEVICE), ("Version", VERSION), ("Key", KEY)]
    }

    /// Generate current timestamp in milliseconds.
    fn now_ms() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as u64
    }

    /// Generate SID for the current timestamp.
    fn current_sid() -> String {
        generate_sid(DEVICE, Self::now_ms())
    }

    /// Build a request with optional DynaPath token header.
    /// Returns `(request_builder, sid)` — SID shares timestamp with token.
    fn request_with_dynapath(
        &self,
        method: reqwest::Method,
        url: &str,
    ) -> (reqwest::RequestBuilder, String) {
        let mut builder = self.client.request(method, url);
        if requires_token(url) {
            let ts = Self::now_ms();
            let mut rng = rand::rng();
            let chars: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
            let rand_str: String = (0..4)
                .map(|_| chars[rng.random_range(0..chars.len())] as char)
                .collect();
            let token = self.dynapath.generate_token(DEVICE_ID, ts, &rand_str);
            let sid = generate_sid(DEVICE, ts);
            builder = builder.header("x-dynapath-m-token", token);
            (builder, sid)
        } else {
            (builder, String::new())
        }
    }

    // ── Encryption Key ──────────────────────────────────────────────

    /// Fetch the encryption key and idx from the `/code` endpoint.
    async fn fetch_encryption_params(&self) -> Result<(String, String), ProviderError> {
        let form: Vec<(&str, &str)> = vec![
            ("Device", DEVICE),
            ("Version", VERSION),
            ("code", "app.login.cphd"),
        ];

        let resp = self
            .client
            .post(Endpoints::url(Endpoints::CODE))
            .form(&form)
            .send()
            .await?;

        let body = resp.text().await?;
        let parsed = KtxResponse::parse(&body)?;

        let key = parsed.str_field("key");
        let idx = parsed.str_field("idx");

        if key.is_empty() {
            return Err(ProviderError::UnexpectedResponse {
                status: 200,
                body: "Missing encryption key from /code endpoint".to_string(),
            });
        }

        Ok((key.to_string(), idx.to_string()))
    }

    // ── Login ────────────────────────────────────────────────────────

    /// Log in to the KTX/Korail server.
    #[instrument(skip(self, password), fields(login_type))]
    pub async fn login(&mut self, login_id: &str, password: &str) -> Result<(), ProviderError> {
        // Step 1: Fetch encryption key
        let (enc_key, idx) = self.fetch_encryption_params().await?;

        // Step 2: Encrypt password
        let encrypted_password = encrypt_password(password, &enc_key).map_err(|e| {
            ProviderError::UnexpectedResponse {
                status: 500,
                body: format!("Password encryption failed: {e}"),
            }
        })?;

        let login_type = Self::auth_code(login_id);
        let sid = Self::current_sid();

        let form: Vec<(&str, &str)> = vec![
            ("Device", DEVICE),
            ("Version", VERSION),
            ("Key", KEY),
            ("txtMemberNo", login_id),
            ("txtPwd", &encrypted_password),
            ("txtInputFlg", login_type),
            ("idx", &idx),
            ("Sid", &sid),
        ];

        let (builder, _) =
            self.request_with_dynapath(reqwest::Method::POST, &Endpoints::url(Endpoints::LOGIN));
        let resp = builder.form(&form).send().await?;

        let body = resp.text().await?;
        debug!(body_len = body.len(), "KTX login response received");

        let parsed = KtxResponse::parse(&body)?;

        if !parsed.is_success() {
            self.is_logged_in = false;
            return Err(ProviderError::LoginFailed {
                message: parsed.message().to_string(),
            });
        }

        self.user_info = Some(KtxUserInfo {
            membership_number: parsed.str_field("strMbCrdNo").to_string(),
            name: parsed.str_field("strCustNm").to_string(),
            email: parsed.str_field("strEmailAdr").to_string(),
            phone: parsed.str_field("strCpNo").to_string(),
        });
        self.is_logged_in = true;

        Ok(())
    }

    // ── Logout ───────────────────────────────────────────────────────

    /// Log out from the KTX server.
    #[instrument(skip(self))]
    pub async fn logout(&mut self) -> Result<(), ProviderError> {
        if !self.is_logged_in {
            return Ok(());
        }

        let form = Self::base_params();

        let resp = self
            .client
            .post(Endpoints::url(Endpoints::LOGOUT))
            .form(&form)
            .send()
            .await?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(ProviderError::UnexpectedResponse { status: 500, body });
        }

        self.is_logged_in = false;
        self.user_info = None;
        Ok(())
    }

    // ── Search ───────────────────────────────────────────────────────

    /// Search for KTX trains (1 adult default).
    ///
    /// - `dep`/`arr`: Korean station names (e.g., "서울", "부산")
    /// - `date`: YYYYMMDD format
    /// - `time`: HHMMSS format (default: "000000")
    /// - `available_only`: filter to trains with available seats
    #[instrument(skip(self))]
    pub async fn search_train(
        &self,
        dep: &str,
        arr: &str,
        date: Option<&str>,
        time: Option<&str>,
        available_only: bool,
    ) -> Result<Vec<KtxTrain>, ProviderError> {
        self.search_train_with_passengers(dep, arr, date, time, &[], available_only)
            .await
    }

    /// Search for KTX trains with explicit passenger groups.
    #[instrument(skip(self, passengers))]
    pub async fn search_train_with_passengers(
        &self,
        dep: &str,
        arr: &str,
        date: Option<&str>,
        time: Option<&str>,
        passengers: &[PassengerGroup],
        available_only: bool,
    ) -> Result<Vec<KtxTrain>, ProviderError> {
        let dep_code = station_code(dep).ok_or_else(|| ProviderError::UnexpectedResponse {
            status: 400,
            body: format!("Unknown departure station: {dep}"),
        })?;
        let arr_code = station_code(arr).ok_or_else(|| ProviderError::UnexpectedResponse {
            status: 400,
            body: format!("Unknown arrival station: {arr}"),
        })?;

        let today = chrono::Local::now().format("%Y%m%d").to_string();
        let date = date.unwrap_or(&today);
        let time = time.unwrap_or("000000");

        let membership_number = self
            .user_info
            .as_ref()
            .map(|u| u.membership_number.as_str())
            .unwrap_or("");

        // Tally passenger counts per type flag
        let psg_counts = ktx_passenger_flag_counts(passengers);

        let (builder, sid) =
            self.request_with_dynapath(reqwest::Method::POST, &Endpoints::url(Endpoints::SEARCH));

        let params: Vec<(&str, &str)> = vec![
            ("Device", DEVICE),
            ("Version", VERSION),
            ("Sid", &sid),
            ("txtMenuId", "11"),
            ("radJobId", "1"),
            ("selGoTrain", "109"),
            ("txtTrnGpCd", "109"),
            ("txtGoStart", dep_code),
            ("txtGoEnd", arr_code),
            ("txtGoAbrdDt", date),
            ("txtGoHour", time),
            ("txtPsgFlg_1", &psg_counts.adults),
            ("txtPsgFlg_2", &psg_counts.children),
            ("txtPsgFlg_3", &psg_counts.seniors),
            ("txtPsgFlg_4", &psg_counts.severe),
            ("txtPsgFlg_5", &psg_counts.mild),
            ("txtSeatAttCd_2", "000"),
            ("txtSeatAttCd_3", "000"),
            ("txtSeatAttCd_4", "015"),
            ("ebizCrossCheck", "N"),
            ("srtCheckYn", "N"),
            ("rtYn", "N"),
            ("adjStnScdlOfrFlg", "N"),
            ("mbCrdNo", membership_number),
        ];

        let resp = builder.form(&params).send().await?;

        let body = resp.text().await?;
        let parsed = KtxResponse::parse(&body)?;

        if !parsed.is_success() {
            return Err(ProviderError::NoResults);
        }

        let trains_json = parsed
            .get("/trn_infos/trn_info")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let mut trains: Vec<KtxTrain> = trains_json
            .iter()
            .filter_map(KtxTrain::from_json)
            .filter(|t| t.is_ktx())
            .collect();

        if available_only {
            trains.retain(|t| t.seat_available());
        }

        Ok(trains)
    }

    // ── Reserve ──────────────────────────────────────────────────────

    /// Reserve a KTX train seat (legacy: count-only, all adults).
    #[instrument(skip(self, train))]
    pub async fn reserve(
        &self,
        train: &KtxTrain,
        seat_pref: SeatPreference,
        passengers: u8,
    ) -> Result<KtxReservation, ProviderError> {
        let groups = vec![PassengerGroup::adults(passengers.max(1))];
        self.reserve_with_passengers(train, &groups, seat_pref)
            .await
    }

    /// Reserve a KTX train seat with explicit passenger groups.
    #[instrument(skip(self, train, passengers))]
    pub async fn reserve_with_passengers(
        &self,
        train: &KtxTrain,
        passengers: &[PassengerGroup],
        seat_pref: SeatPreference,
    ) -> Result<KtxReservation, ProviderError> {
        self.require_login()?;

        let is_special = match seat_pref {
            SeatPreference::GeneralOnly => false,
            SeatPreference::SpecialOnly => true,
            SeatPreference::GeneralFirst => !train.general_seat_available(),
            SeatPreference::SpecialFirst => train.special_seat_available(),
        };
        let seat_class = if is_special { "2" } else { "1" };

        let default_passengers = [PassengerGroup::adults(1)];
        let passengers = if passengers.is_empty() {
            &default_passengers
        } else {
            passengers
        };
        let combined = combine_passengers(passengers);
        let total = total_count(&combined);
        let total_str = total.to_string();

        let mut params: Vec<(String, String)> = vec![
            ("Device".into(), DEVICE.into()),
            ("Version".into(), VERSION.into()),
            ("Key".into(), KEY.into()),
            ("txtMenuId".into(), "11".into()),
            ("txtJobId".into(), "1101".into()),
            ("txtGdNo".into(), String::new()),
            ("hidFreeFlg".into(), "N".into()),
            ("txtTotPsgCnt".into(), total_str),
            ("txtSeatAttCd1".into(), "000".into()),
            ("txtSeatAttCd2".into(), "000".into()),
            ("txtSeatAttCd3".into(), "000".into()),
            ("txtSeatAttCd4".into(), "015".into()),
            ("txtSeatAttCd5".into(), "000".into()),
            ("txtStndFlg".into(), "N".into()),
            ("txtSrcarCnt".into(), "0".into()),
            ("txtJrnyCnt".into(), "1".into()),
            ("txtJrnySqno1".into(), "001".into()),
            ("txtJrnyTpCd1".into(), "11".into()),
            ("txtDptDt1".into(), train.dep_date.clone()),
            ("txtDptRsStnCd1".into(), train.dep_code.clone()),
            ("txtDptTm1".into(), train.dep_time.clone()),
            ("txtArvRsStnCd1".into(), train.arr_code.clone()),
            ("txtTrnNo1".into(), train.train_no.clone()),
            ("txtRunDt1".into(), train.run_date.clone()),
            ("txtTrnClsfCd1".into(), train.train_type.clone()),
            ("txtTrnGpCd1".into(), train.train_group.clone()),
            ("txtPsrmClCd1".into(), seat_class.into()),
            ("txtChgFlg1".into(), String::new()),
        ];

        // Encode per-type passenger groups
        for (idx, group) in combined.iter().enumerate() {
            let i = idx + 1;
            let (type_code, disc_code) = ktx_passenger_codes(group.passenger_type);
            params.push((format!("txtPsgTpCd{i}"), type_code.into()));
            params.push((format!("txtDiscKndCd{i}"), disc_code.into()));
            params.push((format!("txtCompaCnt{i}"), group.count.to_string()));
            params.push((format!("txtCardCode_{i}"), String::new()));
            params.push((format!("txtCardNo_{i}"), String::new()));
            params.push((format!("txtCardPw_{i}"), String::new()));
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();

        let (builder, _) =
            self.request_with_dynapath(reqwest::Method::POST, &Endpoints::url(Endpoints::RESERVE));
        let resp = builder.form(&params_ref).send().await?;

        let body = resp.text().await?;
        let parsed = KtxResponse::parse(&body)?;

        if !parsed.is_success() {
            let msg = parsed.message();
            if msg.contains("이미 예약") || msg.contains("중복") {
                return Err(ProviderError::DuplicateReservation);
            }
            return Err(ProviderError::UnexpectedResponse {
                status: 200,
                body: msg.to_string(),
            });
        }

        let pnr_no = parsed.str_field("h_pnr_no");
        if pnr_no.is_empty() {
            return Err(ProviderError::UnexpectedResponse {
                status: 200,
                body: "Missing h_pnr_no in reserve response".to_string(),
            });
        }

        // Fetch full reservation details
        let reservations = self.get_reservations().await?;
        reservations
            .into_iter()
            .find(|r| r.rsv_id == pnr_no)
            .ok_or_else(|| ProviderError::UnexpectedResponse {
                status: 200,
                body: format!("Reservation {pnr_no} not found after reserve"),
            })
    }

    // ── Get Reservations ─────────────────────────────────────────────

    /// Get all current reservations.
    #[instrument(skip(self))]
    pub async fn get_reservations(&self) -> Result<Vec<KtxReservation>, ProviderError> {
        self.require_login()?;

        let params = Self::base_params();

        let resp = self
            .client
            .post(Endpoints::url(Endpoints::RESERVATION_VIEW))
            .form(&params)
            .send()
            .await?;

        let body = resp.text().await?;
        let parsed = KtxResponse::parse(&body)?;

        if !parsed.is_success() {
            return Err(ProviderError::NoResults);
        }

        let journeys = parsed
            .get("/jrny_infos/jrny_info")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let mut reservations = Vec::new();

        // Collect non-empty PNR numbers and their journey refs
        let pnr_journeys: Vec<(&serde_json::Value, String)> = journeys
            .iter()
            .filter_map(|jrny| {
                let pnr_no = jrny.get("h_pnr_no")?.as_str()?;
                if pnr_no.is_empty() {
                    None
                } else {
                    Some((jrny, pnr_no.to_string()))
                }
            })
            .collect();

        // Fetch all ticket details in parallel
        let ticket_futures: Vec<_> = pnr_journeys
            .iter()
            .map(|(_, pnr)| self.ticket_info(pnr))
            .collect();
        let ticket_results = futures_util::future::join_all(ticket_futures).await;

        for ((jrny, _), tickets_result) in pnr_journeys.iter().zip(ticket_results) {
            match tickets_result {
                Ok(tickets) => {
                    if let Some(rsv) = KtxReservation::from_json(jrny, tickets) {
                        reservations.push(rsv);
                    }
                }
                Err(e) => {
                    tracing::warn!(error = %e, "Failed to fetch ticket info, skipping reservation");
                }
            }
        }

        Ok(reservations)
    }

    // ── Ticket Info ──────────────────────────────────────────────────

    /// Get ticket details for a reservation.
    #[instrument(skip(self))]
    pub async fn ticket_info(&self, pnr_no: &str) -> Result<Vec<KtxTicket>, ProviderError> {
        self.require_login()?;

        let params: Vec<(&str, &str)> = vec![
            ("Device", DEVICE),
            ("Version", VERSION),
            ("Key", KEY),
            ("h_pnr_no", pnr_no),
        ];

        let resp = self
            .client
            .post(Endpoints::url(Endpoints::RESERVATION_LIST))
            .form(&params)
            .send()
            .await?;

        let body = resp.text().await?;
        let parsed = KtxResponse::parse(&body)?;

        if !parsed.is_success() {
            return Err(ProviderError::UnexpectedResponse {
                status: 200,
                body: parsed.message().to_string(),
            });
        }

        let ticket_data = parsed
            .get("/tk_infos/tk_info")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        Ok(ticket_data
            .iter()
            .filter_map(KtxTicket::from_json)
            .collect())
    }

    // ── Cancel ───────────────────────────────────────────────────────

    /// Cancel a reservation.
    #[instrument(skip(self))]
    pub async fn cancel(&self, reservation: &KtxReservation) -> Result<(), ProviderError> {
        self.require_login()?;

        let form: Vec<(&str, &str)> = vec![
            ("Device", DEVICE),
            ("Version", VERSION),
            ("Key", KEY),
            ("txtPnrNo", &reservation.rsv_id),
            ("txtJrnySqno", &reservation.journey_no),
            ("txtJrnyCnt", &reservation.journey_cnt),
            ("hidRsvChgNo", &reservation.rsv_chg_no),
        ];

        let resp = self
            .client
            .post(Endpoints::url(Endpoints::CANCEL))
            .form(&form)
            .send()
            .await?;

        let body = resp.text().await?;
        let parsed = KtxResponse::parse(&body)?;

        if !parsed.is_success() {
            return Err(ProviderError::UnexpectedResponse {
                status: 200,
                body: parsed.message().to_string(),
            });
        }

        Ok(())
    }

    // ── Payment ──────────────────────────────────────────────────────

    /// Pay for a reservation with a credit card.
    ///
    /// - `card_number`: card number (no hyphens, 13-19 digits)
    /// - `card_password`: first 2 digits of card password
    /// - `birthday`: YYMMDD format for personal cards
    /// - `card_expire`: MMYY format (note: MMYY, not YYMM like SRT)
    /// - `installment`: "0" for lump sum, "2"-"24" for installment
    /// - `card_type`: "J" for personal, "S" for corporate
    #[allow(clippy::too_many_arguments)]
    #[instrument(skip(self, card_number, card_password, birthday))]
    pub async fn pay_with_card(
        &self,
        reservation: &KtxReservation,
        card_number: &str,
        card_password: &str,
        birthday: &str,
        card_expire: &str,
        installment: &str,
        card_type: &str,
    ) -> Result<(), ProviderError> {
        self.require_login()?;

        // Validate card inputs
        Self::validate_card_inputs(card_number, card_password, card_expire, card_type)?;

        if reservation.is_waiting {
            return Err(ProviderError::UnexpectedResponse {
                status: 400,
                body: "Cannot pay for standby/waiting reservations".to_string(),
            });
        }

        let form: Vec<(&str, &str)> = vec![
            ("Device", DEVICE),
            ("Version", VERSION),
            ("Key", KEY),
            ("hidPnrNo", &reservation.rsv_id),
            ("hidWctNo", &reservation.wct_no),
            ("hidTmpJobSqno1", "000000"),
            ("hidTmpJobSqno2", "000000"),
            ("hidRsvChgNo", "000"),
            ("hidInrecmnsGridcnt", "1"),
            ("hidStlMnsSqno1", "1"),
            ("hidStlMnsCd1", "02"),
            ("hidMnsStlAmt1", &reservation.price),
            ("hidCrdInpWayCd1", "@"),
            ("hidStlCrCrdNo1", card_number),
            ("hidVanPwd1", card_password),
            ("hidCrdVlidTrm1", card_expire),
            ("hidIsmtMnthNum1", installment),
            ("hidAthnDvCd1", card_type),
            ("hidAthnVal1", birthday),
            ("hiduserYn", "Y"),
        ];

        let resp = self
            .client
            .post(Endpoints::url(Endpoints::PAYMENT))
            .form(&form)
            .send()
            .await?;

        let body = resp.text().await?;
        let parsed = KtxResponse::parse(&body)?;

        if !parsed.is_success() {
            return Err(ProviderError::UnexpectedResponse {
                status: 200,
                body: parsed.message().to_string(),
            });
        }

        Ok(())
    }

    // ── Refund ───────────────────────────────────────────────────────

    /// Refund a paid ticket.
    #[instrument(skip(self))]
    pub async fn refund(&self, ticket: &KtxTicket) -> Result<(), ProviderError> {
        self.require_login()?;

        let form: Vec<(&str, &str)> = vec![
            ("Device", DEVICE),
            ("Version", VERSION),
            ("Key", KEY),
            ("txtPrnNo", &ticket.pnr_no),
            ("h_orgtk_sale_dt", &ticket.sale_info2),
            ("h_orgtk_sale_wct_no", &ticket.sale_info1),
            ("h_orgtk_sale_sqno", &ticket.sale_info3),
            ("h_orgtk_ret_pwd", &ticket.sale_info4),
            ("h_mlg_stl", "N"),
            ("tk_ret_tms_dv_cd", "21"),
            ("trnNo", &ticket.train_no),
            ("pbpAcepTgtFlg", "N"),
            ("latitude", ""),
            ("longitude", ""),
        ];

        let resp = self
            .client
            .post(Endpoints::url(Endpoints::REFUND))
            .form(&form)
            .send()
            .await?;

        let body = resp.text().await?;
        let parsed = KtxResponse::parse(&body)?;

        if !parsed.is_success() {
            return Err(ProviderError::UnexpectedResponse {
                status: 200,
                body: parsed.message().to_string(),
            });
        }

        Ok(())
    }

    // ── Helpers ───────────────────────────────────────────────────────

    fn validate_card_inputs(
        card_number: &str,
        card_password: &str,
        card_expire: &str,
        card_type: &str,
    ) -> Result<(), ProviderError> {
        // Evervault-encrypted values (ev: prefix) bypass digit validation —
        // the Outbound Relay decrypts them to plaintext before forwarding.
        if !card_number.starts_with("ev:") {
            let card_digits = card_number.chars().all(|c| c.is_ascii_digit());
            if !card_digits || card_number.len() < 13 || card_number.len() > 19 {
                return Err(ProviderError::UnexpectedResponse {
                    status: 400,
                    body: "Card number must be 13-19 digits".to_string(),
                });
            }
        }
        if !card_password.starts_with("ev:")
            && (card_password.len() != 2 || !card_password.chars().all(|c| c.is_ascii_digit()))
        {
            return Err(ProviderError::UnexpectedResponse {
                status: 400,
                body: "Card password must be exactly 2 digits".to_string(),
            });
        }
        if !card_expire.starts_with("ev:")
            && (card_expire.len() != 4 || !card_expire.chars().all(|c| c.is_ascii_digit()))
        {
            return Err(ProviderError::UnexpectedResponse {
                status: 400,
                body: "Card expire must be 4 digits (MMYY)".to_string(),
            });
        }
        if card_type != "J" && card_type != "S" {
            return Err(ProviderError::UnexpectedResponse {
                status: 400,
                body: "Card type must be \"J\" (personal) or \"S\" (corporate)".to_string(),
            });
        }
        Ok(())
    }

    fn require_login(&self) -> Result<(), ProviderError> {
        if !self.is_logged_in {
            Err(ProviderError::SessionExpired)
        } else {
            Ok(())
        }
    }
}

impl Default for KtxClient {
    fn default() -> Self {
        Self::new()
    }
}

// ── KTX passenger helpers ─────────────────────────────────────────────

/// Per-flag passenger counts for KTX search form.
struct KtxPassengerFlags {
    adults: String,
    children: String,
    seniors: String,
    severe: String,
    mild: String,
}

/// Map passenger groups to KTX search flag counts (txtPsgFlg_1..5).
/// Empty slice defaults to 1 adult.
fn ktx_passenger_flag_counts(passengers: &[PassengerGroup]) -> KtxPassengerFlags {
    let combined = combine_passengers(passengers);
    let count_of = |pt: PassengerType| -> u8 {
        combined
            .iter()
            .find(|g| g.passenger_type == pt)
            .map(|g| g.count)
            .unwrap_or(0)
    };

    let adults = count_of(PassengerType::Adult);
    KtxPassengerFlags {
        adults: if adults == 0 && combined.is_empty() {
            "1".to_string()
        } else {
            adults.to_string()
        },
        children: count_of(PassengerType::Child).to_string(),
        seniors: count_of(PassengerType::Senior).to_string(),
        severe: count_of(PassengerType::SevereDisability).to_string(),
        mild: count_of(PassengerType::MildDisability).to_string(),
    }
}

/// Map a PassengerType to (KTX type code, discount kind code) for reserve form.
fn ktx_passenger_codes(ptype: PassengerType) -> (&'static str, &'static str) {
    match ptype {
        PassengerType::Adult => ("1", "000"),
        PassengerType::Child => ("3", "101"),
        PassengerType::Senior => ("1", "105"),
        PassengerType::SevereDisability => ("1", "106"),
        PassengerType::MildDisability => ("1", "107"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_code_classification() {
        assert_eq!(KtxClient::auth_code("user@example.com"), "5");
        assert_eq!(KtxClient::auth_code("010-1234-5678"), "4");
        assert_eq!(KtxClient::auth_code("1234567890"), "2");
    }

    #[test]
    fn base_params_include_device() {
        let params = KtxClient::base_params();
        assert!(params.iter().any(|(k, v)| *k == "Device" && *v == "AD"));
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "Version" && *v == "250601002")
        );
    }

    #[test]
    fn require_login_when_not_logged_in() {
        let client = KtxClient::new();
        assert!(client.require_login().is_err());
    }

    #[test]
    fn endpoint_urls() {
        assert!(Endpoints::url(Endpoints::LOGIN).contains("login.Login"));
        assert!(Endpoints::url(Endpoints::SEARCH).contains("ScheduleView"));
        assert!(Endpoints::url(Endpoints::RESERVE).contains("TicketReservation"));
    }

    #[test]
    fn validate_card_valid() {
        assert!(KtxClient::validate_card_inputs("4111111111111111", "12", "1226", "J").is_ok());
    }

    #[test]
    fn validate_card_short_number() {
        assert!(KtxClient::validate_card_inputs("123456", "12", "1226", "J").is_err());
    }

    #[test]
    fn validate_card_bad_password() {
        assert!(KtxClient::validate_card_inputs("4111111111111111", "1", "1226", "J").is_err());
    }

    #[test]
    fn validate_card_bad_expiry() {
        assert!(KtxClient::validate_card_inputs("4111111111111111", "12", "26", "J").is_err());
    }

    #[test]
    fn validate_card_bad_type() {
        assert!(KtxClient::validate_card_inputs("4111111111111111", "12", "1226", "X").is_err());
    }

    #[test]
    fn sid_is_generated() {
        let sid = KtxClient::current_sid();
        assert!(sid.ends_with('\n'));
        assert!(!sid.is_empty());
    }

    // ── KTX passenger helper tests ──────────────────────────────────

    #[test]
    fn ktx_flag_counts_default_one_adult() {
        let flags = ktx_passenger_flag_counts(&[]);
        assert_eq!(flags.adults, "1");
        assert_eq!(flags.children, "0");
        assert_eq!(flags.seniors, "0");
        assert_eq!(flags.severe, "0");
        assert_eq!(flags.mild, "0");
    }

    #[test]
    fn ktx_flag_counts_mixed_passengers() {
        let groups = vec![
            PassengerGroup::adults(2),
            PassengerGroup::new(PassengerType::Child, 1),
            PassengerGroup::new(PassengerType::Senior, 1),
        ];
        let flags = ktx_passenger_flag_counts(&groups);
        assert_eq!(flags.adults, "2");
        assert_eq!(flags.children, "1");
        assert_eq!(flags.seniors, "1");
        assert_eq!(flags.severe, "0");
        assert_eq!(flags.mild, "0");
    }

    #[test]
    fn ktx_flag_counts_disability() {
        let groups = vec![
            PassengerGroup::new(PassengerType::SevereDisability, 1),
            PassengerGroup::new(PassengerType::MildDisability, 2),
        ];
        let flags = ktx_passenger_flag_counts(&groups);
        assert_eq!(flags.adults, "0");
        assert_eq!(flags.severe, "1");
        assert_eq!(flags.mild, "2");
    }

    #[test]
    fn ktx_passenger_codes_adult() {
        let (tp, disc) = ktx_passenger_codes(PassengerType::Adult);
        assert_eq!(tp, "1");
        assert_eq!(disc, "000");
    }

    #[test]
    fn ktx_passenger_codes_child() {
        let (tp, disc) = ktx_passenger_codes(PassengerType::Child);
        assert_eq!(tp, "3");
        assert_eq!(disc, "101");
    }

    #[test]
    fn ktx_passenger_codes_senior() {
        let (tp, disc) = ktx_passenger_codes(PassengerType::Senior);
        assert_eq!(tp, "1");
        assert_eq!(disc, "105");
    }

    #[test]
    fn ktx_passenger_codes_severe_disability() {
        let (tp, disc) = ktx_passenger_codes(PassengerType::SevereDisability);
        assert_eq!(tp, "1");
        assert_eq!(disc, "106");
    }

    #[test]
    fn ktx_passenger_codes_mild_disability() {
        let (tp, disc) = ktx_passenger_codes(PassengerType::MildDisability);
        assert_eq!(tp, "1");
        assert_eq!(disc, "107");
    }
}
