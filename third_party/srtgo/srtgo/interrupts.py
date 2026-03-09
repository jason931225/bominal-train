from __future__ import annotations

import signal
import threading
from functools import wraps
from io import BytesIO
from typing import Any

try:
    from curl_cffi.curl import CURL_WRITEFUNC_ERROR
    from curl_cffi.requests.exceptions import RequestException
except ImportError:
    CURL_WRITEFUNC_ERROR = None
    RequestException = Exception


def wrap_session_for_graceful_sigint(session: Any) -> Any:
    if CURL_WRITEFUNC_ERROR is None or getattr(session, "_graceful_sigint_wrapped", False):
        return session

    original_request = session.request

    @wraps(original_request)
    def request(*args, **kwargs):
        if kwargs.get("stream"):
            return original_request(*args, **kwargs)

        if threading.current_thread() is not threading.main_thread():
            return original_request(*args, **kwargs)

        previous_handler = signal.getsignal(signal.SIGINT)
        if previous_handler != signal.default_int_handler:
            return original_request(*args, **kwargs)

        cancelled = False
        body_buffer = BytesIO()
        original_content_callback = kwargs.pop("content_callback", None)

        def handle_sigint(_signum, _frame):
            nonlocal cancelled
            cancelled = True

        def content_callback(chunk):
            if cancelled:
                return CURL_WRITEFUNC_ERROR

            body_buffer.write(chunk)

            if original_content_callback is not None:
                result = original_content_callback(chunk)
                if result is not None:
                    return result

            return len(chunk)

        try:
            signal.signal(signal.SIGINT, handle_sigint)
            response = original_request(*args, content_callback=content_callback, **kwargs)
            response.content = body_buffer.getvalue()
            if cancelled:
                raise KeyboardInterrupt()
            return response
        except RequestException as ex:
            if cancelled:
                raise KeyboardInterrupt() from ex
            raise
        finally:
            signal.signal(signal.SIGINT, previous_handler)

    session.request = request
    session._graceful_sigint_wrapped = True
    return session
