import logging

from app.core.logging import StructuredFormatter


def test_structured_formatter_redacts_sensitive_message_and_extras() -> None:
    formatter = StructuredFormatter()

    record = logging.LogRecord(
        name="app.test",
        level=logging.ERROR,
        pathname=__file__,
        lineno=1,
        msg="Authorization: Bearer abc card 4111 1111 1111 1111",
        args=(),
        exc_info=None,
    )
    setattr(record, "user_id", "user-1")
    setattr(record, "path", "/api/pay")
    setattr(record, "method", "POST")

    rendered = formatter.format(record)

    assert "Bearer abc" not in rendered
    assert "4111 1111 1111 1111" not in rendered
    assert "REDACTED" in rendered
