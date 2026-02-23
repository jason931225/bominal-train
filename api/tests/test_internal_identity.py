from __future__ import annotations

import base64
import json
from types import SimpleNamespace

import pytest

from app.core import internal_identity


def _settings(**overrides):
    base = {
        "internal_identity_secret": "super-secret",
        "internal_identity_issuer": "bominal-internal",
        "internal_identity_ttl_seconds": 120,
    }
    base.update(overrides)
    return SimpleNamespace(**base)


def _b64url(data: dict) -> str:
    raw = json.dumps(data, separators=(",", ":"), ensure_ascii=True, sort_keys=True).encode("utf-8")
    return base64.urlsafe_b64encode(raw).decode("ascii").rstrip("=")


def test_mint_and_verify_roundtrip(monkeypatch):
    monkeypatch.setattr(internal_identity, "get_settings", lambda: _settings())
    token = internal_identity.mint_internal_service_token(
        subject="gateway",
        audience="internal-api",
        now_epoch_seconds=1_700_000_000,
    )
    claims = internal_identity.verify_internal_service_token(
        token,
        expected_audience="internal-api",
        now_epoch_seconds=1_700_000_010,
    )
    assert claims.sub == "gateway"
    assert claims.iss == "bominal-internal"
    assert claims.aud == "internal-api"
    assert claims.exp > claims.iat


def test_mint_rejects_missing_secret(monkeypatch):
    monkeypatch.setattr(internal_identity, "get_settings", lambda: _settings(internal_identity_secret=None))
    with pytest.raises(internal_identity.InternalIdentityError):
        internal_identity.mint_internal_service_token(subject="gateway")


def test_mint_uses_runtime_clock_and_rejects_non_positive_ttl(monkeypatch):
    monkeypatch.setattr(internal_identity, "get_settings", lambda: _settings())
    monkeypatch.setattr(internal_identity.time, "time", lambda: 1_700_000_000.9)
    token = internal_identity.mint_internal_service_token(subject="gateway")
    claims = internal_identity.verify_internal_service_token(token, now_epoch_seconds=1_700_000_001)
    assert claims.iat == 1_700_000_000

    with pytest.raises(internal_identity.InternalIdentityError, match="ttl"):
        internal_identity.mint_internal_service_token(subject="gateway", ttl_seconds=0)


def test_verify_rejects_invalid_format(monkeypatch):
    monkeypatch.setattr(internal_identity, "get_settings", lambda: _settings())
    with pytest.raises(internal_identity.InternalIdentityError):
        internal_identity.verify_internal_service_token("bad-token", now_epoch_seconds=1_700_000_000)


def test_verify_rejects_invalid_encoding(monkeypatch):
    monkeypatch.setattr(internal_identity, "get_settings", lambda: _settings())
    with pytest.raises(internal_identity.InternalIdentityError):
        internal_identity.verify_internal_service_token("a.b.c", now_epoch_seconds=1_700_000_000)


def test_verify_rejects_missing_secret(monkeypatch):
    monkeypatch.setattr(internal_identity, "get_settings", lambda: _settings(internal_identity_secret=""))
    with pytest.raises(internal_identity.InternalIdentityError):
        internal_identity.verify_internal_service_token("a.b.c", now_epoch_seconds=1_700_000_000)


def test_verify_rejects_unsupported_algorithm(monkeypatch):
    monkeypatch.setattr(internal_identity, "get_settings", lambda: _settings())
    header = _b64url({"alg": "none", "typ": "BIT"})
    claims = _b64url(
        {
            "iss": "bominal-internal",
            "sub": "gateway",
            "aud": "internal-api",
            "iat": 1_700_000_000,
            "exp": 1_700_000_100,
            "jti": "1",
        }
    )
    token = f"{header}.{claims}.sig"
    with pytest.raises(internal_identity.InternalIdentityError):
        internal_identity.verify_internal_service_token(token, now_epoch_seconds=1_700_000_001)


def test_verify_rejects_bad_signature(monkeypatch):
    monkeypatch.setattr(internal_identity, "get_settings", lambda: _settings())
    token = internal_identity.mint_internal_service_token(
        subject="gateway",
        audience="internal-api",
        now_epoch_seconds=1_700_000_000,
    )
    bad = token.rsplit(".", 1)[0] + ".bad"
    with pytest.raises(internal_identity.InternalIdentityError):
        internal_identity.verify_internal_service_token(bad, now_epoch_seconds=1_700_000_001)


@pytest.mark.parametrize(
    ("claims_patch", "error_fragment"),
    [
        ({"iat": "x"}, "timing"),
        ({"iss": "other"}, "issuer"),
        ({"sub": ""}, "subject"),
        ({"aud": "other"}, "audience"),
        ({"jti": ""}, "identifier"),
        ({"exp": 1_700_000_000}, "expired"),
        ({"iat": 1_700_000_100, "exp": 1_700_000_200}, "future"),
    ],
)
def test_verify_claim_validation_branches(monkeypatch, claims_patch, error_fragment):
    monkeypatch.setattr(internal_identity, "get_settings", lambda: _settings())
    token = internal_identity.mint_internal_service_token(
        subject="gateway",
        audience="internal-api",
        now_epoch_seconds=1_700_000_000,
    )
    header_segment, claims_segment, _sig = token.split(".")
    claims = json.loads(base64.urlsafe_b64decode(claims_segment + "=" * ((4 - len(claims_segment) % 4) % 4)))
    claims.update(claims_patch)
    tampered_claims_segment = _b64url(claims)
    signing_input = f"{header_segment}.{tampered_claims_segment}".encode("ascii")
    signature = internal_identity._b64url_encode(  # noqa: SLF001
        internal_identity._sign(signing_input, secret="super-secret")  # noqa: SLF001
    )
    tampered = f"{header_segment}.{tampered_claims_segment}.{signature}"
    with pytest.raises(internal_identity.InternalIdentityError, match=error_fragment):
        internal_identity.verify_internal_service_token(tampered, now_epoch_seconds=1_700_000_001)


def test_verify_rejects_invalid_expiry_order(monkeypatch):
    monkeypatch.setattr(internal_identity, "get_settings", lambda: _settings())
    token = internal_identity.mint_internal_service_token(
        subject="gateway",
        audience="internal-api",
        now_epoch_seconds=1_700_000_000,
    )
    header_segment, claims_segment, _sig = token.split(".")
    claims = json.loads(base64.urlsafe_b64decode(claims_segment + "=" * ((4 - len(claims_segment) % 4) % 4)))
    claims["iat"] = 1_700_000_000
    claims["exp"] = 1_700_000_000
    tampered_claims_segment = _b64url(claims)
    signing_input = f"{header_segment}.{tampered_claims_segment}".encode("ascii")
    signature = internal_identity._b64url_encode(  # noqa: SLF001
        internal_identity._sign(signing_input, secret="super-secret")  # noqa: SLF001
    )
    tampered = f"{header_segment}.{tampered_claims_segment}.{signature}"
    with pytest.raises(internal_identity.InternalIdentityError, match="expiry"):
        internal_identity.verify_internal_service_token(tampered, now_epoch_seconds=1_699_999_990)
