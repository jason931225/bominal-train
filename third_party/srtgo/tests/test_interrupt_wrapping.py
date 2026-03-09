import signal
import unittest
from types import SimpleNamespace
from unittest import mock

from curl_cffi.const import CurlECode
from curl_cffi.curl import CURL_WRITEFUNC_ERROR
from curl_cffi.requests.exceptions import RequestException, Timeout as RequestTimeout

from srtgo.interrupts import wrap_session_for_graceful_sigint


class InterruptAwareSessionTests(unittest.TestCase):
    def test_wrap_converts_cancelled_write_error_to_keyboard_interrupt(self):
        state = {"handler": signal.default_int_handler}

        def fake_signal(_sig, handler):
            previous = state["handler"]
            state["handler"] = handler
            return previous

        def fake_request(*, method, url, content_callback=None, **_kwargs):
            self.assertEqual(method, "GET")
            self.assertEqual(url, "https://example.com")
            state["handler"](signal.SIGINT, None)
            wrote = content_callback(b"chunk")
            self.assertEqual(wrote, CURL_WRITEFUNC_ERROR)
            raise RequestException("cancelled", CurlECode.WRITE_ERROR, None)

        session = SimpleNamespace(request=fake_request)

        with mock.patch.object(signal, "getsignal", return_value=signal.default_int_handler), mock.patch.object(
            signal, "signal", side_effect=fake_signal
        ):
            wrapped = wrap_session_for_graceful_sigint(session)
            with self.assertRaises(KeyboardInterrupt):
                wrapped.request(method="GET", url="https://example.com")

    def test_wrap_converts_cancelled_timeout_to_keyboard_interrupt(self):
        state = {"handler": signal.default_int_handler}

        def fake_signal(_sig, handler):
            previous = state["handler"]
            state["handler"] = handler
            return previous

        def fake_request(*, content_callback=None, **_kwargs):
            state["handler"](signal.SIGINT, None)
            self.assertIsNotNone(content_callback)
            raise RequestTimeout("timed out")

        session = SimpleNamespace(request=fake_request)

        with mock.patch.object(signal, "getsignal", return_value=signal.default_int_handler), mock.patch.object(
            signal, "signal", side_effect=fake_signal
        ):
            wrapped = wrap_session_for_graceful_sigint(session)
            with self.assertRaises(KeyboardInterrupt):
                wrapped.request(method="POST", url="https://example.com")

    def test_wrap_preserves_response_content_without_interrupt(self):
        state = {"handler": signal.default_int_handler}

        def fake_signal(_sig, handler):
            previous = state["handler"]
            state["handler"] = handler
            return previous

        def fake_request(*, content_callback=None, **_kwargs):
            response = SimpleNamespace(content=b"")
            wrote = content_callback(b"abc")
            self.assertEqual(wrote, 3)
            return response

        session = SimpleNamespace(request=fake_request)

        with mock.patch.object(signal, "getsignal", return_value=signal.default_int_handler), mock.patch.object(
            signal, "signal", side_effect=fake_signal
        ):
            wrapped = wrap_session_for_graceful_sigint(session)
            response = wrapped.request(method="GET", url="https://example.com")

        self.assertEqual(response.content, b"abc")


if __name__ == "__main__":
    unittest.main()
