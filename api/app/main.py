"""Backward-compatible API entrypoint.

This module now re-exports the gateway app so existing local tooling and tests
that import `app.main:app` keep working during the service split migration.
"""

from app.main_gateway import app

__all__ = ["app"]
