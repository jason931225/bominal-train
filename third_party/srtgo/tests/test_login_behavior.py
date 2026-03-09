import io
import json
import unittest
from contextlib import redirect_stdout
from unittest import mock

try:
    from curl_cffi.requests.exceptions import Timeout as RequestTimeout
except ImportError:
    from requests.exceptions import Timeout as RequestTimeout

import srtgo.srt as srt_module
import srtgo.srtgo as srtgo_module


class LoginFlowTests(unittest.TestCase):
    def test_login_reuses_authenticated_client_from_set_login(self):
        authenticated_rail = object()

        with mock.patch.object(
            srtgo_module.keyring, "get_password", side_effect=[None, None]
        ), mock.patch.object(
            srtgo_module, "set_login", return_value=authenticated_rail
        ) as set_login_mock, mock.patch.object(
            srtgo_module, "SRT"
        ) as srt_ctor:
            result = srtgo_module.login("SRT")

        self.assertIs(result, authenticated_rail)
        set_login_mock.assert_called_once_with("SRT", debug=False)
        srt_ctor.assert_not_called()

    def test_reserve_exits_cleanly_when_curl_request_is_interrupted(self):
        rail = mock.Mock()
        train = mock.Mock()
        rail.search_train.side_effect = [
            [train],
            Exception(
                "Failed to perform, curl: (23) Failure writing output to destination, passed 7300 returned 0."
            ),
        ]

        prompt_answers = [
            {
                "departure": "수서",
                "arrival": "동대구",
                "date": "20991231",
                "time": "120000",
                "adult": 1,
            },
            {"trains": [0]},
            {"type": srtgo_module.SeatType.GENERAL_FIRST, "pay": False},
        ]

        def fake_get_password(_service, _account, default=None):
            return default

        with mock.patch.object(srtgo_module, "login", return_value=rail), mock.patch.object(
            srtgo_module.keyring, "get_password", side_effect=fake_get_password
        ), mock.patch.object(
            srtgo_module, "get_station", return_value=(["수서", "동대구"], ["수서", "동대구"])
        ), mock.patch.object(
            srtgo_module, "get_options", return_value=[]
        ), mock.patch.object(
            srtgo_module.inquirer, "prompt", side_effect=prompt_answers
        ), mock.patch.object(
            srtgo_module.keyring, "set_password"
        ), mock.patch.object(
            srtgo_module, "_sleep"
        ), mock.patch.object(
            srtgo_module, "_handle_error", return_value=False
        ) as handle_error:
            output = io.StringIO()
            with redirect_stdout(output):
                srtgo_module.reserve("SRT")

        handle_error.assert_not_called()
        self.assertIn("예매를 취소했습니다", output.getvalue())

    def test_reserve_retries_on_transient_timeout_without_prompting(self):
        rail = mock.Mock()
        train = mock.Mock()
        train.seat_available.return_value = False
        train.reserve_standby_available.return_value = False
        rail.search_train.side_effect = [
            [train],
            RequestTimeout(
                "Failed to perform, curl: (28) Operation timed out after 30002 milliseconds with 19610 bytes received."
            ),
            KeyboardInterrupt(),
        ]

        prompt_answers = [
            {
                "departure": "수서",
                "arrival": "동대구",
                "date": "20991231",
                "time": "120000",
                "adult": 1,
            },
            {"trains": [0]},
            {"type": srtgo_module.SeatType.GENERAL_FIRST, "pay": False},
        ]

        def fake_get_password(_service, _account, default=None):
            return default

        with mock.patch.object(srtgo_module, "login", return_value=rail) as login_mock, mock.patch.object(
            srtgo_module.keyring, "get_password", side_effect=fake_get_password
        ), mock.patch.object(
            srtgo_module, "get_station", return_value=(["수서", "동대구"], ["수서", "동대구"])
        ), mock.patch.object(
            srtgo_module, "get_options", return_value=[]
        ), mock.patch.object(
            srtgo_module.inquirer, "prompt", side_effect=prompt_answers
        ), mock.patch.object(
            srtgo_module.keyring, "set_password"
        ), mock.patch.object(
            srtgo_module, "_sleep"
        ) as sleep_mock, mock.patch.object(
            srtgo_module, "_handle_error", return_value=False
        ) as handle_error:
            output = io.StringIO()
            with redirect_stdout(output):
                srtgo_module.reserve("SRT")

        self.assertEqual(login_mock.call_count, 1)
        handle_error.assert_not_called()
        sleep_mock.assert_called_once()
        self.assertIn("네트워크 연결이 불안정합니다. 재시도합니다.", output.getvalue())

    def test_reserve_reauthenticates_after_three_transient_timeouts_then_prompts(self):
        rail = mock.Mock()
        reauthed_rail = mock.Mock()
        train = mock.Mock()
        train.seat_available.return_value = False
        train.reserve_standby_available.return_value = False
        rail.search_train.side_effect = [
            [train],
            RequestTimeout("timeout-1"),
            RequestTimeout("timeout-2"),
            RequestTimeout("timeout-3"),
            RequestTimeout("timeout-4"),
        ]

        prompt_answers = [
            {
                "departure": "수서",
                "arrival": "동대구",
                "date": "20991231",
                "time": "120000",
                "adult": 1,
            },
            {"trains": [0]},
            {"type": srtgo_module.SeatType.GENERAL_FIRST, "pay": False},
        ]

        def fake_get_password(_service, _account, default=None):
            return default

        with mock.patch.object(
            srtgo_module, "login", side_effect=[rail, reauthed_rail]
        ) as login_mock, mock.patch.object(
            srtgo_module.keyring, "get_password", side_effect=fake_get_password
        ), mock.patch.object(
            srtgo_module, "get_station", return_value=(["수서", "동대구"], ["수서", "동대구"])
        ), mock.patch.object(
            srtgo_module, "get_options", return_value=[]
        ), mock.patch.object(
            srtgo_module.inquirer, "prompt", side_effect=prompt_answers
        ), mock.patch.object(
            srtgo_module.keyring, "set_password"
        ), mock.patch.object(
            srtgo_module, "_sleep"
        ) as sleep_mock, mock.patch.object(
            srtgo_module, "_handle_error", return_value=False
        ) as handle_error:
            output = io.StringIO()
            with redirect_stdout(output):
                srtgo_module.reserve("SRT")

        self.assertEqual(login_mock.call_count, 2)
        handle_error.assert_called_once()
        self.assertEqual(sleep_mock.call_count, 3)
        self.assertEqual(output.getvalue().count("네트워크 연결이 불안정합니다. 재시도합니다."), 3)


class SRTLoginNormalizationTests(unittest.TestCase):
    def test_login_accepts_korean_mobile_number_with_or_without_hyphens(self):
        response = mock.Mock(
            text=json.dumps(
                {
                    "userMap": {
                        "MB_CRD_NO": "2385871503",
                        "CUST_NM": "이현준",
                        "MBL_PHONE": "01086868053",
                    }
                }
            )
        )

        for login_id in ("010-8686-8053", "01086868053"):
            with self.subTest(login_id=login_id):
                client = srt_module.SRT(login_id, "pw", auto_login=False)
                client._session = mock.Mock()
                client._session.post.return_value = response

                with redirect_stdout(io.StringIO()):
                    client.login()

                request_data = client._session.post.call_args.kwargs["data"]
                self.assertEqual(request_data["srchDvCd"], "3")
                self.assertEqual(request_data["srchDvNm"], "01086868053")


class ErrorPromptTests(unittest.TestCase):
    def test_handle_error_uses_polite_continue_prompt(self):
        async_mock = mock.AsyncMock()

        with mock.patch.object(
            srtgo_module, "get_telegram", return_value=async_mock
        ), mock.patch.object(
            srtgo_module.inquirer, "confirm", return_value=True
        ) as confirm_mock:
            with redirect_stdout(io.StringIO()):
                result = srtgo_module._handle_error(Exception("boom"))

        self.assertTrue(result)
        self.assertEqual(
            confirm_mock.call_args.kwargs["message"], "계속하시겠습니까?"
        )


if __name__ == "__main__":
    unittest.main()
