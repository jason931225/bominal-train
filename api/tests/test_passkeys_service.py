from __future__ import annotations

import builtins
import json
import sys
import types
from datetime import datetime, timezone
from types import SimpleNamespace
from uuid import UUID, uuid4

import pytest
from fastapi import HTTPException, status
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.security import hash_token
from app.db.models import AuthChallenge, PasskeyCredential, Role, User
from app.services import passkeys


class _Descriptor:
    def __init__(self, *, id: bytes, type: str):  # noqa: A002
        self.id = id
        self.type = type


class _UserVerificationRequirement:
    PREFERRED = "preferred"


class _AuthenticatorSelectionCriteria:
    def __init__(self, *, user_verification: str):
        self.user_verification = user_verification


async def _create_user(db_session: AsyncSession, *, email: str) -> User:
    role_user = (await db_session.execute(select(Role).where(Role.name == "user"))).scalar_one()
    user = User(
        email=email,
        password_hash="hashed-password",
        display_name=f"User-{uuid4().hex[:6]}",
        role_id=role_user.id,
        ui_locale="en",
    )
    db_session.add(user)
    await db_session.commit()
    return user


def test_load_webauthn_runtime_success_with_stubbed_modules(monkeypatch: pytest.MonkeyPatch) -> None:
    webauthn_module = types.ModuleType("webauthn")
    webauthn_module.base64url_to_bytes = lambda value: f"decoded:{value}".encode("utf-8")
    webauthn_module.generate_authentication_options = lambda **kwargs: kwargs
    webauthn_module.generate_registration_options = lambda **kwargs: kwargs
    webauthn_module.options_to_json = lambda payload: json.dumps(payload)
    webauthn_module.verify_authentication_response = lambda **kwargs: kwargs
    webauthn_module.verify_registration_response = lambda **kwargs: kwargs

    helpers_module = types.ModuleType("webauthn.helpers")
    structs_module = types.ModuleType("webauthn.helpers.structs")
    structs_module.PublicKeyCredentialDescriptor = _Descriptor
    structs_module.UserVerificationRequirement = _UserVerificationRequirement
    structs_module.AuthenticatorSelectionCriteria = _AuthenticatorSelectionCriteria
    helpers_module.structs = structs_module

    monkeypatch.setitem(sys.modules, "webauthn", webauthn_module)
    monkeypatch.setitem(sys.modules, "webauthn.helpers", helpers_module)
    monkeypatch.setitem(sys.modules, "webauthn.helpers.structs", structs_module)

    runtime = passkeys._load_webauthn_runtime()
    assert runtime["PublicKeyCredentialDescriptor"] is _Descriptor
    assert runtime["UserVerificationRequirement"] is _UserVerificationRequirement
    assert runtime["base64url_to_bytes"]("abc") == b"decoded:abc"


def test_load_webauthn_runtime_raises_when_dependency_missing(monkeypatch: pytest.MonkeyPatch) -> None:
    real_import = builtins.__import__

    def _raise_for_webauthn(name: str, *args, **kwargs):
        if name.startswith("webauthn"):
            raise ImportError("missing runtime")
        return real_import(name, *args, **kwargs)

    monkeypatch.setattr(builtins, "__import__", _raise_for_webauthn)

    with pytest.raises(passkeys.PasskeyRuntimeError, match="Passkey runtime is unavailable"):
        passkeys._load_webauthn_runtime()


def test_encoding_and_rp_origin_helpers(monkeypatch: pytest.MonkeyPatch) -> None:
    raw = b"hello-passkey"
    encoded = passkeys._b64url_encode(raw)
    assert passkeys._b64url_decode(encoded) == raw

    assert passkeys._bytes_to_b64url(raw) == encoded
    assert passkeys._challenge_bytes({"base64url_to_bytes": lambda _: b"decoded"}, encoded) == b"decoded"
    assert passkeys._challenge_bytes({}, encoded) == raw

    monkeypatch.setattr(passkeys.settings, "passkey_rp_id", " custom.rp ", raising=False)
    assert passkeys._effective_passkey_rp_id() == "custom.rp"

    monkeypatch.setattr(passkeys.settings, "passkey_rp_id", "", raising=False)
    monkeypatch.setattr(passkeys.settings, "app_public_base_url", "https://app.bominal.com/base", raising=False)
    assert passkeys._effective_passkey_rp_id() == "app.bominal.com"

    monkeypatch.setattr(passkeys.settings, "passkey_origin", " https://origin.bominal.com ", raising=False)
    assert passkeys._effective_passkey_origin() == "https://origin.bominal.com"

    monkeypatch.setattr(passkeys.settings, "passkey_origin", "", raising=False)
    monkeypatch.setattr(passkeys.settings, "app_public_base_url", "https://www.bominal.com/", raising=False)
    assert passkeys._effective_passkey_origin() == "https://www.bominal.com"


def test_call_with_supported_kwargs_branches(monkeypatch: pytest.MonkeyPatch) -> None:
    def strict_fn(*, keep: int) -> int:
        return keep

    def kwargs_fn(**kwargs: int) -> int:
        return kwargs["keep"] + kwargs["extra"]

    def fallback_fn(**kwargs: int) -> int:
        return kwargs["keep"]

    assert passkeys._call_with_supported_kwargs(strict_fn, keep=7, drop=99) == 7
    assert passkeys._call_with_supported_kwargs(kwargs_fn, keep=2, extra=3) == 5

    monkeypatch.setattr(passkeys.inspect, "signature", lambda _fn: (_ for _ in ()).throw(TypeError("no-signature")))
    assert passkeys._call_with_supported_kwargs(fallback_fn, keep=11) == 11


def test_ensure_passkeys_enabled_branches(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(passkeys.settings, "passkey_enabled", False, raising=False)
    with pytest.raises(HTTPException) as disabled:
        passkeys.ensure_passkeys_enabled()
    assert disabled.value.status_code == status.HTTP_503_SERVICE_UNAVAILABLE

    monkeypatch.setattr(passkeys.settings, "passkey_enabled", True, raising=False)
    monkeypatch.setattr(passkeys, "_effective_passkey_rp_id", lambda: "")
    monkeypatch.setattr(passkeys, "_effective_passkey_origin", lambda: "https://bominal.com")
    with pytest.raises(HTTPException) as missing_rp:
        passkeys.ensure_passkeys_enabled()
    assert missing_rp.value.status_code == status.HTTP_500_INTERNAL_SERVER_ERROR

    monkeypatch.setattr(passkeys, "_effective_passkey_rp_id", lambda: "bominal.com")
    monkeypatch.setattr(passkeys, "_effective_passkey_origin", lambda: "")
    with pytest.raises(HTTPException) as missing_origin:
        passkeys.ensure_passkeys_enabled()
    assert missing_origin.value.status_code == status.HTTP_500_INTERNAL_SERVER_ERROR

    monkeypatch.setattr(passkeys, "_effective_passkey_origin", lambda: "https://bominal.com")

    def _raise_runtime() -> dict[str, object]:
        raise passkeys.PasskeyRuntimeError("runtime missing")

    monkeypatch.setattr(passkeys, "_load_webauthn_runtime", _raise_runtime)
    with pytest.raises(HTTPException) as unavailable:
        passkeys.ensure_passkeys_enabled()
    assert unavailable.value.status_code == status.HTTP_503_SERVICE_UNAVAILABLE

    monkeypatch.setattr(passkeys, "_load_webauthn_runtime", lambda: {"ok": True})
    passkeys.ensure_passkeys_enabled()


@pytest.mark.asyncio
async def test_challenge_create_consume_list_and_delete(db_session: AsyncSession) -> None:
    user = await _create_user(db_session, email=f"challenge-{uuid4().hex[:8]}@example.com")

    challenge = await passkeys._create_challenge(
        db_session,
        purpose=passkeys.PASSKEY_PURPOSE_REGISTER,
        challenge_b64url="challenge-one",
        user_id=user.id,
        email=user.email,
    )
    assert challenge.challenge_hash == hash_token("challenge-one")
    assert challenge.expires_at > challenge.created_at

    consumed = await passkeys._consume_challenge(
        db_session,
        challenge_id=challenge.id,
        purpose=passkeys.PASSKEY_PURPOSE_REGISTER,
        user_id=user.id,
        email=user.email,
    )
    assert consumed.id == challenge.id

    consumed.used_at = datetime.now(timezone.utc)
    await db_session.commit()
    with pytest.raises(HTTPException, match="Invalid or expired passkey challenge"):
        await passkeys._consume_challenge(
            db_session,
            challenge_id=challenge.id,
            purpose=passkeys.PASSKEY_PURPOSE_REGISTER,
            user_id=user.id,
            email=user.email,
        )

    credential_id = passkeys._bytes_to_b64url(b"credential-list")
    db_session.add(
        PasskeyCredential(
            user_id=user.id,
            credential_id=credential_id,
            public_key=passkeys._bytes_to_b64url(b"public-key"),
            sign_count=3,
        )
    )
    await db_session.commit()

    listed = await passkeys.list_passkeys(db_session, user_id=user.id)
    assert len(listed) == 1
    assert listed[0].credential_id == credential_id

    assert await passkeys.delete_passkey(db_session, user_id=user.id, passkey_id=UUID(int=0)) is False
    assert await passkeys.delete_passkey(db_session, user_id=user.id, passkey_id=listed[0].id) is True


@pytest.mark.asyncio
async def test_begin_registration_success_and_missing_challenge(
    db_session: AsyncSession,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    user = await _create_user(db_session, email=f"begin-reg-{uuid4().hex[:8]}@example.com")
    existing_credential_id = passkeys._bytes_to_b64url(b"existing-credential")
    db_session.add(
        PasskeyCredential(
            user_id=user.id,
            credential_id=existing_credential_id,
            public_key=passkeys._bytes_to_b64url(b"existing-public"),
            sign_count=1,
        )
    )
    await db_session.commit()

    captured: dict[str, object] = {}

    def _generate_registration_options(
        *,
        rp_id: str,
        rp_name: str,
        user_name: str,
        user_id: bytes | None = None,
        user_display_name: str | None = None,
        timeout: int = 60000,
        exclude_credentials: list[object] | None = None,
        authenticator_selection: object | None = None,
    ) -> dict[str, object]:
        captured.update(
            {
                "rp_id": rp_id,
                "rp_name": rp_name,
                "user_name": user_name,
                "user_id": user_id,
                "user_display_name": user_display_name,
                "timeout": timeout,
                "exclude_credentials": exclude_credentials or [],
                "authenticator_selection": authenticator_selection,
            }
        )
        return {"challenge": "challenge-register", "rp": {"name": rp_name}}

    runtime = {
        "generate_registration_options": _generate_registration_options,
        "options_to_json": lambda options: json.dumps(options),
        "UserVerificationRequirement": _UserVerificationRequirement,
        "AuthenticatorSelectionCriteria": _AuthenticatorSelectionCriteria,
        "PublicKeyCredentialDescriptor": _Descriptor,
        "base64url_to_bytes": lambda value: passkeys._b64url_decode(value),
    }
    monkeypatch.setattr(passkeys, "_load_webauthn_runtime", lambda: runtime)
    monkeypatch.setattr(passkeys.settings, "app_name", "bominal", raising=False)
    monkeypatch.setattr(passkeys.settings, "passkey_timeout_ms", 120000, raising=False)
    monkeypatch.setattr(passkeys, "_effective_passkey_rp_id", lambda: "www.bominal.com")

    challenge_id, public_key = await passkeys.begin_passkey_registration(db_session, user=user)
    assert isinstance(challenge_id, UUID)
    assert public_key["challenge"] == "challenge-register"
    assert captured["rp_name"] == "bominal"
    assert len(captured["exclude_credentials"]) == 1
    descriptor = captured["exclude_credentials"][0]
    assert isinstance(descriptor, _Descriptor)
    assert descriptor.type == "public-key"
    selection = captured["authenticator_selection"]
    assert isinstance(selection, _AuthenticatorSelectionCriteria)
    assert selection.user_verification == _UserVerificationRequirement.PREFERRED

    challenge_row = (
        await db_session.execute(select(AuthChallenge).where(AuthChallenge.id == challenge_id))
    ).scalar_one()
    assert challenge_row.purpose == passkeys.PASSKEY_PURPOSE_REGISTER
    assert challenge_row.user_id == user.id

    runtime_missing_challenge = dict(runtime)
    runtime_missing_challenge["options_to_json"] = lambda _options: json.dumps({})
    monkeypatch.setattr(passkeys, "_load_webauthn_runtime", lambda: runtime_missing_challenge)
    with pytest.raises(HTTPException) as missing_challenge:
        await passkeys.begin_passkey_registration(db_session, user=user)
    assert missing_challenge.value.status_code == status.HTTP_500_INTERNAL_SERVER_ERROR

    # Legacy runtime branch: no AuthenticatorSelectionCriteria available.
    captured_legacy: dict[str, object] = {}

    def _generate_registration_options_legacy(
        *,
        rp_id: str,
        rp_name: str,
        user_name: str,
        user_id: bytes | None = None,
        user_display_name: str | None = None,
        timeout: int = 60000,
        exclude_credentials: list[object] | None = None,
        user_verification: str = "preferred",
    ) -> dict[str, object]:
        captured_legacy.update(
            {
                "rp_id": rp_id,
                "rp_name": rp_name,
                "user_name": user_name,
                "user_id": user_id,
                "user_display_name": user_display_name,
                "timeout": timeout,
                "exclude_credentials": exclude_credentials or [],
                "user_verification": user_verification,
            }
        )
        return {"challenge": "challenge-register-legacy", "rp": {"name": rp_name}}

    runtime_legacy = {
        "generate_registration_options": _generate_registration_options_legacy,
        "options_to_json": lambda options: json.dumps(options),
        "UserVerificationRequirement": _UserVerificationRequirement,
        "AuthenticatorSelectionCriteria": None,
        "PublicKeyCredentialDescriptor": _Descriptor,
        "base64url_to_bytes": lambda value: passkeys._b64url_decode(value),
    }
    monkeypatch.setattr(passkeys, "_load_webauthn_runtime", lambda: runtime_legacy)
    legacy_challenge_id, legacy_public_key = await passkeys.begin_passkey_registration(db_session, user=user)
    assert isinstance(legacy_challenge_id, UUID)
    assert legacy_public_key["challenge"] == "challenge-register-legacy"
    assert captured_legacy["user_verification"] == _UserVerificationRequirement.PREFERRED


@pytest.mark.asyncio
async def test_complete_registration_verify_conflict_create_and_update(
    db_session: AsyncSession,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    user = await _create_user(db_session, email=f"complete-reg-{uuid4().hex[:8]}@example.com")
    other_user = await _create_user(db_session, email=f"other-reg-{uuid4().hex[:8]}@example.com")

    monkeypatch.setattr(passkeys, "_effective_passkey_rp_id", lambda: "www.bominal.com")
    monkeypatch.setattr(passkeys, "_effective_passkey_origin", lambda: "https://www.bominal.com")

    challenge_for_verify_error = await passkeys._create_challenge(
        db_session,
        purpose=passkeys.PASSKEY_PURPOSE_REGISTER,
        challenge_b64url="challenge-verify-error",
        user_id=user.id,
    )
    runtime_verify_error = {
        "verify_registration_response": lambda **_kwargs: (_ for _ in ()).throw(RuntimeError("bad proof")),
        "base64url_to_bytes": lambda value: passkeys._b64url_decode(value),
    }
    monkeypatch.setattr(passkeys, "_load_webauthn_runtime", lambda: runtime_verify_error)
    with pytest.raises(HTTPException) as verify_error:
        await passkeys.complete_passkey_registration(
            db_session,
            user=user,
            challenge_id=challenge_for_verify_error.id,
            credential={"id": "cred-verify-error"},
        )
    assert verify_error.value.status_code == status.HTTP_400_BAD_REQUEST

    conflict_credential_bytes = b"shared-credential"
    conflict_credential_id = passkeys._bytes_to_b64url(conflict_credential_bytes)
    db_session.add(
        PasskeyCredential(
            user_id=other_user.id,
            credential_id=conflict_credential_id,
            public_key=passkeys._bytes_to_b64url(b"other-public"),
            sign_count=1,
        )
    )
    await db_session.commit()
    challenge_for_conflict = await passkeys._create_challenge(
        db_session,
        purpose=passkeys.PASSKEY_PURPOSE_REGISTER,
        challenge_b64url="challenge-conflict",
        user_id=user.id,
    )
    runtime_conflict = {
        "verify_registration_response": lambda **_kwargs: SimpleNamespace(
            credential_id=conflict_credential_bytes,
            credential_public_key=b"new-public",
            sign_count=9,
        ),
        "base64url_to_bytes": lambda value: passkeys._b64url_decode(value),
    }
    monkeypatch.setattr(passkeys, "_load_webauthn_runtime", lambda: runtime_conflict)
    with pytest.raises(HTTPException) as conflict:
        await passkeys.complete_passkey_registration(
            db_session,
            user=user,
            challenge_id=challenge_for_conflict.id,
            credential={"id": "cred-conflict"},
        )
    assert conflict.value.status_code == status.HTTP_409_CONFLICT

    challenge_for_create = await passkeys._create_challenge(
        db_session,
        purpose=passkeys.PASSKEY_PURPOSE_REGISTER,
        challenge_b64url="challenge-create",
        user_id=user.id,
    )
    created_credential_bytes = b"created-credential"
    runtime_create = {
        "verify_registration_response": lambda **_kwargs: SimpleNamespace(
            credential_id=created_credential_bytes,
            credential_public_key=b"created-public",
            sign_count=11,
        ),
        "base64url_to_bytes": lambda value: passkeys._b64url_decode(value),
    }
    monkeypatch.setattr(passkeys, "_load_webauthn_runtime", lambda: runtime_create)
    created = await passkeys.complete_passkey_registration(
        db_session,
        user=user,
        challenge_id=challenge_for_create.id,
        credential={"id": "cred-created", "response": {"transports": ["usb", "nfc"]}},
    )
    assert created.user_id == user.id
    assert created.sign_count == 11
    assert created.transports == ["usb", "nfc"]
    challenge_create_row = (
        await db_session.execute(select(AuthChallenge).where(AuthChallenge.id == challenge_for_create.id))
    ).scalar_one()
    assert challenge_create_row.used_at is not None

    update_credential_bytes = b"update-credential"
    update_credential_id = passkeys._bytes_to_b64url(update_credential_bytes)
    db_session.add(
        PasskeyCredential(
            user_id=user.id,
            credential_id=update_credential_id,
            public_key=passkeys._bytes_to_b64url(b"old-public"),
            sign_count=2,
        )
    )
    await db_session.commit()
    challenge_for_update = await passkeys._create_challenge(
        db_session,
        purpose=passkeys.PASSKEY_PURPOSE_REGISTER,
        challenge_b64url="challenge-update",
        user_id=user.id,
    )
    runtime_update = {
        "verify_registration_response": lambda **_kwargs: SimpleNamespace(
            credential_id=update_credential_bytes,
            credential_public_key=b"updated-public",
            sign_count=21,
        ),
        "base64url_to_bytes": lambda value: passkeys._b64url_decode(value),
    }
    monkeypatch.setattr(passkeys, "_load_webauthn_runtime", lambda: runtime_update)
    updated = await passkeys.complete_passkey_registration(
        db_session,
        user=user,
        challenge_id=challenge_for_update.id,
        credential={"id": "cred-updated"},
    )
    assert updated.credential_id == update_credential_id
    assert updated.sign_count == 21
    assert updated.public_key == passkeys._bytes_to_b64url(b"updated-public")


@pytest.mark.asyncio
async def test_begin_authentication_success_missing_challenge_and_no_credentials(
    db_session: AsyncSession,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    user = await _create_user(db_session, email=f"begin-auth-{uuid4().hex[:8]}@example.com")

    runtime = {
        "generate_authentication_options": lambda **kwargs: {"challenge": "challenge-auth"},
        "options_to_json": lambda payload: json.dumps(payload),
        "UserVerificationRequirement": _UserVerificationRequirement,
        "PublicKeyCredentialDescriptor": _Descriptor,
        "base64url_to_bytes": lambda value: value.encode("utf-8"),
    }
    monkeypatch.setattr(passkeys, "_load_webauthn_runtime", lambda: runtime)
    monkeypatch.setattr(passkeys.settings, "passkey_timeout_ms", 45000, raising=False)
    monkeypatch.setattr(passkeys, "_effective_passkey_rp_id", lambda: "www.bominal.com")

    with pytest.raises(HTTPException) as no_credentials:
        await passkeys.begin_passkey_authentication(db_session, email=user.email, user=user)
    assert no_credentials.value.status_code == status.HTTP_400_BAD_REQUEST

    db_session.add(
        PasskeyCredential(
            user_id=user.id,
            credential_id=passkeys._bytes_to_b64url(b"auth-credential"),
            public_key=passkeys._bytes_to_b64url(b"auth-public"),
            sign_count=4,
        )
    )
    await db_session.commit()

    challenge_id, public_key = await passkeys.begin_passkey_authentication(db_session, email=user.email, user=user)
    assert isinstance(challenge_id, UUID)
    assert public_key["challenge"] == "challenge-auth"
    challenge_row = (
        await db_session.execute(select(AuthChallenge).where(AuthChallenge.id == challenge_id))
    ).scalar_one()
    assert challenge_row.purpose == passkeys.PASSKEY_PURPOSE_AUTH
    assert challenge_row.email == user.email

    runtime_missing_challenge = dict(runtime)
    runtime_missing_challenge["options_to_json"] = lambda _payload: json.dumps({})
    monkeypatch.setattr(passkeys, "_load_webauthn_runtime", lambda: runtime_missing_challenge)
    with pytest.raises(HTTPException) as missing_challenge:
        await passkeys.begin_passkey_authentication(db_session, email=user.email, user=user)
    assert missing_challenge.value.status_code == status.HTTP_500_INTERNAL_SERVER_ERROR


@pytest.mark.asyncio
async def test_complete_authentication_branches(
    db_session: AsyncSession,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    user = await _create_user(db_session, email=f"complete-auth-{uuid4().hex[:8]}@example.com")
    credential_id = passkeys._bytes_to_b64url(b"auth-credential")
    stored_public_key = passkeys._bytes_to_b64url(b"auth-public-key")
    passkey_row = PasskeyCredential(
        user_id=user.id,
        credential_id=credential_id,
        public_key=stored_public_key,
        sign_count=5,
    )
    db_session.add(passkey_row)
    await db_session.commit()

    monkeypatch.setattr(passkeys, "_effective_passkey_rp_id", lambda: "www.bominal.com")
    monkeypatch.setattr(passkeys, "_effective_passkey_origin", lambda: "https://www.bominal.com")

    runtime = {
        "base64url_to_bytes": lambda value: value.encode("utf-8"),
    }

    challenge_missing_credential = await passkeys._create_challenge(
        db_session,
        purpose=passkeys.PASSKEY_PURPOSE_AUTH,
        challenge_b64url="challenge-missing-credential",
        user_id=user.id,
        email=user.email,
    )
    monkeypatch.setattr(passkeys, "_load_webauthn_runtime", lambda: runtime)
    with pytest.raises(HTTPException) as missing_credential:
        await passkeys.complete_passkey_authentication(
            db_session,
            email=user.email,
            user=user,
            challenge_id=challenge_missing_credential.id,
            credential={},
        )
    assert missing_credential.value.status_code == status.HTTP_400_BAD_REQUEST

    challenge_unknown_credential = await passkeys._create_challenge(
        db_session,
        purpose=passkeys.PASSKEY_PURPOSE_AUTH,
        challenge_b64url="challenge-unknown-credential",
        user_id=user.id,
        email=user.email,
    )
    with pytest.raises(HTTPException) as unknown_credential:
        await passkeys.complete_passkey_authentication(
            db_session,
            email=user.email,
            user=user,
            challenge_id=challenge_unknown_credential.id,
            credential={"id": passkeys._bytes_to_b64url(b"missing")},
        )
    assert unknown_credential.value.status_code == status.HTTP_400_BAD_REQUEST

    challenge_verify_error = await passkeys._create_challenge(
        db_session,
        purpose=passkeys.PASSKEY_PURPOSE_AUTH,
        challenge_b64url="challenge-verify-error",
        user_id=user.id,
        email=user.email,
    )
    runtime_verify_error = dict(runtime)
    runtime_verify_error["verify_authentication_response"] = lambda **_kwargs: (_ for _ in ()).throw(
        RuntimeError("bad assertion")
    )
    monkeypatch.setattr(passkeys, "_load_webauthn_runtime", lambda: runtime_verify_error)
    with pytest.raises(HTTPException) as auth_failed:
        await passkeys.complete_passkey_authentication(
            db_session,
            email=user.email,
            user=user,
            challenge_id=challenge_verify_error.id,
            credential={"id": credential_id},
        )
    assert auth_failed.value.status_code == status.HTTP_400_BAD_REQUEST

    challenge_success = await passkeys._create_challenge(
        db_session,
        purpose=passkeys.PASSKEY_PURPOSE_AUTH,
        challenge_b64url="challenge-success",
        user_id=user.id,
        email=user.email,
    )
    runtime_success = dict(runtime)
    runtime_success["verify_authentication_response"] = lambda **_kwargs: SimpleNamespace(new_sign_count=99)
    monkeypatch.setattr(passkeys, "_load_webauthn_runtime", lambda: runtime_success)
    await passkeys.complete_passkey_authentication(
        db_session,
        email=user.email,
        user=user,
        challenge_id=challenge_success.id,
        credential={"id": credential_id},
    )

    await db_session.refresh(passkey_row)
    assert passkey_row.sign_count == 99
    assert passkey_row.last_used_at is not None

    challenge_success_row = (
        await db_session.execute(select(AuthChallenge).where(AuthChallenge.id == challenge_success.id))
    ).scalar_one()
    assert challenge_success_row.used_at is not None
