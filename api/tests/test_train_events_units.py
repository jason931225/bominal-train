from __future__ import annotations

from types import SimpleNamespace
from uuid import uuid4

import pytest

from app.modules.train import events as train_events
from app.modules.train.router import stream_train_task_events


class _FakeRedisPublisher:
    def __init__(self) -> None:
        self.calls: list[tuple[str, str]] = []

    async def publish(self, channel: str, payload: str) -> None:
        self.calls.append((channel, payload))


class _FakePubSub:
    def __init__(self, messages: list[bytes]) -> None:
        self._messages = list(messages)
        self.subscribed_channel: str | None = None
        self.unsubscribed_channel: str | None = None
        self.closed = False

    async def subscribe(self, channel: str) -> None:
        self.subscribed_channel = channel

    async def get_message(self, *, ignore_subscribe_messages: bool, timeout: float):  # noqa: ANN003
        _ = ignore_subscribe_messages
        _ = timeout
        if self._messages:
            return {"type": "message", "data": self._messages.pop(0)}
        return None

    async def unsubscribe(self, channel: str) -> None:
        self.unsubscribed_channel = channel

    async def aclose(self) -> None:
        self.closed = True


class _FakeRedisSubscriber:
    def __init__(self, pubsub: _FakePubSub) -> None:
        self._pubsub = pubsub

    def pubsub(self) -> _FakePubSub:
        return self._pubsub


class _FakeRequest:
    def __init__(self, *, disconnect_after: int) -> None:
        self._calls = 0
        self._disconnect_after = disconnect_after

    async def is_disconnected(self) -> bool:
        self._calls += 1
        return self._calls > self._disconnect_after


def _as_text(chunk: str | bytes) -> str:
    if isinstance(chunk, bytes):
        return chunk.decode("utf-8", errors="ignore")
    return chunk


def test_build_task_state_event_payload_contains_required_fields() -> None:
    payload = train_events.build_task_state_event_payload(
        user_id="user-1",
        task_id="task-1",
        state="RUNNING",
    )

    assert payload["type"] == "task_state_changed"
    assert payload["user_id"] == "user-1"
    assert payload["task_id"] == "task-1"
    assert payload["state"] == "RUNNING"
    assert "updated_at" in payload


@pytest.mark.asyncio
async def test_publish_task_state_event_uses_user_channel(monkeypatch):
    fake_redis = _FakeRedisPublisher()

    async def _fake_get_redis_client():
        return fake_redis

    monkeypatch.setattr("app.modules.train.events.get_redis_client", _fake_get_redis_client)

    await train_events.publish_task_state_event(
        user_id="user-2",
        task_id="task-2",
        state="COMPLETED",
    )

    assert len(fake_redis.calls) == 1
    channel, payload = fake_redis.calls[0]
    assert channel == train_events.task_events_channel("user-2")
    assert '"task_id":"task-2"' in payload
    assert '"state":"COMPLETED"' in payload


@pytest.mark.asyncio
@pytest.mark.parametrize("state", ["POLLING", "RUNNING", "RESERVING", "PAYING"])
async def test_publish_task_state_event_skips_noisy_active_states(monkeypatch, state):
    fake_redis = _FakeRedisPublisher()

    async def _fake_get_redis_client():
        return fake_redis

    monkeypatch.setattr("app.modules.train.events.get_redis_client", _fake_get_redis_client)

    await train_events.publish_task_state_event(
        user_id="user-3",
        task_id="task-3",
        state=state,
    )

    assert fake_redis.calls == []


@pytest.mark.asyncio
async def test_stream_train_task_events_emits_connected_and_task_state(monkeypatch):
    user = SimpleNamespace(id=uuid4())
    channel = train_events.task_events_channel(user.id)
    pubsub = _FakePubSub(messages=[b'{"task_id":"abc","state":"RUNNING"}'])

    async def _fake_get_redis_client():
        return _FakeRedisSubscriber(pubsub)

    monkeypatch.setattr("app.modules.train.router.get_redis_client", _fake_get_redis_client)

    request = _FakeRequest(disconnect_after=2)
    response = await stream_train_task_events(request=request, user=user)

    chunks: list[str] = []
    async for chunk in response.body_iterator:
        chunks.append(_as_text(chunk))
        if len(chunks) >= 2:
            break

    await response.body_iterator.aclose()

    assert chunks[0].startswith("event: connected")
    assert "event: task_state" in chunks[1]
    assert '"state":"RUNNING"' in chunks[1]
    assert pubsub.subscribed_channel == channel
    assert pubsub.unsubscribed_channel == channel
    assert pubsub.closed is True
