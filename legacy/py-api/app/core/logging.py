"""Structured logging configuration for bominal API."""

import json
import logging
import sys
from typing import Any

from app.core.config import get_settings
from app.core.crypto.redaction import redact_sensitive


class StructuredFormatter(logging.Formatter):
    """JSON-like structured log formatter for production.
    
    In development: Human-readable format with timestamp, level, logger, message.
    In production: JSON format for log aggregation systems.
    """

    def format(self, record: logging.LogRecord) -> str:
        settings = get_settings()

        log_data: dict[str, Any] = {
            "ts": self.formatTime(record),
            "level": record.levelname,
            "logger": record.name,
            "msg": record.getMessage(),
        }

        if record.exc_info:
            log_data["exc"] = self.formatException(record.exc_info)

        # Include explicit structured extras.
        for key in ("request_id", "user_id", "path", "method", "status_code", "duration_ms"):
            if hasattr(record, key):
                log_data[key] = getattr(record, key)

        # Include any additional extras while excluding standard LogRecord keys.
        standard_keys = {
            "name",
            "msg",
            "args",
            "levelname",
            "levelno",
            "pathname",
            "filename",
            "module",
            "exc_info",
            "exc_text",
            "stack_info",
            "lineno",
            "funcName",
            "created",
            "msecs",
            "relativeCreated",
            "thread",
            "threadName",
            "processName",
            "process",
        }
        for key, value in record.__dict__.items():
            if key in standard_keys or key in log_data:
                continue
            log_data[key] = value

        log_data = redact_sensitive(log_data)

        # In development, use readable format
        if settings.app_env == "development":
            extras = " ".join(f"{k}={v}" for k, v in log_data.items() if k not in ("ts", "level", "logger", "msg"))
            base = f"{log_data['ts']} {log_data['level']:8} [{log_data['logger']}] {log_data['msg']}"
            return f"{base} {extras}".strip() if extras else base

        # In production, use JSON format for log aggregation
        return json.dumps(log_data, default=str)


def setup_logging() -> None:
    """Configure application-wide structured logging.
    
    Sets log level based on environment:
    - development: DEBUG level with readable format
    - production: INFO level with JSON format
    """
    settings = get_settings()
    
    log_level = logging.DEBUG if settings.app_env == "development" else logging.INFO
    
    # Configure root logger
    root_logger = logging.getLogger()
    root_logger.setLevel(log_level)
    
    # Remove existing handlers
    for handler in root_logger.handlers[:]:
        root_logger.removeHandler(handler)
    
    # Add structured handler
    handler = logging.StreamHandler(sys.stdout)
    handler.setFormatter(StructuredFormatter())
    root_logger.addHandler(handler)
    
    # Silence noisy libraries
    logging.getLogger("uvicorn.access").setLevel(logging.WARNING)
    logging.getLogger("httpx").setLevel(logging.WARNING)
    logging.getLogger("httpcore").setLevel(logging.WARNING)
    
    logging.info("Logging configured", extra={"app_env": settings.app_env})


def get_logger(name: str) -> logging.Logger:
    """Get a logger instance with the given name.
    
    Args:
        name: Logger name, typically __name__ of the calling module.
        
    Returns:
        Configured logger instance.
    """
    return logging.getLogger(name)
