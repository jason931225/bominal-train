from __future__ import annotations

import base64
import logging
import sys
from types import SimpleNamespace

import pytest

from app.core.config import DEFAULT_MASTER_KEY_B64, Settings, get_settings, is_upstash_redis_url
from app.core.crypto.envelope import EnvelopeCrypto
from app.core.crypto.redaction import _luhn_ok as _redaction_luhn_ok
from app.core.crypto.redaction import _mask_pan, _maybe_parse_json_string, redact_sensitive
from app.core.crypto.safe_metadata import _contains_luhn_pan, _luhn_ok as _safe_luhn_ok
from app.core.crypto.safe_metadata import validate_safe_metadata
from app.core.logging import StructuredFormatter, setup_logging
from app.core.supabase_jwt import SupabaseJWTError, _jwk_client, verify_supabase_jwt


def _valid_master_key_b64() -> str:
    return base64.b64encode(b"m" * 32).decode("utf-8")


def test_is_upstash_redis_url_defensive_paths(monkeypatch) -> None:
    assert is_upstash_redis_url(None) is False
    assert is_upstash_redis_url("") is False
    assert is_upstash_redis_url("redis://redis:6379/0") is False
    assert is_upstash_redis_url("rediss://cache.example.upstash.io:6379") is True
    assert is_upstash_redis_url("redis://user@:6379/0") is False

    def _raise(_value: str):
        raise ValueError("boom")

    monkeypatch.setattr("app.core.config.urlparse", _raise)
    assert is_upstash_redis_url("rediss://anything") is False
    assert is_upstash_redis_url("redis://") is False
    assert is_upstash_redis_url("not-a-url") is False
    assert is_upstash_redis_url("redis://:6379/0") is False


def test_settings_list_parsers_and_jwks_resolution() -> None:
    parsed_origins = Settings.parse_cors_origins(["http://localhost:3000", "http://127.0.0.1:3000"])
    parsed_hosts = Settings.parse_payment_provider_allowed_hosts([" APP.SRAIL.OR.KR ", "smart.letskorail.com"])
    assert parsed_origins == ["http://localhost:3000", "http://127.0.0.1:3000"]
    assert parsed_hosts == ["app.srail.or.kr", "smart.letskorail.com"]

    settings = Settings(
        _env_file=None,
        SUPABASE_URL="https://project.supabase.co/",
    )
    assert settings.resolved_supabase_jwks_url == "https://project.supabase.co/auth/v1/.well-known/jwks.json"


def test_settings_uses_explicit_jwks_url_over_derived() -> None:
    settings = Settings(
        _env_file=None,
        SUPABASE_URL="https://project.supabase.co",
        SUPABASE_JWKS_URL="https://custom.example/jwks.json",
    )
    assert settings.resolved_supabase_jwks_url == "https://custom.example/jwks.json"


def test_settings_model_validation_branches(monkeypatch) -> None:
    monkeypatch.setenv("APP_ENV", "production")
    monkeypatch.setenv("DEV_DEMO_AUTH_ENABLED", "false")
    monkeypatch.setenv("MASTER_KEY", _valid_master_key_b64())
    monkeypatch.setenv("INTERNAL_API_KEY", "internal")
    monkeypatch.setenv("PAYMENT_ENABLED", "true")

    # AUTH_MODE validator
    monkeypatch.setenv("AUTH_MODE", "invalid")
    with pytest.raises(ValueError):
        Settings(_env_file=None)

    # Reset to valid auth mode.
    monkeypatch.setenv("AUTH_MODE", "legacy")

    # EMAIL_PROVIDER validator
    monkeypatch.setenv("EMAIL_PROVIDER", "invalid")
    with pytest.raises(ValueError):
        Settings(_env_file=None)
    monkeypatch.setenv("EMAIL_PROVIDER", "disabled")

    # Retry-attempt validator
    monkeypatch.setenv("TRAIN_PROVIDER_RETRY_ATTEMPTS", "0")
    with pytest.raises(ValueError):
        Settings(_env_file=None)
    monkeypatch.setenv("TRAIN_PROVIDER_RETRY_ATTEMPTS", "2")

    # Float timeout validator
    monkeypatch.setenv("TRAIN_PROVIDER_TIMEOUT_CONNECT_SECONDS", "-1")
    with pytest.raises(ValueError):
        Settings(_env_file=None)
    monkeypatch.setenv("TRAIN_PROVIDER_TIMEOUT_CONNECT_SECONDS", "1")

    # Password hash validators
    monkeypatch.setenv("PASSWORD_HASH_TIME_COST", "0")
    with pytest.raises(ValueError):
        Settings(_env_file=None)
    monkeypatch.setenv("PASSWORD_HASH_TIME_COST", "3")

    monkeypatch.setenv("PASSWORD_HASH_MEMORY_COST_KIB", "0")
    with pytest.raises(ValueError):
        Settings(_env_file=None)
    monkeypatch.setenv("PASSWORD_HASH_MEMORY_COST_KIB", "65536")

    monkeypatch.setenv("PASSWORD_HASH_PARALLELISM", "0")
    with pytest.raises(ValueError):
        Settings(_env_file=None)
    monkeypatch.setenv("PASSWORD_HASH_PARALLELISM", "4")

    # Supabase mode missing URL / issuer branches.
    monkeypatch.setenv("AUTH_MODE", "supabase")
    monkeypatch.delenv("SUPABASE_URL", raising=False)
    monkeypatch.delenv("SUPABASE_JWT_ISSUER", raising=False)
    with pytest.raises(ValueError):
        Settings(_env_file=None)

    monkeypatch.setenv("SUPABASE_URL", "https://project.supabase.co")
    with pytest.raises(ValueError):
        Settings(_env_file=None)

    monkeypatch.setenv("SUPABASE_JWT_ISSUER", "https://project.supabase.co/auth/v1")
    settings_ok = Settings(_env_file=None)
    assert settings_ok.auth_mode == "supabase"

    # Supabase storage enabled without key.
    monkeypatch.setenv("AUTH_MODE", "legacy")
    monkeypatch.setenv("SUPABASE_STORAGE_ENABLED", "true")
    monkeypatch.delenv("SUPABASE_SERVICE_ROLE_KEY", raising=False)
    with pytest.raises(ValueError):
        Settings(_env_file=None)

    monkeypatch.setenv("SUPABASE_STORAGE_ENABLED", "false")

    # CVV TTL bounds branches.
    monkeypatch.setenv("PAYMENT_CVV_TTL_MIN_SECONDS", "0")
    with pytest.raises(ValueError):
        Settings(_env_file=None)

    monkeypatch.setenv("PAYMENT_CVV_TTL_MIN_SECONDS", "60")
    monkeypatch.setenv("PAYMENT_CVV_TTL_MAX_SECONDS", "30")
    with pytest.raises(ValueError):
        Settings(_env_file=None)

    monkeypatch.setenv("PAYMENT_CVV_TTL_MAX_SECONDS", "900")
    monkeypatch.setenv("PAYMENT_CVV_TTL_SECONDS", "901")
    with pytest.raises(ValueError):
        Settings(_env_file=None)

    # SMTP conflict branch.
    monkeypatch.setenv("PAYMENT_CVV_TTL_SECONDS", "600")
    monkeypatch.setenv("SMTP_USE_SSL", "true")
    monkeypatch.setenv("SMTP_STARTTLS", "true")
    with pytest.raises(ValueError):
        Settings(_env_file=None)

    # Resend key required branch.
    monkeypatch.setenv("SMTP_USE_SSL", "false")
    monkeypatch.setenv("SMTP_STARTTLS", "false")
    monkeypatch.setenv("EMAIL_PROVIDER", "resend")
    monkeypatch.delenv("RESEND_API_KEY", raising=False)
    with pytest.raises(ValueError):
        Settings(_env_file=None)


def test_production_requires_non_default_master_key_and_internal_key(monkeypatch) -> None:
    monkeypatch.setenv("APP_ENV", "production")
    monkeypatch.setenv("MASTER_KEY", DEFAULT_MASTER_KEY_B64)
    monkeypatch.setenv("INTERNAL_API_KEY", "internal")
    with pytest.raises(ValueError):
        Settings(_env_file=None)

    monkeypatch.setenv("MASTER_KEY", _valid_master_key_b64())
    monkeypatch.delenv("INTERNAL_API_KEY", raising=False)
    with pytest.raises(ValueError):
        Settings(_env_file=None)


def test_get_settings_cache_roundtrip(monkeypatch) -> None:
    monkeypatch.setenv("APP_ENV", "development")
    get_settings.cache_clear()
    first = get_settings()
    second = get_settings()
    assert first is second

    get_settings.cache_clear()
    third = get_settings()
    assert third is not first
    assert third.resolved_supabase_jwks_url is None


def test_envelope_crypto_constructor_and_decrypt_branches() -> None:
    with pytest.raises(ValueError):
        EnvelopeCrypto(master_key_b64=None, master_keys_b64_by_version=None)

    with pytest.raises(ValueError):
        EnvelopeCrypto(master_key_b64=base64.b64encode(b"short").decode("utf-8"))

    valid = _valid_master_key_b64()
    with pytest.raises(ValueError):
        EnvelopeCrypto(master_keys_b64_by_version={1: base64.b64encode(b"short").decode("utf-8")})

    with pytest.raises(ValueError):
        EnvelopeCrypto(master_keys_b64_by_version={1: valid}, active_kek_version=2)

    crypto = EnvelopeCrypto(master_key_b64=valid, kek_version=3)
    encrypted = crypto.encrypt_payload({"x": 1}, "aad")

    with pytest.raises(ValueError):
        crypto.decrypt_payload(
            ciphertext=encrypted.ciphertext,
            nonce=encrypted.nonce,
            wrapped_dek=encrypted.wrapped_dek,
            dek_nonce=encrypted.dek_nonce,
            aad=encrypted.aad,
            enforce_kek_version=True,
        )

    with pytest.raises(ValueError):
        crypto.decrypt_payload(
            ciphertext=encrypted.ciphertext,
            nonce=encrypted.nonce,
            wrapped_dek=encrypted.wrapped_dek,
            dek_nonce=encrypted.dek_nonce,
            aad=encrypted.aad,
            kek_version=999,
        )


def test_envelope_crypto_keyring_multi_version_flow() -> None:
    key_v1 = base64.b64encode(b"a" * 32).decode("utf-8")
    key_v2 = base64.b64encode(b"b" * 32).decode("utf-8")

    crypto = EnvelopeCrypto(
        master_keys_b64_by_version={1: key_v1, 2: key_v2},
        active_kek_version=2,
    )
    encrypted = crypto.encrypt_payload({"hello": "world"}, "aad:keyring")

    decrypted = crypto.decrypt_payload(
        ciphertext=encrypted.ciphertext,
        nonce=encrypted.nonce,
        wrapped_dek=encrypted.wrapped_dek,
        dek_nonce=encrypted.dek_nonce,
        aad=encrypted.aad,
        kek_version=2,
        enforce_kek_version=True,
    )
    assert decrypted == {"hello": "world"}
    assert crypto.kek_version == 2


def test_redaction_internal_branches(monkeypatch) -> None:
    nested = {"a": {"b": {"c": {"d": {"e": {"f": {"g": {"h": {"i": "x"}}}}}}}}}
    assert redact_sensitive(nested, max_depth=2)["a"]["b"]["c"] == "[REDACTED_DEPTH_LIMIT]"

    assert redact_sensitive(("4111 1111 1111 1111", "ok"))[0].startswith("[REDACTED_PAN")
    assert _mask_pan("123456789012") == "123456789012"
    assert _mask_pan("1111 1111 1111 1111") == "1111 1111 1111 1111"
    assert _redaction_luhn_ok("79927398713") is True
    assert _maybe_parse_json_string("  ") is None
    assert _maybe_parse_json_string("{bad-json}") is None
    assert _maybe_parse_json_string("{not-json") is None
    assert redact_sensitive("") == ""

    def _boom(_value: str):
        raise RuntimeError("boom")

    monkeypatch.setattr("app.core.crypto.redaction._redact_string", _boom)
    assert redact_sensitive(b"abc") == "[REDACTED_BYTES]"


def test_safe_metadata_rejects_unredacted_sensitive_fields(monkeypatch) -> None:
    monkeypatch.setattr("app.core.crypto.safe_metadata.redact_sensitive", lambda value: value)

    with pytest.raises(ValueError):
        validate_safe_metadata({"authorization": "Bearer secret"})
    with pytest.raises(ValueError):
        validate_safe_metadata({"cookie": "sid=secret"})
    with pytest.raises(ValueError):
        validate_safe_metadata({"cvv": "123"})


def test_safe_metadata_rejects_pan_after_redaction_if_checker_flags(monkeypatch) -> None:
    checks = iter([False, True])

    monkeypatch.setattr("app.core.crypto.safe_metadata._contains_luhn_pan", lambda _text: next(checks))
    monkeypatch.setattr("app.core.crypto.safe_metadata.redact_sensitive", lambda value: value)

    with pytest.raises(ValueError):
        validate_safe_metadata({"safe": "value"})


def test_safe_metadata_internal_luhn_helpers_cover_branches() -> None:
    assert _safe_luhn_ok("79927398713") is True
    assert _contains_luhn_pan("invalid 1234567890123 and valid 4111 1111 1111 1111") is True


def test_structured_formatter_dev_and_prod(monkeypatch) -> None:
    formatter = StructuredFormatter()

    monkeypatch.setattr("app.core.logging.get_settings", lambda: SimpleNamespace(app_env="development"))
    record_dev = logging.LogRecord(
        name="test.logger",
        level=logging.INFO,
        pathname=__file__,
        lineno=1,
        msg="hello %s",
        args=("world",),
        exc_info=None,
    )
    record_dev.request_id = "req-1"
    record_dev.extra_field = "value"
    dev_output = formatter.format(record_dev)
    assert "hello world" in dev_output
    assert "request_id=req-1" in dev_output
    assert "extra_field=value" in dev_output

    monkeypatch.setattr("app.core.logging.get_settings", lambda: SimpleNamespace(app_env="production"))
    try:
        raise RuntimeError("boom")
    except RuntimeError:
        record_prod = logging.LogRecord(
            name="test.logger",
            level=logging.ERROR,
            pathname=__file__,
            lineno=1,
            msg="failed",
            args=(),
            exc_info=sys.exc_info(),
        )
    prod_output = formatter.format(record_prod)
    assert '"msg": "failed"' in prod_output
    assert '"exc"' in prod_output


def test_setup_logging_configures_root_logger(monkeypatch) -> None:
    monkeypatch.setattr("app.core.logging.get_settings", lambda: SimpleNamespace(app_env="development"))
    setup_logging()
    root_logger = logging.getLogger()
    assert root_logger.level == logging.DEBUG
    assert any(isinstance(handler.formatter, StructuredFormatter) for handler in root_logger.handlers)

    monkeypatch.setattr("app.core.logging.get_settings", lambda: SimpleNamespace(app_env="production"))
    setup_logging()
    assert logging.getLogger().level == logging.INFO
    assert logging.getLogger("uvicorn.access").level == logging.WARNING


def test_supabase_jwt_validation_paths(monkeypatch) -> None:
    # Missing JWKS URL.
    monkeypatch.setattr(
        "app.core.supabase_jwt.get_settings",
        lambda: SimpleNamespace(
            resolved_supabase_jwks_url=None,
            supabase_jwt_issuer="https://issuer",
            supabase_jwt_audience="authenticated",
        ),
    )
    with pytest.raises(SupabaseJWTError):
        verify_supabase_jwt("token")

    # Missing issuer.
    monkeypatch.setattr(
        "app.core.supabase_jwt.get_settings",
        lambda: SimpleNamespace(
            resolved_supabase_jwks_url="https://jwks",
            supabase_jwt_issuer=None,
            supabase_jwt_audience="authenticated",
        ),
    )
    with pytest.raises(SupabaseJWTError):
        verify_supabase_jwt("token")

    class _FakeSigningKey:
        key = "public"

    class _FakeClient:
        def get_signing_key_from_jwt(self, _token: str):
            return _FakeSigningKey()

    monkeypatch.setattr("app.core.supabase_jwt._jwk_client", lambda _url: _FakeClient())

    monkeypatch.setattr(
        "app.core.supabase_jwt.get_settings",
        lambda: SimpleNamespace(
            resolved_supabase_jwks_url="https://jwks",
            supabase_jwt_issuer="https://issuer",
            supabase_jwt_audience="authenticated",
        ),
    )

    def _decode_fail(*_args, **_kwargs):  # noqa: ANN002, ANN003
        raise ValueError("invalid")

    monkeypatch.setattr("app.core.supabase_jwt.jwt.decode", _decode_fail)
    with pytest.raises(SupabaseJWTError):
        verify_supabase_jwt("token")

    monkeypatch.setattr("app.core.supabase_jwt.jwt.decode", lambda *_args, **_kwargs: {"iat": 1, "exp": 2})
    with pytest.raises(SupabaseJWTError):
        verify_supabase_jwt("token")

    monkeypatch.setattr(
        "app.core.supabase_jwt.jwt.decode",
        lambda *_args, **_kwargs: {"sub": "user-1", "iat": 1, "exp": 2},
    )
    assert verify_supabase_jwt("token")["sub"] == "user-1"


def test_supabase_jwk_client_cache(monkeypatch) -> None:
    _jwk_client.cache_clear()
    constructed: list[str] = []

    class _Client:
        def __init__(self, url: str):
            self.url = url

    def _factory(url: str):
        constructed.append(url)
        return _Client(url)

    monkeypatch.setattr("app.core.supabase_jwt.PyJWKClient", _factory)

    first = _jwk_client("https://example/jwks")
    second = _jwk_client("https://example/jwks")
    assert first is second
    assert constructed == ["https://example/jwks"]


def test_settings_rejects_upstash_for_cde_and_empty_hosts_in_production(monkeypatch) -> None:
    monkeypatch.setenv("APP_ENV", "production")
    monkeypatch.setenv("MASTER_KEY", _valid_master_key_b64())
    monkeypatch.setenv("INTERNAL_API_KEY", "internal")
    monkeypatch.setenv("EMAIL_PROVIDER", "disabled")
    monkeypatch.setenv("PAYMENT_ENABLED", "true")

    monkeypatch.setenv("REDIS_URL_CDE", "rediss://cache.example.upstash.io:6379")
    with pytest.raises(ValueError):
        Settings(_env_file=None)

    monkeypatch.setenv("REDIS_URL_CDE", "redis://redis:6379/0")
    monkeypatch.setenv("PAYMENT_PROVIDER_ALLOWED_HOSTS", "")
    with pytest.raises(ValueError):
        Settings(_env_file=None)


def test_settings_rejects_dual_mode(monkeypatch) -> None:
    monkeypatch.setenv("APP_ENV", "production")
    monkeypatch.setenv("MASTER_KEY", _valid_master_key_b64())
    monkeypatch.setenv("INTERNAL_API_KEY", "internal")
    monkeypatch.setenv("AUTH_MODE", "dual")
    monkeypatch.setenv("EMAIL_PROVIDER", "disabled")
    monkeypatch.setenv("PAYMENT_PROVIDER_ALLOWED_HOSTS", "app.srail.or.kr")
    monkeypatch.setenv("REDIS_URL_CDE", "redis://redis:6379/0")

    with pytest.raises(ValueError, match="AUTH_MODE must be one of: legacy, supabase"):
        Settings(_env_file=None)


def test_settings_legacy_mode_is_allowed_and_is_production_property(monkeypatch) -> None:
    monkeypatch.setenv("APP_ENV", "production")
    monkeypatch.setenv("MASTER_KEY", _valid_master_key_b64())
    monkeypatch.setenv("INTERNAL_API_KEY", "internal")
    monkeypatch.setenv("AUTH_MODE", "legacy")
    monkeypatch.setenv("EMAIL_PROVIDER", "disabled")
    monkeypatch.setenv("PAYMENT_PROVIDER_ALLOWED_HOSTS", "app.srail.or.kr")
    monkeypatch.setenv("REDIS_URL_CDE", "redis://redis:6379/0")

    settings = Settings(_env_file=None)
    assert settings.auth_mode == "legacy"
    assert settings.is_production is True


def test_settings_rejects_dev_auth_bypass_in_production(monkeypatch) -> None:
    monkeypatch.setenv("APP_ENV", "production")
    monkeypatch.setenv("MASTER_KEY", _valid_master_key_b64())
    monkeypatch.setenv("INTERNAL_API_KEY", "internal")
    monkeypatch.setenv("EMAIL_PROVIDER", "disabled")
    monkeypatch.setenv("PAYMENT_ENABLED", "false")
    monkeypatch.setenv("DEV_AUTH_BYPASS_ENABLED", "true")
    monkeypatch.setenv("DEV_AUTH_BYPASS_ROLE", "admin")

    with pytest.raises(ValueError, match="DEV_AUTH_BYPASS_ENABLED must be false in production"):
        Settings(_env_file=None)

    monkeypatch.setenv("DEV_AUTH_BYPASS_ENABLED", "false")
    monkeypatch.setenv("DEV_AUTH_BYPASS_ROLE", "invalid")
    with pytest.raises(ValueError, match="DEV_AUTH_BYPASS_ROLE must be one of: admin, user"):
        Settings(_env_file=None)


def test_settings_rejects_dev_demo_auth_in_production(monkeypatch) -> None:
    monkeypatch.setenv("APP_ENV", "production")
    monkeypatch.setenv("MASTER_KEY", _valid_master_key_b64())
    monkeypatch.setenv("INTERNAL_API_KEY", "internal")
    monkeypatch.setenv("EMAIL_PROVIDER", "disabled")
    monkeypatch.setenv("PAYMENT_ENABLED", "false")
    monkeypatch.setenv("DEV_DEMO_AUTH_ENABLED", "true")
    monkeypatch.setenv("DEV_DEMO_ROLE", "admin")

    with pytest.raises(ValueError, match="DEV_DEMO_AUTH_ENABLED must be false in production"):
        Settings(_env_file=None)

    monkeypatch.setenv("DEV_DEMO_AUTH_ENABLED", "false")
    monkeypatch.setenv("DEV_DEMO_ROLE", "invalid")
    with pytest.raises(ValueError, match="DEV_DEMO_ROLE must be one of: admin, user"):
        Settings(_env_file=None)


def test_settings_requires_dev_demo_credentials_when_enabled(monkeypatch) -> None:
    monkeypatch.setenv("APP_ENV", "development")
    monkeypatch.setenv("DEV_DEMO_AUTH_ENABLED", "true")
    monkeypatch.setenv("DEV_DEMO_EMAIL", " ")
    monkeypatch.setenv("DEV_DEMO_PASSWORD", "demo-passkey-123")
    with pytest.raises(ValueError, match="DEV_DEMO_EMAIL is required when DEV_DEMO_AUTH_ENABLED=true"):
        Settings(_env_file=None)

    monkeypatch.setenv("DEV_DEMO_EMAIL", "demo@bominal.dev")
    monkeypatch.setenv("DEV_DEMO_PASSWORD", "short")
    with pytest.raises(ValueError, match="DEV_DEMO_PASSWORD must be at least 8 characters when DEV_DEMO_AUTH_ENABLED=true"):
        Settings(_env_file=None)


def test_redaction_sensitive_key_and_nested_json_string_paths() -> None:
    masked = redact_sensitive({"password": "secret", "nested": '{"card_number":"4111111111111111"}'})
    assert masked["password"] == "[REDACTED]"
    assert isinstance(masked["nested"], dict)
    assert masked["nested"]["card_number"] == "[REDACTED]"


def test_safe_metadata_pan_precheck_and_success_path() -> None:
    with pytest.raises(ValueError):
        validate_safe_metadata({"payload": "4111 1111 1111 1111"})

    sanitized = validate_safe_metadata({"status": "ok", "count": 1})
    assert sanitized == {"status": "ok", "count": 1}
