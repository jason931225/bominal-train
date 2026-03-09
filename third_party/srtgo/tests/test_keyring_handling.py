import io
import unittest
from contextlib import redirect_stdout
from unittest import mock

import keyring.errors

import srtgo.srtgo as srtgo_module


class KeyringHandlingTests(unittest.TestCase):
    def test_keyring_get_returns_default_when_backend_errors(self):
        with mock.patch.object(
            srtgo_module.keyring,
            "get_password",
            side_effect=keyring.errors.KeyringError("boom"),
        ):
            output = io.StringIO()
            with redirect_stdout(output):
                result = srtgo_module._keyring_get("SRT", "id", default="fallback")

        self.assertEqual(result, "fallback")
        self.assertIn("키체인 조회 중 오류가 발생했습니다", output.getvalue())

    def test_keyring_set_reports_invalid_owner_edit(self):
        error = keyring.errors.PasswordSetError(
            "Can't store password on keychain: (-25244, 'Unknown Error')"
        )

        with mock.patch.object(srtgo_module.keyring, "set_password", side_effect=error):
            output = io.StringIO()
            with redirect_stdout(output):
                result = srtgo_module._keyring_set("SRT", "id", "user")

        self.assertFalse(result)
        self.assertIn("-25244", output.getvalue())
        self.assertIn("소유자/접근 권한", output.getvalue())

    def test_keyring_set_many_rolls_back_partial_writes(self):
        calls = []

        def fake_set_password(service, account, value):
            calls.append(("set", service, account, value))
            if account == "pass":
                raise keyring.errors.PasswordSetError("boom")

        def fake_delete_password(service, account):
            calls.append(("delete", service, account))

        with mock.patch.object(
            srtgo_module.keyring, "set_password", side_effect=fake_set_password
        ), mock.patch.object(
            srtgo_module.keyring, "delete_password", side_effect=fake_delete_password
        ):
            output = io.StringIO()
            with redirect_stdout(output):
                result = srtgo_module._keyring_set_many(
                    "SRT", {"id": "user", "pass": "pw"}, ok_key="ok"
                )

        self.assertFalse(result)
        self.assertEqual(
            calls,
            [
                ("set", "SRT", "id", "user"),
                ("set", "SRT", "pass", "pw"),
                ("delete", "SRT", "id"),
            ],
        )
        self.assertIn("키체인 저장 중 오류가 발생했습니다", output.getvalue())

    def test_set_login_returns_false_when_storage_fails(self):
        with mock.patch.object(
            srtgo_module.inquirer,
            "prompt",
            return_value={"id": "user", "pass": "pw"},
        ), mock.patch.object(srtgo_module, "SRT"), mock.patch.object(
            srtgo_module, "_keyring_set_many", return_value=False
        ):
            result = srtgo_module.set_login("SRT")

        self.assertFalse(result)

    def test_login_returns_none_when_credentials_remain_unavailable(self):
        with mock.patch.object(
            srtgo_module, "_keyring_get", side_effect=[None, None, None, None]
        ), mock.patch.object(
            srtgo_module, "set_login", return_value=False
        ), mock.patch.object(
            srtgo_module, "SRT"
        ) as srt_ctor:
            result = srtgo_module.login("SRT")

        self.assertIsNone(result)
        srt_ctor.assert_not_called()

    def test_reserve_continues_when_preference_save_fails(self):
        rail = mock.Mock()
        rail.search_train.return_value = []

        def fake_get_password(_service, _account, default=None):
            return default

        with mock.patch.object(srtgo_module, "login", return_value=rail), mock.patch.object(
            srtgo_module, "_keyring_get", side_effect=fake_get_password
        ), mock.patch.object(
            srtgo_module, "get_station", return_value=(["수서", "동대구"], ["수서", "동대구"])
        ), mock.patch.object(
            srtgo_module, "get_options", return_value=[]
        ), mock.patch.object(
            srtgo_module.inquirer,
            "prompt",
            return_value={
                "departure": "수서",
                "arrival": "동대구",
                "date": "20991231",
                "time": "120000",
                "adult": 1,
            },
        ), mock.patch.object(
            srtgo_module, "_keyring_set_many", return_value=False
        ):
            output = io.StringIO()
            with redirect_stdout(output):
                srtgo_module.reserve("SRT")

        rail.search_train.assert_called_once()
        self.assertIn("예약 가능한 열차가 없습니다", output.getvalue())


if __name__ == "__main__":
    unittest.main()
