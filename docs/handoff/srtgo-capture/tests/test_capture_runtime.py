from __future__ import annotations

import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
VENDOR = ROOT / "vendor"
if str(VENDOR) not in sys.path:
    sys.path.insert(0, str(VENDOR))

from srtgo_capture.capture_runtime import (  # noqa: E402
    REDACTED,
    _redact_headers,
    _redact_payload,
    _serialize_cookies,
)


class CaptureRuntimeRedactionTests(unittest.TestCase):
    def test_redact_payload_sensitive_fields(self) -> None:
        payload = {
            "hmpgPwdCphd": "pw123",
            "payment_card": {"number": "4111111111111111"},
            "stlCrCrdNo1": "5555444433332222",
            "vanPwd1": "12",
            "hidVanPwd1": "34",
            "athnVal1": "900101",
            "hidAthnVal1": "1234567890",
            "safe_key": "safe_value",
        }

        redacted = _redact_payload(payload)

        self.assertEqual(redacted["hmpgPwdCphd"], REDACTED)
        self.assertEqual(redacted["payment_card"]["number"], REDACTED)
        self.assertEqual(redacted["stlCrCrdNo1"], REDACTED)
        self.assertEqual(redacted["vanPwd1"], REDACTED)
        self.assertEqual(redacted["hidVanPwd1"], REDACTED)
        self.assertEqual(redacted["athnVal1"], REDACTED)
        self.assertEqual(redacted["hidAthnVal1"], REDACTED)
        self.assertEqual(redacted["safe_key"], "safe_value")

    def test_non_sensitive_number_remains(self) -> None:
        payload = {"train": {"number": "301"}}

        redacted = _redact_payload(payload)

        self.assertEqual(redacted["train"]["number"], "301")

    def test_redact_headers_cookies_and_tokens(self) -> None:
        headers = {
            "Cookie": "JSESSIONID=abc",
            "Set-Cookie": "SID=xyz",
            "Authorization": "Bearer token",
            "X-Api-Key": "secret",
            "X-Custom-Token": "secret2",
            "Accept": "application/json",
        }

        redacted = _redact_headers(headers)

        self.assertEqual(redacted["Cookie"], REDACTED)
        self.assertEqual(redacted["Set-Cookie"], REDACTED)
        self.assertEqual(redacted["Authorization"], REDACTED)
        self.assertEqual(redacted["X-Api-Key"], REDACTED)
        self.assertEqual(redacted["X-Custom-Token"], REDACTED)
        self.assertEqual(redacted["Accept"], "application/json")

    def test_serialize_cookies_redacts_values(self) -> None:
        class Cookie:
            def __init__(self, name: str, value: str, domain: str, path: str) -> None:
                self.name = name
                self.value = value
                self.domain = domain
                self.path = path
                self.secure = True
                self.expires = None

        jar = [Cookie("JSESSIONID", "abcdef", "app.srail.or.kr", "/")]

        serialized = _serialize_cookies(jar)

        self.assertEqual(len(serialized), 1)
        self.assertEqual(serialized[0]["name"], "JSESSIONID")
        self.assertEqual(serialized[0]["value"], REDACTED)


if __name__ == "__main__":
    unittest.main()
