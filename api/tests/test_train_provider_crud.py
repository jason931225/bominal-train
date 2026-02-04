from __future__ import annotations

import json

import pytest

from app.modules.train.providers.ktx_client import KTXClient
from app.modules.train.providers.srt_client import SRTClient
from app.modules.train.providers.transport import TransportResponse


class _QueueTransport:
    def __init__(self, payloads: list[dict], *, status_code: int = 200) -> None:
        self._payloads = list(payloads)
        self._status_code = status_code
        self.requests: list[dict] = []

    async def request(self, **kwargs) -> TransportResponse:
        self.requests.append(kwargs)
        if not self._payloads:
            raise AssertionError("No queued payload available for transport request")
        payload = self._payloads.pop(0)
        return TransportResponse(
            status_code=self._status_code,
            text=json.dumps(payload),
            headers={},
        )


@pytest.mark.asyncio
async def test_srt_get_reservations_parses_ticket_info():
    transport = _QueueTransport(
        [
            {
                "userMap": {
                    "MB_CRD_NO": "1234567890",
                    "CUST_NM": "Tester",
                    "MBL_PHONE": "01012341234",
                }
            },
            {
                "resultMap": [{"strResult": "SUCC", "msgTxt": ""}],
                "trainListMap": [{"pnrNo": "PNR1001", "rcvdAmt": "55000", "tkSpecNum": "1"}],
                "payListMap": [
                    {
                        "stlFlg": "N",
                        "stlbTrnClsfCd": "17",
                        "trnNo": "381",
                        "dptDt": "20260226",
                        "dptTm": "120400",
                        "dptRsStnCd": "0551",
                        "arvTm": "151800",
                        "arvRsStnCd": "0059",
                        "iseLmtDt": "20260226",
                        "iseLmtTm": "235900",
                    }
                ],
            },
            {
                "resultMap": [{"strResult": "SUCC", "msgTxt": ""}],
                "trainListMap": [
                    {
                        "scarNo": "3",
                        "seatNo": "7A",
                        "psrmClCd": "1",
                        "dcntKndCd": "000",
                        "rcvdAmt": "55000",
                        "stdrPrc": "55000",
                        "dcntPrc": "0",
                    }
                ],
            },
        ]
    )
    client = SRTClient(transport=transport)

    login = await client.login(
        user_id="u1",
        credentials={"username": "mock-user", "password": "mock-pass"},
    )
    assert login.ok is True

    reservations = await client.get_reservations(user_id="u1")
    assert reservations.ok is True
    rows = reservations.data["reservations"]
    assert len(rows) == 1
    assert rows[0]["reservation_id"] == "PNR1001"
    assert rows[0]["dep"] == "수서"
    assert rows[0]["arr"] == "마산"
    assert rows[0]["tickets"][0]["seat_no"] == "7A"


@pytest.mark.asyncio
async def test_ktx_cancel_uses_reservation_lookup_when_metadata_missing():
    transport = _QueueTransport(
        [
            {"strResult": "SUCC", "app.login.cphd": {"idx": "1", "key": "1234567890123456"}},
            {
                "strResult": "SUCC",
                "strMbCrdNo": "1234567890",
                "strCustNm": "Tester",
                "strCpNo": "01012341234",
                "strEmailAdr": "tester@example.com",
            },
            {
                "strResult": "SUCC",
                "jrny_infos": {
                    "jrny_info": [
                        {
                            "train_infos": {
                                "train_info": [
                                    {
                                        "h_pnr_no": "PNR2002",
                                        "h_tot_seat_cnt": "1",
                                        "h_rsv_amt": "48000",
                                        "h_run_dt": "20260226",
                                        "h_dpt_tm": "120400",
                                        "h_arv_tm": "151800",
                                        "h_dpt_rs_stn_nm": "수서",
                                        "h_arv_rs_stn_nm": "마산",
                                        "h_trn_no": "381",
                                        "txtJrnySqno": "001",
                                        "txtJrnyCnt": "01",
                                        "hidRsvChgNo": "00000",
                                        "h_ntisu_lmt_dt": "20260226",
                                        "h_ntisu_lmt_tm": "235900",
                                    }
                                ]
                            }
                        }
                    ]
                },
            },
            {
                "strResult": "SUCC",
                "h_wct_no": "WCTNO1",
                "jrny_infos": {
                    "jrny_info": [
                        {
                            "seat_infos": {
                                "seat_info": [
                                    {
                                        "h_srcar_no": "3",
                                        "h_seat_no": "7A",
                                        "h_psrm_cl_nm": "일반실",
                                        "h_psg_tp_dv_nm": "어른/청소년",
                                        "h_rcvd_amt": "48000",
                                        "h_seat_prc": "48000",
                                        "h_dcnt_amt": "0",
                                    }
                                ]
                            }
                        }
                    ]
                },
            },
            {"strResult": "SUCC"},
        ]
    )

    client = KTXClient(transport=transport)

    login = await client.login(
        user_id="u1",
        credentials={"username": "mock-user", "password": "mock-pass"},
    )
    assert login.ok is True

    cancelled = await client.cancel(
        artifact_data={"reservation_id": "PNR2002"},
        user_id="u1",
    )
    assert cancelled.ok is True
    cancel_requests = [req for req in transport.requests if req.get("url", "").endswith("reservationCancel.ReservationCancelChk")]
    assert cancel_requests, "Expected one cancel request"
    cancel_payload = cancel_requests[0]["data"]
    assert cancel_payload["txtPnrNo"] == "PNR2002"
    assert cancel_payload["txtJrnySqno"] == "001"
    assert cancel_payload["txtJrnyCnt"] == "01"


@pytest.mark.asyncio
async def test_srt_pay_uses_saved_card_payload():
    transport = _QueueTransport(
        [
            {
                "userMap": {
                    "MB_CRD_NO": "1234567890",
                    "CUST_NM": "Tester",
                    "MBL_PHONE": "01012341234",
                }
            },
            {
                "resultMap": [{"strResult": "SUCC", "msgTxt": ""}],
                "trainListMap": [{"pnrNo": "PNR3003", "rcvdAmt": "55000", "tkSpecNum": "1"}],
                "payListMap": [
                    {
                        "stlFlg": "N",
                        "stlbTrnClsfCd": "17",
                        "trnNo": "381",
                        "dptDt": "20260226",
                        "dptTm": "120400",
                        "dptRsStnCd": "0551",
                        "arvTm": "151800",
                        "arvRsStnCd": "0059",
                        "iseLmtDt": "20260226",
                        "iseLmtTm": "235900",
                    }
                ],
            },
            {
                "resultMap": [{"strResult": "SUCC", "msgTxt": ""}],
                "trainListMap": [
                    {
                        "scarNo": "3",
                        "seatNo": "7A",
                        "psrmClCd": "1",
                        "dcntKndCd": "000",
                        "rcvdAmt": "55000",
                        "stdrPrc": "55000",
                        "dcntPrc": "0",
                    }
                ],
            },
            {
                "outDataSets": {
                    "dsOutput0": [{"strResult": "SUCC", "msgTxt": ""}],
                }
            },
        ]
    )
    client = SRTClient(transport=transport)
    login = await client.login(user_id="u1", credentials={"username": "mock-user", "password": "mock-pass"})
    assert login.ok is True

    paid = await client.pay(
        reservation_id="PNR3003",
        user_id="u1",
        payment_card={
            "card_number": "1234567890123456",
            "card_password": "12",
            "validation_number": "900101",
            "card_expire": "2911",
            "card_type": "J",
            "installment": 0,
        },
    )
    assert paid.ok is True
    payment_requests = [req for req in transport.requests if req.get("url", "").endswith("selectListAta09036_n.do")]
    assert payment_requests, "Expected one SRT payment request"
    payment_payload = payment_requests[0]["data"]
    assert payment_payload["pnrNo"] == "PNR3003"
    assert payment_payload["stlCrCrdNo1"] == "1234567890123456"
    assert payment_payload["vanPwd1"] == "12"
    assert payment_payload["athnVal1"] == "900101"
    assert payment_payload["crdVlidTrm1"] == "2911"


@pytest.mark.asyncio
async def test_ktx_pay_uses_saved_card_payload():
    transport = _QueueTransport(
        [
            {"strResult": "SUCC", "app.login.cphd": {"idx": "1", "key": "1234567890123456"}},
            {
                "strResult": "SUCC",
                "strMbCrdNo": "1234567890",
                "strCustNm": "Tester",
                "strCpNo": "01012341234",
                "strEmailAdr": "tester@example.com",
            },
            {
                "strResult": "SUCC",
                "jrny_infos": {
                    "jrny_info": [
                        {
                            "train_infos": {
                                "train_info": [
                                    {
                                        "h_pnr_no": "PNR4004",
                                        "h_tot_seat_cnt": "1",
                                        "h_rsv_amt": "48000",
                                        "h_run_dt": "20260226",
                                        "h_dpt_tm": "120400",
                                        "h_arv_tm": "151800",
                                        "h_dpt_rs_stn_nm": "수서",
                                        "h_arv_rs_stn_nm": "마산",
                                        "h_trn_no": "381",
                                        "txtJrnySqno": "001",
                                        "txtJrnyCnt": "01",
                                        "hidRsvChgNo": "00000",
                                        "h_ntisu_lmt_dt": "20260226",
                                        "h_ntisu_lmt_tm": "235900",
                                    }
                                ]
                            }
                        }
                    ]
                },
            },
            {
                "strResult": "SUCC",
                "h_wct_no": "WCTNO1",
                "jrny_infos": {
                    "jrny_info": [
                        {
                            "seat_infos": {
                                "seat_info": [
                                    {
                                        "h_srcar_no": "3",
                                        "h_seat_no": "7A",
                                        "h_psrm_cl_nm": "일반실",
                                        "h_psg_tp_dv_nm": "어른/청소년",
                                        "h_rcvd_amt": "48000",
                                        "h_seat_prc": "48000",
                                        "h_dcnt_amt": "0",
                                    }
                                ]
                            }
                        }
                    ]
                },
            },
            {"strResult": "SUCC"},
        ]
    )
    client = KTXClient(transport=transport)
    login = await client.login(user_id="u1", credentials={"username": "mock-user", "password": "mock-pass"})
    assert login.ok is True

    paid = await client.pay(
        reservation_id="PNR4004",
        user_id="u1",
        payment_card={
            "card_number": "1234567890123456",
            "card_password": "12",
            "validation_number": "900101",
            "card_expire": "2911",
            "card_type": "J",
            "installment": 0,
        },
    )
    assert paid.ok is True
    payment_requests = [req for req in transport.requests if req.get("url", "").endswith("payment.ReservationPayment")]
    assert payment_requests, "Expected one KTX payment request"
    payment_payload = payment_requests[0]["data"]
    assert payment_payload["hidPnrNo"] == "PNR4004"
    assert payment_payload["hidWctNo"] == "WCTNO1"
    assert payment_payload["hidStlCrCrdNo1"] == "1234567890123456"
    assert payment_payload["hidVanPwd1"] == "12"
    assert payment_payload["hidAthnVal1"] == "900101"
    assert payment_payload["hidCrdVlidTrm1"] == "2911"


@pytest.mark.asyncio
async def test_srt_cancel_paid_uses_refund_flow():
    transport = _QueueTransport(
        [
            {
                "userMap": {
                    "MB_CRD_NO": "1234567890",
                    "CUST_NM": "Tester",
                    "MBL_PHONE": "01012341234",
                }
            },
            {
                "ErrorCode": "0",
                "ErrorMsg": "",
                "outDataSets": {
                    "dsOutput1": [
                        {
                            "pnrNo": "PNR5005",
                            "ogtkSaleDt": "20260203",
                            "ogtkSaleWctNo": "WCTNO55",
                            "ogtkSaleSqno": "12345",
                            "ogtkRetPwd": "4321",
                            "buyPsNm": "Tester",
                        }
                    ]
                },
            },
            {"resultMap": [{"strResult": "SUCC", "msgTxt": ""}]},
        ]
    )
    client = SRTClient(transport=transport)
    login = await client.login(user_id="u1", credentials={"username": "mock-user", "password": "mock-pass"})
    assert login.ok is True

    cancelled = await client.cancel(
        artifact_data={"reservation_id": "PNR5005", "paid": True, "status": "paid"},
        user_id="u1",
    )
    assert cancelled.ok is True
    reserve_info_calls = [req for req in transport.requests if req.get("url", "").endswith("getListAtc14087.do")]
    assert reserve_info_calls, "Expected reserve_info request for paid SRT cancellation"
    refund_calls = [req for req in transport.requests if req.get("url", "").endswith("selectListAtc02063_n.do")]
    assert refund_calls, "Expected refund request for paid SRT cancellation"
    refund_payload = refund_calls[0]["data"]
    assert refund_payload["pnr_no"] == "PNR5005"
    assert refund_payload["saleWctNo"] == "WCTNO55"


@pytest.mark.asyncio
async def test_ktx_cancel_paid_uses_refund_flow():
    transport = _QueueTransport(
        [
            {"strResult": "SUCC", "app.login.cphd": {"idx": "1", "key": "1234567890123456"}},
            {
                "strResult": "SUCC",
                "strMbCrdNo": "1234567890",
                "strCustNm": "Tester",
                "strCpNo": "01012341234",
                "strEmailAdr": "tester@example.com",
            },
            {
                "strResult": "SUCC",
                "reservation_list": [
                    {
                        "ticket_list": [
                            {
                                "train_info": [
                                    {
                                        "h_pnr_no": "PNR6006",
                                        "h_orgtk_wct_no": "WCT66",
                                        "h_orgtk_ret_sale_dt": "20260203",
                                        "h_orgtk_sale_sqno": "9988",
                                        "h_orgtk_ret_pwd": "3344",
                                        "h_trn_no": "381",
                                    }
                                ]
                            }
                        ]
                    }
                ],
            },
            {"strResult": "SUCC"},
        ]
    )
    client = KTXClient(transport=transport)
    login = await client.login(user_id="u1", credentials={"username": "mock-user", "password": "mock-pass"})
    assert login.ok is True

    cancelled = await client.cancel(
        artifact_data={"reservation_id": "PNR6006", "paid": True, "status": "paid"},
        user_id="u1",
    )
    assert cancelled.ok is True
    refund_calls = [req for req in transport.requests if req.get("url", "").endswith("refunds.RefundsRequest")]
    assert refund_calls, "Expected refund request for paid KTX cancellation"
    refund_payload = refund_calls[0]["data"]
    assert refund_payload["txtPrnNo"] == "PNR6006"
    assert refund_payload["h_orgtk_sale_wct_no"] == "WCT66"
