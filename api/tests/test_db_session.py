from __future__ import annotations

import pytest

from app.db import session as session_mod


@pytest.mark.asyncio
async def test_get_db_yields_session_from_sessionlocal(monkeypatch):
    session_obj = object()
    state = {"entered": False, "exited": False}

    class _SessionContext:
        async def __aenter__(self):
            state["entered"] = True
            return session_obj

        async def __aexit__(self, *_args):
            state["exited"] = True
            return None

    monkeypatch.setattr(session_mod, "SessionLocal", lambda: _SessionContext())

    gen = session_mod.get_db()
    yielded = await anext(gen)
    assert yielded is session_obj
    await gen.aclose()

    assert state["entered"] is True
    assert state["exited"] is True
