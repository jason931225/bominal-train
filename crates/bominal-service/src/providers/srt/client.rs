//! SRT HTTP client implementation.
//!
//! Faithfully ported from `third_party/srt/SRT/srt.py`.
//! All requests are form-encoded POST. All responses are JSON.

use tracing::{debug, instrument, warn};
use wreq_util::Emulation;

use super::super::netfunnel::NetFunnelHelper;
use super::super::types::{AuthType, ProviderError, SeatPreference, classify_auth};

use super::passenger::{PassengerGroup, WindowSeat, passenger_form_fields};
use super::reservation::{SrtReservation, SrtTicket};
use super::response::{SrtResponse, parse_payment_response};
use super::stations::station_code;
use super::train::SrtTrain;

// ── Constants ────────────────────────────────────────────────────────

const BASE_URL: &str = "https://app.srail.or.kr:443";

const USER_AGENT: &str = "Mozilla/5.0 (Linux; Android 15; SM-S912N Build/UP1A.231005.007; wv) \
    AppleWebKit/537.36 (KHTML, like Gecko) Version/4.0 Chrome/136.0.7103.125 \
    Mobile Safari/537.36SRT-APP-Android V.2.0.38";

/// API endpoint paths (appended to BASE_URL).
struct Endpoints;

impl Endpoints {
    const MAIN: &str = "/main/main.do";
    const LOGIN: &str = "/apb/selectListApb01080_n.do";
    const LOGOUT: &str = "/login/loginOut.do";
    const SEARCH: &str = "/ara/selectListAra10007_n.do";
    const RESERVE: &str = "/arc/selectListArc05013_n.do";
    const TICKETS: &str = "/atc/selectListAtc14016_n.do";
    const TICKET_INFO: &str = "/ard/selectListArd02019_n.do";
    const CANCEL: &str = "/ard/selectListArd02045_n.do";
    const RESERVE_INFO: &str = "/atc/getListAtc14087.do";
    const REFUND: &str = "/atc/selectListAtc02063_n.do";
    const STANDBY_OPTION: &str = "/ata/selectListAta01135_n.do";
    const PAYMENT: &str = "/ata/selectListAta09036_n.do";

    fn url(path: &str) -> String {
        format!("{BASE_URL}{path}")
    }
}

/// NetFunnel error code that triggers cache invalidation + retry.
const INVALID_NETFUNNEL_KEY: &str = "NET000001";

/// Reserve job IDs.
const JOB_ID_PERSONAL: &str = "1101";
const JOB_ID_STANDBY: &str = "1102";

// ── Client ───────────────────────────────────────────────────────────

/// SRT provider client with persistent session.
///
/// Two separate HTTP clients: one for the SRT API (with cookies), one for NetFunnel.
pub struct SrtClient {
    api_client: wreq::Client,
    netfunnel_client: wreq::Client,
    netfunnel: NetFunnelHelper,
    user_info: Option<SrtUserInfo>,
    is_logged_in: bool,
}

/// User info extracted after successful login.
#[derive(Debug, Clone)]
pub struct SrtUserInfo {
    pub membership_number: String,
    pub name: String,
    pub phone: String,
}

impl SrtClient {
    /// Create a new SRT client with fresh session.
    pub fn new() -> Self {
        let api_client = wreq::Client::builder()
            .emulation(Emulation::Chrome136)
            .cookie_store(true)
            .user_agent(USER_AGENT)
            .default_headers({
                let mut h = wreq::header::HeaderMap::new();
                h.insert(wreq::header::ACCEPT, "application/json".parse().unwrap());
                h
            })
            .build()
            .expect("Failed to build SRT API client");

        let netfunnel_client = wreq::Client::builder()
            .emulation(Emulation::Chrome136)
            .cookie_store(true)
            .build()
            .expect("Failed to build SRT NetFunnel client");

        Self {
            api_client,
            netfunnel_client,
            netfunnel: NetFunnelHelper::new(),
            user_info: None,
            is_logged_in: false,
        }
    }

    /// Create a new SRT client that proxies through an Evervault Relay.
    ///
    /// All requests go through the relay, which transparently decrypts
    /// `ev:`-prefixed card fields in-flight. Cookies, Host, and UA are
    /// preserved because the target URL stays `app.srail.or.kr`.
    pub fn with_relay(relay_domain: &str) -> Self {
        let proxy =
            wreq::Proxy::all(format!("https://{relay_domain}")).expect("Invalid relay domain");

        let api_client = wreq::Client::builder()
            .emulation(Emulation::Chrome136)
            .cookie_store(true)
            .user_agent(USER_AGENT)
            .proxy(proxy)
            .default_headers({
                let mut h = wreq::header::HeaderMap::new();
                h.insert(wreq::header::ACCEPT, "application/json".parse().unwrap());
                h
            })
            .build()
            .expect("Failed to build SRT API client with relay");

        let netfunnel_client = wreq::Client::builder()
            .emulation(Emulation::Chrome136)
            .cookie_store(true)
            .build()
            .expect("Failed to build SRT NetFunnel client");

        Self {
            api_client,
            netfunnel_client,
            netfunnel: NetFunnelHelper::new(),
            user_info: None,
            is_logged_in: false,
        }
    }

    /// Whether the client is currently logged in.
    pub fn is_logged_in(&self) -> bool {
        self.is_logged_in
    }

    /// User info (available after login).
    pub fn user_info(&self) -> Option<&SrtUserInfo> {
        self.user_info.as_ref()
    }

    /// Access the NetFunnel client for external use.
    pub fn netfunnel_client(&self) -> &wreq::Client {
        &self.netfunnel_client
    }

    /// SRT auth code: Email -> "2", Phone -> "3", Membership -> "1".
    fn auth_code(login_id: &str) -> &'static str {
        match classify_auth(login_id) {
            AuthType::Email => "2",
            AuthType::Phone => "3",
            AuthType::Membership => "1",
        }
    }

    /// Strip hyphens from phone numbers for SRT API.
    fn normalize_id(login_id: &str) -> String {
        if classify_auth(login_id) == AuthType::Phone {
            login_id.replace('-', "")
        } else {
            login_id.to_string()
        }
    }

    // ── Login ────────────────────────────────────────────────────────

    /// Log in to the SRT server.
    ///
    /// Error detection uses Korean string search in the raw response body,
    /// NOT JSON status codes, because SRT returns errors as HTTP 200.
    #[instrument(skip(self, password), fields(login_type))]
    pub async fn login(&mut self, login_id: &str, password: &str) -> Result<(), ProviderError> {
        let login_type = Self::auth_code(login_id);
        let normalized_id = Self::normalize_id(login_id);
        let login_referer = Endpoints::url(Endpoints::MAIN);

        let form: Vec<(&str, &str)> = vec![
            ("auto", "Y"),
            ("check", "Y"),
            ("page", "menu"),
            ("deviceKey", "-"),
            ("customerYn", ""),
            ("login_referer", &login_referer),
            ("srchDvCd", login_type),
            ("srchDvNm", &normalized_id),
            // NOTE: SRT protocol sends password in plaintext over HTTPS (by design).
            // The upstream Python client does the same — no client-side encryption.
            ("hmpgPwdCphd", password),
        ];

        let resp = self
            .api_client
            .post(Endpoints::url(Endpoints::LOGIN))
            .form(&form)
            .send()
            .await?;

        let body = resp.text().await?;
        debug!(body_len = body.len(), "SRT login response received");

        // Error detection via Korean string search (before JSON parsing)
        if body.contains("존재하지않는 회원입니다") {
            self.is_logged_in = false;
            return Err(ProviderError::LoginFailed {
                message: "User not found (존재하지않는 회원입니다)".to_string(),
            });
        }
        if body.contains("비밀번호 오류") {
            self.is_logged_in = false;
            return Err(ProviderError::LoginFailed {
                message: "Wrong password (비밀번호 오류)".to_string(),
            });
        }
        if body.contains("Your IP Address Blocked") {
            self.is_logged_in = false;
            return Err(ProviderError::LoginFailed {
                message: body.trim().to_string(),
            });
        }

        // Parse successful login response
        let json: serde_json::Value =
            serde_json::from_str(&body).map_err(|_| ProviderError::UnexpectedResponse {
                status: 200,
                body: body.clone(),
            })?;

        let user_map = json
            .get("userMap")
            .ok_or_else(|| ProviderError::UnexpectedResponse {
                status: 200,
                body: "Missing userMap in login response".to_string(),
            })?;

        let membership_number = user_map
            .get("MB_CRD_NO")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let name = user_map
            .get("CUST_NM")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let phone = user_map
            .get("MBL_PHONE")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        self.user_info = Some(SrtUserInfo {
            membership_number,
            name,
            phone,
        });
        self.is_logged_in = true;

        Ok(())
    }

    // ── Logout ───────────────────────────────────────────────────────

    /// Log out from the SRT server.
    #[instrument(skip(self))]
    pub async fn logout(&mut self) -> Result<(), ProviderError> {
        if !self.is_logged_in {
            return Ok(());
        }

        let resp = self
            .api_client
            .post(Endpoints::url(Endpoints::LOGOUT))
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

    /// Search for SRT trains.
    ///
    /// - `dep`/`arr`: Korean station names (e.g. "수서", "부산")
    /// - `date`: YYYYMMDD format (default: today)
    /// - `time`: HHMMSS format (default: "000000")
    /// - `passenger_count`: total number of passengers (default: 1)
    /// - `available_only`: filter to trains with available seats
    #[instrument(skip(self))]
    pub async fn search_train(
        &mut self,
        dep: &str,
        arr: &str,
        date: Option<&str>,
        time: Option<&str>,
        available_only: bool,
    ) -> Result<Vec<SrtTrain>, ProviderError> {
        self.search_train_with_count(dep, arr, date, time, None, available_only)
            .await
    }

    /// Search for SRT trains with explicit passenger count.
    #[instrument(skip(self))]
    pub async fn search_train_with_count(
        &mut self,
        dep: &str,
        arr: &str,
        date: Option<&str>,
        time: Option<&str>,
        passenger_count: Option<u8>,
        available_only: bool,
    ) -> Result<Vec<SrtTrain>, ProviderError> {
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

        self.search_train_internal(
            dep_code,
            arr_code,
            date,
            time,
            passenger_count,
            available_only,
        )
        .await
    }

    async fn search_train_internal(
        &mut self,
        dep_code: &str,
        arr_code: &str,
        date: &str,
        time: &str,
        passenger_count: Option<u8>,
        available_only: bool,
    ) -> Result<Vec<SrtTrain>, ProviderError> {
        // Acquire NetFunnel key before search
        let nf_key = match self.netfunnel.run(&self.netfunnel_client).await {
            Ok(key) => key,
            Err(e) => {
                warn!(error = %e, "NetFunnel acquisition failed, proceeding without key");
                String::new()
            }
        };

        let psg_num = passenger_count.unwrap_or(1).max(1).to_string();

        let form: Vec<(&str, &str)> = vec![
            ("chtnDvCd", "1"),
            ("arriveTime", "N"),
            ("seatAttCd", "015"),
            ("psgNum", &psg_num),
            ("trnGpCd", "109"),
            ("stlbTrnClsfCd", "05"), // search all train types
            ("dptDt", date),
            ("dptTm", time),
            ("arvRsStnCd", arr_code),
            ("dptRsStnCd", dep_code),
            // Duplicate date/time fields required by reference protocol
            ("dptDt1", date),
            ("dptTm1", time),
            // Empty fields required by reference protocol
            ("trnNo", ""),
            ("tkDptDt", ""),
            ("tkDptTm", ""),
            ("tkTrnNo", ""),
            ("tkTripChgFlg", ""),
            ("dlayTnumAplFlg", "Y"),
            ("netfunnelKey", &nf_key),
        ];

        let resp = self
            .api_client
            .post(Endpoints::url(Endpoints::SEARCH))
            .form(&form)
            .send()
            .await?;

        let body = resp.text().await?;
        let parsed = SrtResponse::parse(&body)?;

        if !parsed.is_success() {
            let msg_code = parsed.message_code();
            if msg_code == INVALID_NETFUNNEL_KEY {
                warn!("Invalid netfunnel key, clearing cache and retrying");
                self.netfunnel.clear();
                return Err(ProviderError::NetFunnelBlocked);
            }
            return Err(ProviderError::NoResults);
        }

        let trains_json = parsed
            .get("/outDataSets/dsOutput1")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let mut trains: Vec<SrtTrain> = trains_json
            .iter()
            .enumerate()
            .filter_map(|(i, t)| match SrtTrain::from_json(t) {
                Some(train) => Some(train),
                None => {
                    warn!(
                        index = i,
                        "Failed to parse SRT train from response — skipping"
                    );
                    None
                }
            })
            .filter(|t| t.is_srt()) // filter SRT only, drop KTX/ITX
            .collect();

        if available_only {
            trains.retain(|t| t.seat_available());
        }

        Ok(trains)
    }

    // ── Reserve ──────────────────────────────────────────────────────

    /// Reserve a train (personal reservation).
    #[instrument(skip(self, train, passengers))]
    pub async fn reserve(
        &mut self,
        train: &SrtTrain,
        passengers: &[PassengerGroup],
        seat_pref: SeatPreference,
        window_seat: WindowSeat,
    ) -> Result<SrtReservation, ProviderError> {
        self.require_login()?;
        self.reserve_internal(
            JOB_ID_PERSONAL,
            train,
            passengers,
            seat_pref,
            window_seat,
            None,
        )
        .await
    }

    /// Reserve standby (waiting list).
    #[instrument(skip(self, train, passengers))]
    pub async fn reserve_standby(
        &mut self,
        train: &SrtTrain,
        passengers: &[PassengerGroup],
        seat_pref: SeatPreference,
        phone: Option<&str>,
    ) -> Result<SrtReservation, ProviderError> {
        self.require_login()?;
        self.reserve_internal(
            JOB_ID_STANDBY,
            train,
            passengers,
            seat_pref.coerce_for_standby(),
            WindowSeat::None,
            phone,
        )
        .await
    }

    async fn reserve_internal(
        &mut self,
        job_id: &str,
        train: &SrtTrain,
        passengers: &[PassengerGroup],
        seat_pref: SeatPreference,
        window_seat: WindowSeat,
        phone: Option<&str>,
    ) -> Result<SrtReservation, ProviderError> {
        if !train.is_srt() {
            return Err(ProviderError::UnexpectedResponse {
                status: 400,
                body: format!("Expected SRT train, got {}", train.display_name()),
            });
        }

        // Acquire NetFunnel key before reserve
        let nf_key = match self.netfunnel.run(&self.netfunnel_client).await {
            Ok(key) => key,
            Err(e) => {
                warn!(error = %e, "NetFunnel acquisition failed for reserve, proceeding without key");
                String::new()
            }
        };

        let default_passengers = [PassengerGroup::adults(1)];
        let passengers = if passengers.is_empty() {
            &default_passengers
        } else {
            passengers
        };

        // Determine seat type based on preference + availability
        let is_special_seat = match seat_pref {
            SeatPreference::GeneralOnly => false,
            SeatPreference::SpecialOnly => true,
            SeatPreference::GeneralFirst => !train.general_seat_available(),
            SeatPreference::SpecialFirst => train.special_seat_available(),
        };

        let train_number: u32 =
            train
                .train_number
                .parse()
                .map_err(|_| ProviderError::UnexpectedResponse {
                    status: 400,
                    body: format!("Invalid train number: {}", train.train_number),
                })?;
        let train_number_padded = format!("{train_number:05}");
        let phone_str = phone.unwrap_or("");

        let mut form: Vec<(String, String)> = vec![
            ("jobId".into(), job_id.into()),
            ("jrnyCnt".into(), "1".into()),
            ("jrnyTpCd".into(), "11".into()),
            ("jrnySqno1".into(), "001".into()),
            ("stndFlg".into(), "N".into()),
            ("trnGpCd1".into(), "300".into()),
            ("trnGpCd".into(), "109".into()),
            ("grpDv".into(), "0".into()),
            ("rtnDv".into(), "0".into()),
            ("stlbTrnClsfCd1".into(), train.train_code.clone()),
            ("dptRsStnCd1".into(), train.dep_station_code.clone()),
            ("dptRsStnCdNm1".into(), train.dep_station_name.clone()),
            ("arvRsStnCd1".into(), train.arr_station_code.clone()),
            ("arvRsStnCdNm1".into(), train.arr_station_name.clone()),
            ("dptDt1".into(), train.dep_date.clone()),
            ("dptTm1".into(), train.dep_time.clone()),
            ("arvTm1".into(), train.arr_time.clone()),
            ("trnNo1".into(), train_number_padded),
            ("runDt1".into(), train.dep_date.clone()),
            (
                "dptStnConsOrdr1".into(),
                train.dep_station_constitution_order.clone(),
            ),
            (
                "arvStnConsOrdr1".into(),
                train.arr_station_constitution_order.clone(),
            ),
            ("dptStnRunOrdr1".into(), train.dep_station_run_order.clone()),
            ("arvStnRunOrdr1".into(), train.arr_station_run_order.clone()),
            ("mblPhone".into(), phone_str.to_string()),
            ("netfunnelKey".into(), nf_key),
        ];

        // Personal reservation gets reserveType
        if job_id == JOB_ID_PERSONAL {
            form.push(("reserveType".into(), "11".into()));
        }

        // Add passenger fields
        form.extend(passenger_form_fields(
            passengers,
            is_special_seat,
            window_seat,
        ));

        let resp = self
            .api_client
            .post(Endpoints::url(Endpoints::RESERVE))
            .form(&form)
            .send()
            .await?;

        let body = resp.text().await?;
        let parsed = SrtResponse::parse(&body)?;

        if !parsed.is_success() {
            let msg = parsed.message();
            let msg_code = parsed.message_code();
            if msg_code == INVALID_NETFUNNEL_KEY {
                warn!("Invalid netfunnel key in reserve, clearing cache");
                self.netfunnel.clear();
                return Err(ProviderError::NetFunnelBlocked);
            }
            if msg.contains("이미 예약") || msg.contains("중복") {
                return Err(ProviderError::DuplicateReservation);
            }
            return Err(ProviderError::UnexpectedResponse {
                status: 200,
                body: msg.to_string(),
            });
        }

        // Extract pnrNo from reservation result
        let pnr_no = parsed
            .get("/reservListMap/0/pnrNo")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ProviderError::UnexpectedResponse {
                status: 200,
                body: "Missing pnrNo in reserve response".to_string(),
            })?
            .to_string();

        // Must call get_reservations() to get full reservation data
        let reservations = self.get_reservations().await?;
        reservations
            .into_iter()
            .find(|r| r.reservation_number == pnr_no)
            .ok_or_else(|| ProviderError::UnexpectedResponse {
                status: 200,
                body: format!("Reservation {pnr_no} not found after reserve"),
            })
    }

    // ── Standby Option Settings ──────────────────────────────────────

    /// Apply standby reservation options (SMS, class change agreement).
    #[instrument(skip(self))]
    pub async fn reserve_standby_option_settings(
        &self,
        pnr_no: &str,
        agree_sms: bool,
        agree_class_change: bool,
        phone: Option<&str>,
    ) -> Result<bool, ProviderError> {
        self.require_login()?;

        let class_change_flag = if agree_class_change { "Y" } else { "N" };
        let sms_flag = if agree_sms { "Y" } else { "N" };
        let tel = if agree_sms { phone.unwrap_or("") } else { "" };

        let form: Vec<(&str, &str)> = vec![
            ("pnrNo", pnr_no),
            ("psrmClChgFlg", class_change_flag),
            ("smsSndFlg", sms_flag),
            ("telNo", tel),
        ];

        let resp = self
            .api_client
            .post(Endpoints::url(Endpoints::STANDBY_OPTION))
            .form(&form)
            .send()
            .await?;

        // Success check: HTTP 200 only (no body parsing)
        Ok(resp.status().is_success())
    }

    // ── Get Reservations ─────────────────────────────────────────────

    /// Get all current reservations.
    #[instrument(skip(self))]
    pub async fn get_reservations(&self) -> Result<Vec<SrtReservation>, ProviderError> {
        self.require_login()?;

        let form: Vec<(&str, &str)> = vec![("pageNo", "0")];

        let resp = self
            .api_client
            .post(Endpoints::url(Endpoints::TICKETS))
            .form(&form)
            .send()
            .await?;

        let body = resp.text().await?;
        let parsed = SrtResponse::parse(&body)?;

        if !parsed.is_success() {
            return Err(ProviderError::NoResults);
        }

        let train_data = parsed
            .get("/trainListMap")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let pay_data = parsed
            .get("/payListMap")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let mut reservations = Vec::new();

        if train_data.len() != pay_data.len() {
            warn!(
                trains = train_data.len(),
                payments = pay_data.len(),
                "SRT reservation data length mismatch — some reservations may be missing"
            );
        }

        for (train, pay) in train_data.iter().zip(pay_data.iter()) {
            let pnr_no = train.get("pnrNo").and_then(|v| v.as_str()).unwrap_or("");

            let tickets = self.ticket_info(pnr_no).await?;

            if let Some(reservation) = SrtReservation::from_json(train, pay, tickets) {
                reservations.push(reservation);
            }
        }

        Ok(reservations)
    }

    // ── Ticket Info ──────────────────────────────────────────────────

    /// Get ticket details for a reservation.
    #[instrument(skip(self))]
    pub async fn ticket_info(&self, pnr_no: &str) -> Result<Vec<SrtTicket>, ProviderError> {
        self.require_login()?;

        let form: Vec<(&str, &str)> = vec![("pnrNo", pnr_no), ("jrnySqno", "1")];

        let resp = self
            .api_client
            .post(Endpoints::url(Endpoints::TICKET_INFO))
            .form(&form)
            .send()
            .await?;

        let body = resp.text().await?;
        let parsed = SrtResponse::parse(&body)?;

        if !parsed.is_success() {
            return Err(ProviderError::UnexpectedResponse {
                status: 200,
                body: parsed.message().to_string(),
            });
        }

        let tickets = parsed
            .get("/trainListMap")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(SrtTicket::from_json).collect())
            .unwrap_or_default();

        Ok(tickets)
    }

    // ── Cancel ───────────────────────────────────────────────────────

    /// Cancel a reservation.
    #[instrument(skip(self))]
    pub async fn cancel(&self, pnr_no: &str) -> Result<(), ProviderError> {
        self.require_login()?;

        let form: Vec<(&str, &str)> = vec![("pnrNo", pnr_no), ("jrnyCnt", "1"), ("rsvChgTno", "0")];

        let resp = self
            .api_client
            .post(Endpoints::url(Endpoints::CANCEL))
            .form(&form)
            .send()
            .await?;

        let body = resp.text().await?;
        let parsed = SrtResponse::parse(&body)?;

        if !parsed.is_success() {
            return Err(ProviderError::UnexpectedResponse {
                status: 200,
                body: parsed.message().to_string(),
            });
        }

        Ok(())
    }

    // ── Reserve Info ─────────────────────────────────────────────────

    /// Get detailed reservation info (used before refund).
    #[instrument(skip(self))]
    pub async fn reserve_info(&self, pnr_no: &str) -> Result<serde_json::Value, ProviderError> {
        self.require_login()?;

        let form: Vec<(&str, &str)> = vec![("pnrNo", pnr_no), ("jrnySqno", "1")];

        let resp = self
            .api_client
            .post(Endpoints::url(Endpoints::RESERVE_INFO))
            .form(&form)
            .send()
            .await?;

        let body = resp.text().await?;
        let parsed = SrtResponse::parse(&body)?;

        if !parsed.is_success() {
            return Err(ProviderError::UnexpectedResponse {
                status: 200,
                body: parsed.message().to_string(),
            });
        }

        Ok(parsed.json().clone())
    }

    // ── Refund ──────────────────────────────────────────────────────

    /// Refund a paid reservation.
    #[instrument(skip(self))]
    pub async fn refund(&self, pnr_no: &str) -> Result<(), ProviderError> {
        self.require_login()?;

        let form: Vec<(&str, &str)> = vec![("pnrNo", pnr_no), ("jrnyCnt", "1"), ("rsvChgTno", "0")];

        let resp = self
            .api_client
            .post(Endpoints::url(Endpoints::REFUND))
            .form(&form)
            .send()
            .await?;

        let body = resp.text().await?;
        let parsed = SrtResponse::parse(&body)?;

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
    /// - `validation_number`: birthday (6 digits) for personal, or business number (10 digits) for corporate
    /// - `expire_date`: card expiry in YYMM format (4 digits)
    /// - `installment`: 0 for lump sum, 2-24 for installment months
    /// - `card_type`: "J" for personal, "S" for corporate
    #[allow(clippy::too_many_arguments)]
    #[instrument(skip(self, card_number, card_password, validation_number))]
    pub async fn pay_with_card(
        &self,
        reservation: &SrtReservation,
        card_number: &str,
        card_password: &str,
        validation_number: &str,
        expire_date: &str,
        installment: u8,
        card_type: &str,
    ) -> Result<(), ProviderError> {
        self.require_login()?;

        // Validate card inputs
        Self::validate_card_inputs(
            card_number,
            card_password,
            expire_date,
            card_type,
            installment,
        )?;

        if reservation.is_waiting {
            return Err(ProviderError::UnexpectedResponse {
                status: 400,
                body: "Cannot pay for standby/waiting reservations".to_string(),
            });
        }

        let membership_number = self
            .user_info
            .as_ref()
            .map(|u| u.membership_number.as_str())
            .unwrap_or("");

        let today = chrono::Local::now().format("%Y%m%d").to_string();
        let installment_str = installment.to_string();

        let form: Vec<(&str, &str)> = vec![
            ("stlDmnDt", &today),
            ("mbCrdNo", membership_number),
            ("stlMnsSqno1", "1"),
            ("ststlGridcnt", "1"),
            ("totNewStlAmt", &reservation.total_cost),
            ("athnDvCd1", card_type),
            ("vanPwd1", card_password),
            ("crdVlidTrm1", expire_date),
            ("stlMnsCd1", "02"),
            ("rsvChgTno", "0"),
            ("chgMcs", "0"),
            ("ismtMnthNum1", &installment_str),
            ("ctlDvCd", "3102"),
            ("cgPsId", "korail"),
            ("pnrNo", &reservation.reservation_number),
            ("totPrnb", &reservation.seat_count),
            ("mnsStlAmt1", &reservation.total_cost),
            ("crdInpWayCd1", "@"),
            ("athnVal1", validation_number),
            ("stlCrCrdNo1", card_number),
            ("jrnyCnt", "1"),
            ("strJobId", "3102"),
            ("inrecmnsGridcnt", "1"),
            ("dptTm", &reservation.dep_time),
            ("arvTm", &reservation.arr_time),
            ("dptStnConsOrdr2", "000000"),
            ("arvStnConsOrdr2", "000000"),
            ("trnGpCd", "300"),
            ("pageNo", "-"),
            ("rowCnt", "-"),
            ("pageUrl", ""),
        ];

        let resp = self
            .api_client
            .post(Endpoints::url(Endpoints::PAYMENT))
            .form(&form)
            .send()
            .await?;

        let body = resp.text().await?;
        parse_payment_response(&body)
    }

    // ── Helpers ───────────────────────────────────────────────────────

    fn validate_card_inputs(
        card_number: &str,
        card_password: &str,
        expire_date: &str,
        card_type: &str,
        installment: u8,
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
        if !expire_date.starts_with("ev:")
            && (expire_date.len() != 4 || !expire_date.chars().all(|c| c.is_ascii_digit()))
        {
            return Err(ProviderError::UnexpectedResponse {
                status: 400,
                body: "Expire date must be 4 digits (YYMM)".to_string(),
            });
        }
        if card_type != "J" && card_type != "S" {
            return Err(ProviderError::UnexpectedResponse {
                status: 400,
                body: "Card type must be \"J\" (personal) or \"S\" (corporate)".to_string(),
            });
        }
        if installment != 0 && !(2..=24).contains(&installment) {
            return Err(ProviderError::UnexpectedResponse {
                status: 400,
                body: "Installment must be 0 (lump sum) or 2-24 months".to_string(),
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

impl Default for SrtClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_code_classification() {
        assert_eq!(SrtClient::auth_code("user@example.com"), "2");
        assert_eq!(SrtClient::auth_code("010-1234-5678"), "3");
        assert_eq!(SrtClient::auth_code("1234567890"), "1");
    }

    #[test]
    fn normalize_phone() {
        assert_eq!(SrtClient::normalize_id("010-1234-5678"), "01012345678");
    }

    #[test]
    fn normalize_email() {
        assert_eq!(
            SrtClient::normalize_id("user@example.com"),
            "user@example.com"
        );
    }

    #[test]
    fn normalize_membership() {
        assert_eq!(SrtClient::normalize_id("1234567890"), "1234567890");
    }

    #[test]
    fn require_login_when_not_logged_in() {
        let client = SrtClient::new();
        assert!(client.require_login().is_err());
    }

    #[test]
    fn train_number_zero_padding() {
        let num: u32 = 305;
        assert_eq!(format!("{num:05}"), "00305");
    }

    #[test]
    fn train_number_padding_large() {
        let num: u32 = 12345;
        assert_eq!(format!("{num:05}"), "12345");
    }

    #[test]
    fn validate_card_valid() {
        assert!(SrtClient::validate_card_inputs("4111111111111111", "12", "2612", "J", 0).is_ok());
    }

    #[test]
    fn validate_card_short_number() {
        assert!(SrtClient::validate_card_inputs("123456", "12", "2612", "J", 0).is_err());
    }

    #[test]
    fn validate_card_bad_password() {
        assert!(SrtClient::validate_card_inputs("4111111111111111", "1", "2612", "J", 0).is_err());
    }

    #[test]
    fn validate_card_bad_expiry() {
        assert!(SrtClient::validate_card_inputs("4111111111111111", "12", "26", "J", 0).is_err());
    }

    #[test]
    fn validate_card_bad_type() {
        assert!(SrtClient::validate_card_inputs("4111111111111111", "12", "2612", "X", 0).is_err());
    }

    #[test]
    fn validate_card_bad_installment() {
        assert!(SrtClient::validate_card_inputs("4111111111111111", "12", "2612", "J", 1).is_err());
        assert!(
            SrtClient::validate_card_inputs("4111111111111111", "12", "2612", "J", 25).is_err()
        );
    }
}
