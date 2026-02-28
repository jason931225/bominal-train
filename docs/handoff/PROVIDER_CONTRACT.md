# Bominal Provider Contract Reference (Tools-Only)

## 1) Scope And Precedence

- Canonical SRT runtime contract for bominal: `third_party/srtgo/srtgo/srt.py`
- Supplemental SRT context: `third_party/srt/SRT/*`
- Supplemental KTX/Korail context: `third_party/srtgo/srtgo/ktx.py`, `third_party/korail2/korail2/korail2.py`
- Web/mobile-web supplement (non-canonical):
  - `/Users/jasonlee/Downloads/flows`
  - user-provided cURL traces (운행시간/운임요금)
- Captured runtime evidence:
  - `docs/handoff/output/*/*.json`
- Field-map artifacts:
  - `docs/handoff/PROVIDER_FIELD_MAP.md`
  - `docs/handoff/PROVIDER_FIELD_MAP.json`
  - `docs/handoff/output/auth_scope_probe/latest_auth_scope_probe.json`
- External rewrite handoff package:
  - `docs/handoff/HANDOFF_EXTERNAL_REWRITE.md`
  - `docs/handoff/RESOURCE_MANIFEST.json`
  - `docs/handoff/RESOURCE_MANIFEST.md`
  - `docs/handoff/STATLESS_ENDPOINT_SHORTLIST.md`
  - `docs/handoff/UNRESOLVED_ENDPOINTS.md`
  - `docs/handoff/README.md`
  - `docs/handoff/RESOURCE_INDEX.md`
  - `docs/handoff/EXTERNAL_REWRITE_RUNBOOK.md`
  - `docs/handoff/NON_REPO_ARTIFACTS.md`

Label taxonomy used by `PROVIDER_FIELD_MAP.*`:
- Endpoint tier label: `critical`, `high_signal`, `mapped`, `unknown`
- Field confidence label: `confirmed`, `inferred`, `unresolved`
- Auth scope label: `public_stateless`, `public_context_required`, `auth_required`, `unknown`

Contract confidence terms used in this document:
- `verified`: directly observed in capture/source payload parsing
- `inferred`: strongly implied by source/comments/adjacent flow, but not directly captured in this run

## 1.1) Rewrite Handoff

For external provider rewrites targeting third-party functional parity:
- Primary entrypoint: `docs/handoff/HANDOFF_EXTERNAL_REWRITE.md`
- Machine-readable inventory: `docs/handoff/RESOURCE_MANIFEST.json`
- Discoverability index: `docs/handoff/RESOURCE_INDEX.md`
- Non-portable artifact pointers: `docs/handoff/NON_REPO_ARTIFACTS.md`

## 2) Security/Redaction Contract (High-Risk Only)

Always redact with `<redacted>` before storage/export:
- Password/credential fields: `password`, `pass`, `hmpgPwdCphd`, `txtPwd`
- Card/PAN fields: `stlCrCrdNo*`, `hidStlCrCrdNo*`, card-number contexts
- Card PIN-like fields: `vanPwd*`, `hidVanPwd*`
- Auth validation fields: `athnVal*`, `hidAthnVal*`
- Session cookies: `Cookie` (request), `Set-Cookie` (response), serialized cookie jars
- Auth headers/tokens: `Authorization`, `Proxy-Authorization`, `X-Auth-Token`, `X-Access-Token`, `X-Api-Key`, `Id-Token`, header names containing `token`

## 3) Common Envelope Contracts

### 3.1 SRT (app.srail.or.kr)

- Typical success/failure envelope:
  - `resultMap[0].strResult` (`SUCC`/`FAIL`)
  - `resultMap[0].msgCd`, `msgTxt`
- Data containers vary by endpoint:
  - `outDataSets.dsOutput*`
  - `trainListMap`, `payListMap`, `reservListMap`
- Alternate envelope observed on some endpoints:
  - `ERROR_CODE`, `ERROR_MSG`, `dsResultMap`

### 3.2 NetFunnel (nf.letskorail.com)

- Response is JS/plain-text style, not JSON
- Parsed contract from `srtgo/srt.py`:
  - `NetFunnel.gControl.result='<code>:<status>:key=...&nwait=...&ip=...'
- Key fields:
  - `status`: `200` pass, `201` wait, `502` already completed
  - `key`, `nwait`, `ip`

### 3.3 KTX/Korail (smart.letskorail.com)

- Typical envelope:
  - `strResult` (`SUCC`/`FAIL`)
  - on failure: `h_msg_cd`, `h_msg_txt`
- Data containers:
  - `trn_infos.trn_info`
  - `jrny_infos.jrny_info`
  - `reservation_list`

## 4) Canonical SRT Endpoint Contracts (srtgo)

All entries below use fixed schema.

### 4.1 Login

- Source: `srtgo`
- Method: `POST`
- Host: `app.srail.or.kr:443`
- Path: `/apb/selectListApb01080_n.do`
- Request contract:
  - Required: `auto`, `check`, `page`, `deviceKey`, `login_referer`, `srchDvCd`, `srchDvNm`, `hmpgPwdCphd`
  - Optional: `customerYn`
- Response contract:
  - Envelope: `commandMap`, `userMap`
  - Parsed fields used by client: `userMap.MB_CRD_NO`, `userMap.CUST_NM`, `userMap.MBL_PHONE`
- Auth/session requirements: none (entrypoint)
- Redaction-sensitive fields: `hmpgPwdCphd`, cookies
- Confidence: `verified`
- Evidence:
  - `third_party/srtgo/srtgo/srt.py` (`login`)
  - `docs/handoff/output/20260228T092205Z_70825b27/0001_srt_POST_selectListApb01080_n.do.json`

### 4.2 Logout

- Source: `srtgo`
- Method: `POST`
- Host: `app.srail.or.kr:443`
- Path: `/login/loginOut.do`
- Request contract: empty body expected
- Response contract: `commandMap`, `userMap` (session teardown context)
- Auth/session requirements: active session cookie
- Redaction-sensitive fields: cookies
- Confidence: `verified`
- Evidence:
  - `third_party/srtgo/srtgo/srt.py` (`logout`)
  - `docs/handoff/output/20260228T093914Z_1ef8280a/0001_srt_POST_loginOut.do.json`

### 4.3 Train Search

- Source: `srtgo`
- Method: `POST`
- Host: `app.srail.or.kr:443`
- Path: `/ara/selectListAra10007_n.do`
- Request contract:
  - Required (observed):
    - `chtnDvCd`, `dptDt`, `dptTm`, `dptDt1`, `dptTm1`
    - `dptRsStnCd`, `arvRsStnCd`
    - `stlbTrnClsfCd`, `trnGpCd`, `trnNo`
    - `psgNum`, `seatAttCd`, `arriveTime`
    - `dlayTnumAplFlg`, `netfunnelKey`
  - Optional/blank fields often sent: `tkDptDt`, `tkDptTm`, `tkTripChgFlg`, `tkTrnNo`
- Response contract:
  - Envelope: `resultMap[0].strResult`, `msgCd`, `msgTxt`
  - Top-level keys: `ErrorCode`, `ErrorMsg`, `commandMap`, `outDataSets`, `resultMap`, `trainListMap`
  - Primary dataset: `outDataSets.dsOutput1[]`
  - Key fields (sample):
    - Routing/time: `dptDt`, `dptTm`, `arvDt`, `arvTm`, `runDt`, `runTm`, `trnNo`
    - Station/order: `dptRsStnCd`, `arvRsStnCd`, `dptStnRunOrdr`, `arvStnRunOrdr`, `dptStnConsOrdr`, `arvStnConsOrdr`
    - Seat state: `gnrmRsvPsbStr`, `sprmRsvPsbStr`, `rsvWaitPsbCd`, `rsvWaitPsbCdNm`
    - Pricing: `rcvdAmt`, `rcvdFare`
    - Train kind: `stlbTrnClsfCd`, `trnGpCd`
- Auth/session requirements: logged-in session + valid `netfunnelKey`
- Redaction-sensitive fields: cookies, auth headers
- Confidence: `verified`
- Evidence:
  - `third_party/srtgo/srtgo/srt.py` (`search_train`)
  - `docs/handoff/output/20260228T092614Z_9155f005/0006_srt_POST_selectListAra10007_n.do.json`

### 4.4 Reserve (Personal/Standby)

- Source: `srtgo`
- Method: `POST`
- Host: `app.srail.or.kr:443`
- Path: `/arc/selectListArc05013_n.do`
- Request contract:
  - Required core: `jobId` (`1101` personal, `1102` standby), `jrnyCnt`, `jrnyTpCd`, `jrnySqno1`, `trnGpCd`, `stlbTrnClsfCd1`, `trnNo1`, `runDt1`, `netfunnelKey`
  - Station/time/order: `dptRsStnCd1`, `arvRsStnCd1`, `dptDt1`, `dptTm1`, `arvTm1`, `dptStnConsOrdr1`, `arvStnConsOrdr1`, `dptStnRunOrdr1`, `arvStnRunOrdr1`
  - Passenger block: `totPrnb`, `psgGridcnt`, `psgTpCd1..N`, `psgInfoPerPrnb1..N`
  - Seat attrs: `locSeatAttCd1`, `rqSeatAttCd1`, `dirSeatAttCd1`, `smkSeatAttCd1`, `etcSeatAttCd1`, `psrmClCd1`
  - Personal-only add-on: `reserveType=11`
- Response contract:
  - Envelope: `resultMap[0].strResult`
  - Keys: `commandMap`, `payListMap`, `reservListMap`, `resultMap`, `trainListMap`
  - Reservation handle: `reservListMap[0].pnrNo`
- Auth/session requirements: login + valid `netfunnelKey`
- Redaction-sensitive fields: cookies; downstream payment linkage fields
- Confidence: `verified`
- Evidence:
  - `third_party/srtgo/srtgo/srt.py` (`_reserve`)
  - `docs/handoff/output/20260228T092614Z_9155f005/0008_srt_POST_selectListArc05013_n.do.json`

### 4.5 Standby Option Settings

- Source: `srtgo`
- Method: `POST`
- Host: `app.srail.or.kr:443`
- Path: `/ata/selectListAta01135_n.do`
- Request contract: `pnrNo`, `psrmClChgFlg`, `smsSndFlg`, `telNo`
- Response contract: `ERROR_CODE`, `ERROR_MSG`, `dsResultMap`
- Auth/session requirements: logged-in session
- Redaction-sensitive fields: phone number (`telNo`) + cookies
- Confidence: `verified`
- Evidence:
  - `third_party/srtgo/srtgo/srt.py` (`reserve_standby_option_settings`)
  - `docs/handoff/output/20260228T092614Z_9155f005/0011_srt_POST_selectListAta01135_n.do.json`

### 4.6 Reservation List

- Source: `srtgo`
- Method: `POST`
- Host: `app.srail.or.kr:443`
- Path: `/atc/selectListAtc14016_n.do`
- Request contract: `pageNo`
- Response contract:
  - Envelope: `resultMap[0].strResult`
  - Keys: `commandMap`, `payListMap`, `resultMap`, `rsListMap`, `rsMap`, `trainListMap`
  - Client consumption:
    - reservation join uses paired `trainListMap[]` + `payListMap[]`
- Auth/session requirements: logged-in session
- Redaction-sensitive fields: cookies
- Confidence: `verified`
- Evidence:
  - `third_party/srtgo/srtgo/srt.py` (`get_reservations`)
  - `docs/handoff/output/20260228T092614Z_9155f005/0009_srt_POST_selectListAtc14016_n.do.json`

### 4.7 Ticket Detail

- Source: `srtgo`
- Method: `POST`
- Host: `app.srail.or.kr:443`
- Path: `/ard/selectListArd02019_n.do`
- Request contract: `pnrNo`, `jrnySqno`
- Response contract:
  - Envelope: `resultMap[0].strResult`
  - Keys: `commandMap`, `resultMap`, `trainListMap`, `tranferListMap`
  - `trainListMap[]` ticket row fields include seat/class/discount/pricing fields consumed by `SRTTicket`
- Auth/session requirements: logged-in session
- Redaction-sensitive fields: cookies
- Confidence: `verified`
- Evidence:
  - `third_party/srtgo/srtgo/srt.py` (`ticket_info`)
  - `docs/handoff/output/20260228T092614Z_9155f005/0010_srt_POST_selectListArd02019_n.do.json`

### 4.8 Cancel Reservation

- Source: `srtgo`
- Method: `POST`
- Host: `app.srail.or.kr:443`
- Path: `/ard/selectListArd02045_n.do`
- Request contract: `pnrNo`, `jrnyCnt`, `rsvChgTno`
- Response contract: `commandMap`, `resultMap`, `trainListMap`, with `resultMap[0].strResult`
- Auth/session requirements: logged-in session
- Redaction-sensitive fields: cookies
- Confidence: `verified`
- Evidence:
  - `third_party/srtgo/srtgo/srt.py` (`cancel`)
  - `docs/handoff/output/20260228T092916Z_17168665/0005_srt_POST_selectListArd02045_n.do.json`

### 4.9 Card Payment

- Source: `srtgo`
- Method: `POST`
- Host: `app.srail.or.kr:443`
- Path: `/ata/selectListAta09036_n.do`
- Request contract (selected):
  - Identity/reservation: `mbCrdNo`, `pnrNo`, `totPrnb`, `jrnyCnt`, `rsvChgTno`
  - Amount: `totNewStlAmt`, `mnsStlAmt1`
  - Card auth: `athnDvCd1`, `athnVal1`, `stlCrCrdNo1`, `vanPwd1`, `crdVlidTrm1`, `ismtMnthNum1`
  - Control fields: `stlDmnDt`, `stlMnsCd1`, `stlMnsSqno1`, `strJobId`, `ctlDvCd`, `cgPsId`, `pageNo`, `rowCnt`
  - Time/station context: `dptTm`, `arvTm`, `dptStnConsOrdr2`, `arvStnConsOrdr2`, `trnGpCd`
- Response contract:
  - Envelope observed: `ERROR_CODE`, `ERROR_MSG`, `dsResultMap`, `outDataSets.dsOutput0[]`
  - Success/fail key: `outDataSets.dsOutput0[0].strResult`
- Auth/session requirements: logged-in session with valid unpaid reservation
- Redaction-sensitive fields: `stlCrCrdNo1`, `vanPwd1`, `athnVal1`, cookies, token headers
- Confidence: `verified`
- Evidence:
  - `third_party/srtgo/srtgo/srt.py` (`pay_with_card`)
  - `docs/handoff/output/20260228T092916Z_17168665/0024_srt_POST_selectListAta09036_n.do.json`

### 4.10 Reserve Info (Refund Preload)

- Source: `srtgo`
- Method: `POST`
- Host: `app.srail.or.kr:443`
- Path: `/atc/getListAtc14087.do`
- Request contract:
  - Body: none in current implementation
  - Header dependence: referer must be set to `/common/ATC/ATC0201L/view.do?pnrNo=<reservation>`
- Response contract:
  - Envelope: `ErrorCode`, `ErrorMsg`, `outDataSets`
  - Primary fields from `outDataSets.dsOutput1[0]` used by refund:
    - `pnrNo`, `ogtkSaleDt`, `ogtkSaleWctNo`, `ogtkSaleSqno`, `ogtkRetPwd`, `buyPsNm`
- Auth/session requirements: logged-in session; referer-sensitive
- Redaction-sensitive fields: ticket/password-like `ogtkRetPwd`, cookies
- Confidence: `verified`
- Evidence:
  - `third_party/srtgo/srtgo/srt.py` (`reserve_info`)
  - `docs/handoff/output/20260228T093249Z_98e24e48/0008_srt_POST_getListAtc14087.do.json`

### 4.11 Refund

- Source: `srtgo`
- Method: `POST`
- Host: `app.srail.or.kr:443`
- Path: `/atc/selectListAtc02063_n.do`
- Request contract: `pnr_no`, `cnc_dmn_cont`, `saleDt`, `saleWctNo`, `saleSqno`, `tkRetPwd`, `psgNm`
- Response contract:
  - Envelope: `resultMap[0]`
  - Success key: `resultMap[0].strResult`
- Auth/session requirements: logged-in session + valid refundable ticket context
- Redaction-sensitive fields: `tkRetPwd`, cookies
- Confidence: `verified`
- Evidence:
  - `third_party/srtgo/srtgo/srt.py` (`refund`)
  - `docs/handoff/output/20260228T093249Z_98e24e48/0009_srt_POST_selectListAtc02063_n.do.json`

### 4.12 NetFunnel

- Source: `srtgo`
- Method: `GET`
- Host: `nf.letskorail.com` (plus nf sub-host variants)
- Path: `/ts.wseq`
- Request contract:
  - Common: `opcode`, `nfid`, `prefix`, `js`, `<timestamp>=`
  - Start/check: `sid=service_1`, `aid=act_10`
  - Check: `key`, `ttl=1`
  - Complete: `key`
- Response contract:
  - Text parse into `code`, `status`, `key`, `nwait`, `ip`
- Auth/session requirements: none; flow-coupled with SRT search/reserve
- Redaction-sensitive fields: `key` (treat as transient secret)
- Confidence: `verified`
- Evidence:
  - `third_party/srtgo/srtgo/srt.py` (`NetFunnelHelper`)
  - `docs/handoff/output/20260228T092916Z_17168665/0011_netfunnel_GET_ts.wseq.json`

## 5) Supplemental SRT Contract Notes (third_party/srt)

### 5.1 Confirmed overlaps with srtgo

- Uses same host and major SRT paths:
  - login/logout/search/reserve/tickets/cancel/standby_option/payment
- Same login/search core parameters and same `strResult` handling model

### 5.2 Divergent/extra behavior

- `ticket_info` endpoint path differs:
  - `srtgo`: `/ard/selectListArd02019_n.do`
  - `srt`: `/ard/selectListArd02017_n.do?`
- NetFunnel handling adds explicit retry logic on invalid key:
  - checks `message_code == NET000001` then re-issues key
- Search pagination behavior is explicitly looped in `srt` (`_search_train`) to fetch additional blocks until exhaustion/time-limit
- `srt` module in this pinned revision does not expose `reserve_info` + `refund` methods present in `srtgo`

Evidence:
- `third_party/srt/SRT/constants.py`
- `third_party/srt/SRT/srt.py`
- `third_party/srt/SRT/netfunnel.py`

## 6) KTX/Korail Supplemental Contracts (srtgo ktx + korail2)

These are supplemental to SRT canonical contracts.

### 6.1 Base host and endpoint family

- Host: `smart.letskorail.com:443`
- Base path family: `/classes/com.korail.mobile.*`
- Core endpoints (observed in both `srtgo/ktx.py` and `korail2/korail2.py`):
  - `.login.Login`
  - `.common.logout`
  - `.seatMovie.ScheduleView`
  - `.certification.TicketReservation`
  - `.reservationCancel.ReservationCancelChk`
  - `.myTicket.MyTicketList`
  - `.refunds.SelTicketInfo`
  - `.reservation.ReservationView`
  - `.certification.ReservationList`
  - `.common.code.do` (password encryption key handshake)
- Additional in `srtgo/ktx.py` modernized contract:
  - `.payment.ReservationPayment`
  - `.refunds.RefundsRequest`

### 6.2 Auth/cipher contract

- Password is encrypted via preflight key exchange:
  - POST `.common.code.do` with `code=app.login.cphd`
  - response includes `app.login.cphd.idx` and `key`
  - password encrypted AES-CBC, then base64(base64(ciphertext))
  - login payload includes `idx`, `txtPwd`

### 6.3 Representative KTX payload schema

- Login request:
  - `Device`, `Version`, `Key`, `txtMemberNo`, `txtPwd`, `txtInputFlg`, `idx`
- Search request:
  - `txtGoStart`, `txtGoEnd`, `txtGoAbrdDt`, `txtGoHour`, `selGoTrain`, `txtTrnGpCd`
  - passenger counts: `txtPsgFlg_1..5`
  - options: `srtCheckYn`, `rtYn`, `adjStnScdlOfrFlg`
- Reserve request:
  - journey keys `txtJrny*`, train keys `txtTrnNo1`, `txtRunDt1`, `txtTrnClsfCd1`, `txtTrnGpCd1`, `txtPsrmClCd1`
  - passenger block `txtPsgTpCd*`, `txtDiscKndCd*`, `txtCompaCnt*`
- Cancel request:
  - `txtPnrNo`, `txtJrnySqno`, `txtJrnyCnt`, `hidRsvChgNo`
- Card payment request (KTX):
  - `hidStlCrCrdNo1`, `hidVanPwd1`, `hidAthnVal1`, `hidCrdVlidTrm1`, `hidAthnDvCd1`

Evidence:
- `third_party/srtgo/srtgo/ktx.py`
- `third_party/korail2/korail2/korail2.py`

## 7) Web/Mobile-Web Supplemental Contracts

This section documents provider-relevant web flow contracts (non-canonical vs srtgo).

### 7.1 Static JS asset endpoints

#### A) `GET /js/stationInfo.js`

- Source: web capture (`flows`) + user curl
- Method/Host/Path: `GET app.srail.or.kr /js/stationInfo.js`
- Request contract:
  - Typical headers: app/mobile UA, referer from app pages
  - cache-buster query often used: `?_={epoch_ms}`
- Response contract:
  - JavaScript station metadata used for station code/name mappings and UI helpers
- Auth/session: fetched within session context in observed flow
- Redaction-sensitive fields: cookies if present in request
- Confidence: `verified`
- Evidence:
  - `/Users/jasonlee/Downloads/flows` (network entry for `/js/stationInfo.js`)
  - user-provided curl trace on 2026-02-28

#### B) `GET /js/common/bridge_new.js`

- Source: web capture (`flows`)
- Method/Host/Path: `GET app.srail.or.kr /js/common/bridge_new.js`
- Request contract: script fetch from page context
- Response contract (key behavior):
  - registers WebView bridge handlers (`getDataBridge`, `returnShcardSmartPay`, etc.)
  - `getDataBridge(data)` constructs and navigates to:
    - `/ara/selectListAra10007_n.do?directYn=Y&jobId=1101&...`
  - `returnShcardSmartPay(data)` constructs and navigates to:
    - `/common/ATA/ATA0204C/view.do?directYn=Y&...`
- Auth/session: webview/app bridge context dependent
- Redaction-sensitive fields: query can include user/travel context; cookies
- Confidence: `verified`
- Evidence:
  - `/Users/jasonlee/Downloads/flows` strings around `getDataBridge` and `returnShcardSmartPay`

#### C) `GET /js/common/storage_new.js`

- Source: web capture (`flows`)
- Method/Host/Path: `GET app.srail.or.kr /js/common/storage_new.js`
- Request contract: script fetch
- Response contract (key behavior):
  - storage bridge helpers (`getStorageItem`, `setStorageItem`, `removeStorageItem` callback pattern)
  - login bootstrap pattern posts to `/neo/apb/selectListApb01080_n.do` with stored credentials (`loginCheck_new.js` interaction)
- Auth/session: app storage + cookie/session interplay
- Redaction-sensitive fields: stored encoded credentials, device identifiers
- Confidence: `verified` (behavior), `inferred` (complete call graph)
- Evidence:
  - `/Users/jasonlee/Downloads/flows` lines around storage callback functions and `/neo/apb/selectListApb01080_n.do`

#### D) `GET /js/common/messages.js`

- Source: web capture (`flows`)
- Method/Host/Path: `GET app.srail.or.kr /js/common/messages.js`
- Request contract: script fetch
- Response contract:
  - message dictionary (`Sr.msgs`) used for result/error/user-notice text rendering
- Auth/session: script-level, not direct auth API
- Redaction-sensitive fields: none intrinsically (content only)
- Confidence: `verified`
- Evidence:
  - `/Users/jasonlee/Downloads/flows` entry for `/js/common/messages.js`

#### E) `GET /js/common/util.js`

- Source: web capture (`flows`) + direct fetch verification
- Method/Host/Path: `GET app.srail.or.kr /js/common/util.js`
- Request contract: script fetch from app/webview page context
- Response contract (provider-relevant behavior):
  - exposes helper `getServerTime()` which performs:
    - synchronous `POST /ara/ara0101v.do`
    - reads HTTP `Date` response header
    - normalizes to KR-time helper output via `koreaToday(...)`
  - also contains train-number formatting helpers (`getTrNoView`, `getTrNoData`)
- Auth/session: likely usable without special auth for static script fetch; inner `/ara/ara0101v.do` behavior may be session/page-context dependent
- Redaction-sensitive fields: cookies if sent by browser context
- Confidence: `verified`
- Evidence:
  - `/Users/jasonlee/Downloads/flows` entry for `/js/common/util.js`
  - `/tmp/util.js` analysis (`getServerTime` -> `POST /ara/ara0101v.do`)

#### F) `GET /js/common/push.js`

- Source: web capture (`flows`) + direct fetch verification
- Method/Host/Path: `GET app.srail.or.kr /js/common/push.js`
- Request contract: script fetch from app/webview page context
- Response contract (provider-relevant behavior):
  - defines push registration/cancel helpers for ticket lifecycle events
  - `pushReg(data, mbCrdNo, contextPath)` emits:
    - `POST {contextPath}/ata/selectListAta04A01_n.do`
    - payload fields:
      - `pnr_no`, `mb_crd_no`, `dpt_dt`, `run_dt`, `trn_no`
      - `dpt_stn_cd`, `arv_stn_cd`
      - `dpt_snd_dttm`, `arv_snd_dttm`
      - `PushContentDpt`, `PushContentArv`
  - conditional survey push:
    - `POST {contextPath}/push/surveyPush.do` with the ticket data object (plus `SIUFlg=Insert`)
  - `pushCancle(data, str, contextPath)` emits:
    - `POST {contextPath}/atc/selectListAtc02A01_n.do`
    - payload fields: `pnrNo`, `cnc_dmn_cont`
- Auth/session: session-bound ajax from authenticated app/webview context
- Redaction-sensitive fields: cookies; `mb_crd_no` (membership id) should be treated as sensitive personal identifier
- Confidence: `verified`
- Evidence:
  - `/Users/jasonlee/Downloads/flows` entry for `/js/common/push.js`
  - `/tmp/push.js` analysis (`pushReg`, `pushCancle`)

#### G) `GET /js/holidays.js`

- Source: direct fetch verification
- Method/Host/Path: `GET app.srail.or.kr /js/holidays.js`
- Request contract: script fetch from app/webview page context
- Response contract (provider-relevant behavior):
  - static/dynamic holiday policy map keyed by `MMDD`
  - holiday entry schema:
    - `type` (observed: `1` normal holiday display, `0` special transport restriction marker)
    - `title`
    - `year` (empty means all years, specific year supported)
    - `limitTime` (cutoff timestamp, `yyyyMMddHHmmss`)
  - helper `holidayCheck(day)` returns datepicker constraints/class markers:
    - `[displayFlag, cssClass, title]` with `date-holiday` / weekend classes
  - special period logic observed for 2026 설 특별대수송기간 (`type:0`) with time-window checks
- Auth/session: none required beyond static script fetch context
- Redaction-sensitive fields: none intrinsic (no user secrets in payload)
- Confidence: `verified`
- Evidence:
  - `/tmp/holidays.js` analysis (`holidays`, `holidayCheck`, `limitTime` rules)

### 7.2 Popup/ETK train info endpoints

#### A) 운행시간 (Schedule popup)

- Source: user run + user URL
- Method/Host/Path:
  - `GET etk.srail.kr /hpg/hra/01/selectTrainScheduleList.do`
  - mobile-app variant observed in curl: `POST app.srail.or.kr /ara/selectListAra12009_n.do`
- Request contract (`/ara/selectListAra12009_n.do`, observed):
  - `stnCourseNm`, `trnSort`, `runDt`, `trnNo`
- Response contract:
  - HTML fragment/table with stop-by-stop schedule:
    - station name, arrival time, departure time (and delay context when applicable)
- Auth/session: session-bound in app/webview context
- Redaction-sensitive fields: cookies
- Confidence: `verified`
- Evidence:
  - user-provided curl (2026-02-28)
  - user-provided popup URL

#### B) 운임요금 (Fare popup)

- Source: user run + user URL
- Method/Host/Path:
  - `GET etk.srail.kr /hpg/hra/01/selectTrainChargeList.do?...`
  - mobile-app variant observed in curl: `POST app.srail.or.kr /ara/selectListAra13010_n.do`
- Request contract (`/ara/selectListAra13010_n.do`, observed):
  - Core: `stnCourseNm`, `trnSort`, `runDt`, `trnNo`
  - Pricing/search context: `chtnDvCd`, `dptRsStnCd1`, `arvRsStnCd1`, `runDt1`, `trnNo1`
  - Passenger block: `psgTpCd1..6`, `psgInfoPerPrnb1..6`
  - second-leg placeholders: `dptRsStnCd2`, `arvRsStnCd2`, `runDt2`, `trnNo2`
- Response contract:
  - HTML fare/charge breakdown by class/passenger configuration
- Auth/session: session-bound in app/webview context
- Redaction-sensitive fields: cookies
- Confidence: `verified`
- Evidence:
  - user-provided curl (2026-02-28)
  - user-provided popup URL

#### C) 승차권구매이력 조회 (ATD history list)

- Source: user-provided curl + web flow artifact
- Method/Host/Path:
  - `POST app.srail.or.kr /atd/selectListAtd02039_n.do`
- Request contract (observed):
  - Required:
    - `qryNumNext` (pagination cursor/offset control; observed `0`)
    - `dptDtFrom` (`yyyyMMdd`, start date)
    - `dptDtTo` (`yyyyMMdd`, end date)
  - Example windows observed:
    - `dptDtFrom=20251128`, `dptDtTo=20260228`
    - `dptDtFrom=20260128`, `dptDtTo=20260228`
- Response contract:
  - HTML fragment/list payload (AJAX `dataType: "html"` in page script context)
  - Used to render ticket purchase-history rows in `ATD0101V` flow
- Auth/session requirements:
  - logged-in session cookie required
  - XHR context headers typically include `X-Requested-With: XMLHttpRequest`
- Redaction-sensitive fields: cookies, membership-linked identifiers in rendered HTML
- Confidence: `verified`
- Evidence:
  - user-provided curl traces (2026-02-28)
  - `/Users/jasonlee/Downloads/flows` (`url:"/atd/selectListAtd02039_n.do", dataType:"html"`)

#### D) 승차권확인 목록 페이지 (ATC14017)

- Source: user-provided curl + web flow artifact
- Method/Host/Path:
  - `GET app.srail.or.kr /atc/selectListAtc14017_n.do`
- Request contract (observed):
  - Query:
    - `pageNo` (observed `0`)
  - Navigation context:
    - typically opened from `ATC14021`/quick menu links
    - browser/app navigation (`Sec-Fetch-Mode: navigate`, `Sec-Fetch-Dest: document`)
- Response contract:
  - full HTML document payload for 승차권확인 목록 rendering
  - acts as ticket-view landing/list page in web/app UI flow
- Auth/session requirements:
  - logged-in session cookie required
- Redaction-sensitive fields:
  - cookies
  - user-identifying ticket/member data in rendered HTML
- Confidence: `verified`
- Evidence:
  - user-provided curl trace (2026-02-28): `/atc/selectListAtc14017_n.do?pageNo=0`
  - `/Users/jasonlee/Downloads/flows` entries showing direct navigation links to `/atc/selectListAtc14017_n.do?pageNo=0`

### 7.3 Additional provider-relevant web endpoints found in flow artifact

These appeared in `/Users/jasonlee/Downloads/flows` and are relevant for contract mapping:
- `/apb/selectListApb01017_n.do` (device key update ajax)
- `/ata/selectListAta04A01_n.do` (push/service registration-like ajax)
- `/atc/selectListAtc02A01_n.do` (push cancellation/unregister-side ajax)
- `/atd/selectListAtd02039_n.do` (승차권구매이력 html ajax)
- `/atc/selectListAtc02085_n.do` (refund pre-check json)
- `/atc/selectListAtc02063_n.do` (refund execute json)
- `/atc/selectListAtc14040_n.do` (ticket-related navigation/data page)
- `/atc/selectListAtc14017_n.do`, `/atc/selectListAtc14021_n.do`, `/atc/selectListAtc14022_n.do` (승차권확인 ticket views)
- `/ara/ara0101v.do` (server time sync header endpoint used by `util.js`)
- `/push/surveyPush.do` (travel survey push registration hook)
- `/js/holidays.js` (holiday/special-transport booking-calendar policy script)

Confidence: `verified` existence, `inferred` full payload schema (unless captured separately)

## 8) Divergence Matrix

| Domain | Item | srtgo Canonical | Supplemental (`srt` / `korail2` / web) | Impact |
|---|---|---|---|---|
| SRT | `ticket_info` path | `/ard/selectListArd02019_n.do` | `srt` uses `/ard/selectListArd02017_n.do?` | Path/version drift risk |
| SRT | Refund flow | includes `reserve_info` + `refund` methods | pinned `srt` module lacks explicit equivalent in `srt.py` | `srtgo` preferred for refund implementation |
| SRT | NetFunnel retry | key fetch + wait loop | `srt` adds explicit retry on `NET000001` invalid key | robustness behavior divergence |
| SRT Search | query breadth | richer search payload in `srtgo` | `srt` uses lean payload, then paginates in loop | response completeness semantics differ |
| KTX/Korail | transport style | modernized mix (`GET`/`POST`) in `srtgo/ktx.py` | older `korail2` often uses `GET` with params | request-shape drift risk |
| KTX/Korail | payment endpoints | includes `.payment.ReservationPayment`, `.refunds.RefundsRequest` | older `korail2` constants include web payment voucher URLs too | choose endpoint family explicitly |
| Web/App | bridge adapter | not part of canonical SDK | `bridge_new.js` builds SRT search and smart-pay redirect URLs | useful for web parity and hidden params |
| Web/App | popup schedule/fare | not in `srtgo` API surface | ETK popup endpoints + app mirror endpoints (`Ara12009`,`Ara13010`) | additional contracts not covered by CLI flow |

## 9) Use In Bominal

### 9.1 Immediately actionable

- Canonical SRT booking lifecycle (login/search/reserve/list/detail/cancel/payment/refund) using `srtgo` contract
- NetFunnel key negotiation (`/ts.wseq`) as required precondition for search/reserve
- Web supplemental endpoints for parity features:
  - 운행시간: `/ara/selectListAra12009_n.do` (and ETK popup variant)
  - 운임요금: `/ara/selectListAra13010_n.do` (and ETK popup variant)
- Optional KTX provider lane via `srtgo/ktx.py` contract family

### 9.2 Supporting/non-actionable (reference)

- Static JS assets (`stationInfo.js`, `bridge_new.js`, `storage_new.js`, `messages.js`) for parameter discovery and UX behavior mapping
- Menu/navigation/list pages from flow dump that are not direct provider contract endpoints

### 9.3 Statelessly Useful Endpoint Shortlist (Auth-Scope Verified)

- `public_context_required`:
  - `GET /ts.wseq` (NetFunnel context token flow)
  - `POST /ara/selectListAra10007_n.do` (requires valid NetFunnel key, not user session cookie)
- `public_stateless`:
  - `POST /ara/selectListAra12009_n.do` (운행시간 상세 HTML payload)
  - `POST /ara/selectListAra13010_n.do` (운임요금 상세 HTML payload)
  - `GET /js/stationInfo.js`
  - `GET /js/common/bridge_new.js`
  - `GET /js/common/storage_new.js`
  - `GET /js/common/messages.js`
  - `GET /js/common/util.js`
  - `GET /js/common/push.js`
  - `GET /js/holidays.js`
- `auth_required`:
  - ticket/reservation/cancel/refund/history endpoints in `PROVIDER_FIELD_MAP.*` are session-gated by response-body signals or source-code login guards

## 10) Endpoint Tier Classification

### 10.1 Tier 0 (Critical, covered by srtgo runtime contract)

- SRT core:
  - `/apb/selectListApb01080_n.do`
  - `/login/loginOut.do`
  - `/ara/selectListAra10007_n.do`
  - `/arc/selectListArc05013_n.do`
  - `/ata/selectListAta01135_n.do`
  - `/atc/selectListAtc14016_n.do`
  - `/ard/selectListArd02019_n.do`
  - `/ard/selectListArd02045_n.do`
  - `/ata/selectListAta09036_n.do`
  - `/atc/getListAtc14087.do`
  - `/atc/selectListAtc02063_n.do`
  - `/ts.wseq`
- KTX/Korail lane via `srtgo/ktx.py`:
  - `.login.Login`, `.common.logout`, `.seatMovie.ScheduleView`
  - `.certification.TicketReservation`, `.reservationCancel.ReservationCancelChk`
  - `.myTicket.MyTicketList`, `.refunds.SelTicketInfo`
  - `.reservation.ReservationView`, `.certification.ReservationList`
  - `.payment.ReservationPayment`, `.refunds.RefundsRequest`
  - `.common.code.do`

### 10.2 Tier 1 (High-signal, not srtgo-canonical but integration-useful)

- `/ara/selectListAra12009_n.do` (운행시간 상세)
- `/ara/selectListAra13010_n.do` (운임요금 상세)
- `/hpg/hra/01/selectTrainScheduleList.do` (ETK popup schedule)
- `/hpg/hra/01/selectTrainChargeList.do` (ETK popup fare)
- `/atd/selectListAtd02039_n.do` (승차권구매이력)
- `/atc/selectListAtc14017_n.do` (승차권확인 목록)
- `/atc/selectListAtc02085_n.do` (refund pre-check)
- `/ata/selectListAta04A01_n.do` (push registration)
- `/atc/selectListAtc02A01_n.do` (push cancellation)
- `/apb/selectListApb01017_n.do` (device key update)

### 10.3 Tier 2 (High-signal support assets/policy scripts)

- `/js/stationInfo.js`
- `/js/common/bridge_new.js`
- `/js/common/storage_new.js`
- `/js/common/messages.js`
- `/js/common/util.js` (includes `/ara/ara0101v.do` server-time sync call)
- `/js/common/push.js`
- `/js/holidays.js`

### 10.4 Tier 3 (Endpoint signals mapped, lower priority)

- `/atc/selectListAtc14021_n.do` (승차권확인 하위 화면, mapped by routing context)
- `/atc/selectListAtc14022_n.do` (승차권확인 하위 화면, mapped by routing context)
- `/atc/selectListAtc14040_n.do` (ticket-related page transition target, mapped by changePage call)
- `/atc/selectListAtc14087_n.do` (ticket confirm/issuance-related view, observed with `tConfirm` query variants)
- `/ara/selectListAra2701V.do` (정기/회수승차권 발권 계열 화면)
- `/ara/selectListAra2702V.do` (정기/회수승차권 발권 계열 화면)
- `/ara/selectListAra2700V.do` (정기/회수승차권 연계 화면, observed with `ticketType` query)
- `/common/ATA/ATA0204C/view.do` (smart-pay bridge landing target from `returnShcardSmartPay`)
- `/neo/apb/selectListApb01080_n.do` (auto-login submit target from login-check flow)
- `/login/loginOutFido.do` (FIDO logout variant endpoint)

### 10.5 Tier 4 (Endpoint signals observed but unresolved)

- `/apa/selectListApa03020_n.do`
- `/apc/selectListApc04034_n.do`
- `/apc/selectAPC0109C1.do`
- `/apc/selectAPC0109C2.do`
- `/apc/selectApc10A01_n.do`
- `/sns/snsInfoSelect.do`
- `/common/AAA/AAA0101L/view.do`
- `/common/AAA/AAA0102P/view.do`
- `/common/APD/APD0101L/view.do`
- `/common/APC/APC0106C/view.do`
- `/common/APA/APA0101P/view.do`
- `/common/APA/APA0102L/view.do`
- `/common/APA/APA0103L/view.do`
- `/common/APA/APA0103P/view.do`
- `/common/APA/APA0104P/view.do`
- `/common/APA/APA0105L/view.do`
- `/common/APA/APA0106L/view.do`
- `/common/APA/APA0201L/view.do`
- `/common/APA/APA0202L/view.do`
- `/common/APA/APA0303P/view.do`

## 11) Coverage Snapshot (Current Capture Set)

From `docs/handoff/output` runtime capture set:
- Event files parsed: `81`
- Distinct endpoint paths captured: `12`
- Highest-frequency endpoints:
  - `/apb/selectListApb01080_n.do` (19)
  - `/ara/selectListAra10007_n.do` (19)
  - `/ard/selectListArd02019_n.do` (11)
  - `/atc/selectListAtc14016_n.do` (10)
  - `/ts.wseq` (10)

This is sufficient to confirm end-to-end login/search/reserve/cancel/payment/refund contracts for SRT canonical flows.
