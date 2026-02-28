# Unresolved Endpoints Backlog

Endpoints here are observed but not yet safely classifiable for auth scope and/or semantics.

- Total unresolved (`auth_scope_label=unknown`): `186`
- Prioritized subset in this file (`critical` + `high_signal` + `mapped`): `41`

## Prioritized unresolved endpoints

| tier | method | host | path | reason unresolved | next probe action |
|---|---|---|---|---|---|
| critical | POST | app.srail.or.kr | /apb/selectListApb01080_n.do | insufficient response semantics in current probe/capture set | capture with controlled request context and compare login vs no-login body markers |
| critical | GET | nf5.letskorail.com | /ts.wseq | insufficient response semantics in current probe/capture set | probe each nf host variant with/without fresh key to map behavior parity |
| critical | GET | nf6.letskorail.com | /ts.wseq | insufficient response semantics in current probe/capture set | probe each nf host variant with/without fresh key to map behavior parity |
| critical | GET | nf7.letskorail.com | /ts.wseq | insufficient response semantics in current probe/capture set | probe each nf host variant with/without fresh key to map behavior parity |
| critical | GET | nf9.letskorail.com | /ts.wseq | insufficient response semantics in current probe/capture set | probe each nf host variant with/without fresh key to map behavior parity |
| critical | GET? | smart.letskorail.com | .certification.ReservationList | insufficient response semantics in current probe/capture set | capture with controlled request context and compare login vs no-login body markers |
| critical | GET? | smart.letskorail.com | .certification.TicketReservation | insufficient response semantics in current probe/capture set | capture with controlled request context and compare login vs no-login body markers |
| critical | GET? | smart.letskorail.com | .common.code.do | insufficient response semantics in current probe/capture set | capture with controlled request context and compare login vs no-login body markers |
| critical | GET? | smart.letskorail.com | .common.logout | insufficient response semantics in current probe/capture set | capture with controlled request context and compare login vs no-login body markers |
| critical | GET? | smart.letskorail.com | .login.Login | insufficient response semantics in current probe/capture set | capture with controlled request context and compare login vs no-login body markers |
| critical | GET? | smart.letskorail.com | .myTicket.MyTicketList | insufficient response semantics in current probe/capture set | capture with controlled request context and compare login vs no-login body markers |
| critical | GET? | smart.letskorail.com | .payment.ReservationPayment | insufficient response semantics in current probe/capture set | capture with controlled request context and compare login vs no-login body markers |
| critical | GET? | smart.letskorail.com | .refunds.RefundsRequest | insufficient response semantics in current probe/capture set | capture with controlled request context and compare login vs no-login body markers |
| critical | GET? | smart.letskorail.com | .refunds.SelTicketInfo | insufficient response semantics in current probe/capture set | capture with controlled request context and compare login vs no-login body markers |
| critical | GET? | smart.letskorail.com | .reservation.ReservationView | insufficient response semantics in current probe/capture set | capture with controlled request context and compare login vs no-login body markers |
| critical | GET? | smart.letskorail.com | .reservationCancel.ReservationCancelChk | insufficient response semantics in current probe/capture set | capture with controlled request context and compare login vs no-login body markers |
| critical | GET? | smart.letskorail.com | .seatMovie.ScheduleView | insufficient response semantics in current probe/capture set | capture with controlled request context and compare login vs no-login body markers |
| high_signal | GET? | app.srail.or.kr | /apb/selectListApb01017_n.do | insufficient response semantics in current probe/capture set | capture with controlled request context and compare login vs no-login body markers |
| high_signal | GET? | app.srail.or.kr | /ara/ara0101v.do | insufficient response semantics in current probe/capture set | capture with controlled request context and compare login vs no-login body markers |
| high_signal | POST | app.srail.or.kr | /ata/selectListAta04A01_n.do | insufficient response semantics in current probe/capture set | capture with controlled request context and compare login vs no-login body markers |
| high_signal | POST | app.srail.or.kr | /atc/selectListAtc02A01_n.do | insufficient response semantics in current probe/capture set | capture with controlled request context and compare login vs no-login body markers |
| high_signal | GET? | etk.srail.kr | /hpg/hra/01/selectTrainChargeList.do | insufficient response semantics in current probe/capture set | capture popup flow network with parameters and correlate rendered table rows |
| high_signal | GET? | etk.srail.kr | /hpg/hra/01/selectTrainScheduleList.do | insufficient response semantics in current probe/capture set | capture popup flow network with parameters and correlate rendered table rows |
| mapped | GET? | app.srail.or.kr | /ara/selectListAra2700V.do | insufficient response semantics in current probe/capture set | capture with controlled request context and compare login vs no-login body markers |
| mapped | GET? | app.srail.or.kr | /ara/selectListAra2701V.do | insufficient response semantics in current probe/capture set | capture with controlled request context and compare login vs no-login body markers |
| mapped | GET? | app.srail.or.kr | /ara/selectListAra2702V.do | insufficient response semantics in current probe/capture set | capture with controlled request context and compare login vs no-login body markers |
| mapped | GET? | app.srail.or.kr | /atc/selectListAtc14021_n.do | insufficient response semantics in current probe/capture set | capture with controlled request context and compare login vs no-login body markers |
| mapped | GET? | app.srail.or.kr | /atc/selectListAtc14022_n.do | insufficient response semantics in current probe/capture set | capture with controlled request context and compare login vs no-login body markers |
| mapped | GET? | app.srail.or.kr | /atc/selectListAtc14040_n.do | insufficient response semantics in current probe/capture set | capture with controlled request context and compare login vs no-login body markers |
| mapped | GET? | app.srail.or.kr | /atc/selectListAtc14087_n.do | insufficient response semantics in current probe/capture set | capture with controlled request context and compare login vs no-login body markers |
| mapped | GET? | app.srail.or.kr | /common/ATA/ATA0204C/view.do | insufficient response semantics in current probe/capture set | capture with controlled request context and compare login vs no-login body markers |
| mapped | GET? | app.srail.or.kr | /login/loginOutFido.do | insufficient response semantics in current probe/capture set | capture with controlled request context and compare login vs no-login body markers |
| mapped | GET? | app.srail.or.kr | /neo/apb/selectListApb01080_n.do | insufficient response semantics in current probe/capture set | capture with controlled request context and compare login vs no-login body markers |
| mapped | GET | smart.letskorail.com | /MB_CRD_NO | insufficient response semantics in current probe/capture set | capture with controlled request context and compare login vs no-login body markers |
| mapped | GET | smart.letskorail.com | /app.login.cphd | insufficient response semantics in current probe/capture set | capture with controlled request context and compare login vs no-login body markers |
| mapped | GET | smart.letskorail.com | /dsOutput0 | insufficient response semantics in current probe/capture set | capture with controlled request context and compare login vs no-login body markers |
| mapped | GET | smart.letskorail.com | /msgTxt | insufficient response semantics in current probe/capture set | capture with controlled request context and compare login vs no-login body markers |
| mapped | GET | smart.letskorail.com | /outDataSets | insufficient response semantics in current probe/capture set | capture with controlled request context and compare login vs no-login body markers |
| mapped | GET | smart.letskorail.com | /strMbCrdNo | insufficient response semantics in current probe/capture set | capture with controlled request context and compare login vs no-login body markers |
| mapped | GET | smart.letskorail.com | /strResult | insufficient response semantics in current probe/capture set | capture with controlled request context and compare login vs no-login body markers |
| mapped | GET | smart.letskorail.com | /userMap | insufficient response semantics in current probe/capture set | capture with controlled request context and compare login vs no-login body markers |

## Deferred unresolved tail

- Remaining unresolved endpoints are mostly document/view/navigation routes with low direct provider contract value.
- Full tail remains in `docs/handoff/srtgo-capture/PROVIDER_FIELD_MAP.json` under `auth_scope_label=unknown`.
