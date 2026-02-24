# Redaction and Envelope Hardening Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Harden crypto primitives so redaction meets PCI isolation policy requirements and envelope decryption enforces `kek_version` semantics.

**Architecture:** Add defense-in-depth at two boundaries: data sanitization (`redact_sensitive`) and decrypt-time key-version validation (`EnvelopeCrypto`). Start with failing tests that encode policy guarantees, then implement minimal changes in core crypto modules and wire call sites in `secrets_store`. Keep behavior backward-safe except where policy requires strict failure.

**Tech Stack:** Python 3.12, `pytest`, FastAPI backend modules (`api/app/core/crypto/*`, `api/tests/*`).

---

### Task 1: Add failing tests for PCI-grade redaction behavior

**Files:**
- Create: `api/tests/test_crypto_redaction.py`
- Modify: `api/app/core/crypto/redaction.py`

**Step 1: Write the failing tests**

```python
from app.core.crypto.redaction import redact_sensitive


def test_redact_sensitive_masks_pan_like_strings_even_without_sensitive_key_names():
    payload = {"data": "4111111111111111", "nested": {"note": "card 5555555555554444"}}
    redacted = redact_sensitive(payload)
    assert redacted["data"] == "[REDACTED]"
    assert redacted["nested"]["note"] == "[REDACTED]"


def test_redact_sensitive_masks_authorization_cookie_headers():
    payload = {
        "headers": {
            "Authorization": "Bearer secret-token",
            "Cookie": "bominal_session=abc123",
            "Set-Cookie": "session=abc; HttpOnly",
        }
    }
    redacted = redact_sensitive(payload)
    assert redacted["headers"]["Authorization"] == "[REDACTED]"
    assert redacted["headers"]["Cookie"] == "[REDACTED]"
    assert redacted["headers"]["Set-Cookie"] == "[REDACTED]"


def test_redact_sensitive_redacts_nested_json_string_payloads():
    payload = {"raw": '{"number":"4111111111111111","cvv":"123"}'}
    redacted = redact_sensitive(payload)
    assert redacted["raw"] == "[REDACTED]"
```

**Step 2: Run test to verify it fails**

Run: `cd api && ./.venv/bin/pytest -q tests/test_crypto_redaction.py -k "pan or authorization or nested_json"`
Expected: FAIL (current implementation only redacts by key names).

**Step 3: Write minimal implementation in `redaction.py`**

```python
# add patterns and header keys
PAN_PATTERN = re.compile(r"\b(?:\d[ -]?){13,19}\b")
SENSITIVE_HEADER_KEYS = {"authorization", "cookie", "set-cookie"}


def _string_is_sensitive(value: str) -> bool:
    lowered = value.lower()
    if "bearer " in lowered:
        return True
    if PAN_PATTERN.search(value):
        return True
    if "cvv" in lowered:
        return True
    return False
```

```python
# in redact_sensitive(...)
if isinstance(data, str):
    # try parsing nested JSON string, then fallback to pattern scan
    ...
```

**Step 4: Run test to verify it passes**

Run: `cd api && ./.venv/bin/pytest -q tests/test_crypto_redaction.py`
Expected: PASS.

**Step 5: Commit**

```bash
git add api/tests/test_crypto_redaction.py api/app/core/crypto/redaction.py
git commit -m "test+fix: harden redaction for pan/header/string payload leaks"
```

### Task 2: Add failing tests for envelope `kek_version` enforcement

**Files:**
- Create: `api/tests/test_crypto_envelope.py`
- Modify: `api/app/core/crypto/envelope.py`

**Step 1: Write the failing tests**

```python
import pytest
from app.core.crypto.envelope import EnvelopeCrypto


def test_decrypt_payload_rejects_mismatched_kek_version():
    key = "MDEyMzQ1Njc4OWFiY2RlZjAxMjM0NTY3ODlhYmNkZWY="
    crypto = EnvelopeCrypto(master_key_b64=key, kek_version=2)
    encrypted = crypto.encrypt_payload(payload={"x": 1}, aad_text="unit:test")

    with pytest.raises(ValueError, match="kek_version"):
        crypto.decrypt_payload(
            ciphertext=encrypted.ciphertext,
            nonce=encrypted.nonce,
            wrapped_dek=encrypted.wrapped_dek,
            dek_nonce=encrypted.dek_nonce,
            aad=encrypted.aad,
            kek_version=1,
            enforce_kek_version=True,
        )


def test_decrypt_payload_allows_matching_kek_version_when_enforced():
    key = "MDEyMzQ1Njc4OWFiY2RlZjAxMjM0NTY3ODlhYmNkZWY="
    crypto = EnvelopeCrypto(master_key_b64=key, kek_version=2)
    encrypted = crypto.encrypt_payload(payload={"x": 1}, aad_text="unit:test")

    payload = crypto.decrypt_payload(
        ciphertext=encrypted.ciphertext,
        nonce=encrypted.nonce,
        wrapped_dek=encrypted.wrapped_dek,
        dek_nonce=encrypted.dek_nonce,
        aad=encrypted.aad,
        kek_version=2,
        enforce_kek_version=True,
    )
    assert payload == {"x": 1}
```

**Step 2: Run test to verify it fails**

Run: `cd api && ./.venv/bin/pytest -q tests/test_crypto_envelope.py`
Expected: FAIL because `decrypt_payload` currently has no version-aware parameters.

**Step 3: Write minimal implementation in `envelope.py`**

```python
# signature extension
def decrypt_payload(..., kek_version: int | None = None, enforce_kek_version: bool = False) -> dict[str, Any]:
    if enforce_kek_version and kek_version is None:
        raise ValueError("kek_version is required when enforcement is enabled")
    if enforce_kek_version and kek_version != self._kek_version:
        raise ValueError("kek_version mismatch")
    ...
```

**Step 4: Run test to verify it passes**

Run: `cd api && ./.venv/bin/pytest -q tests/test_crypto_envelope.py`
Expected: PASS.

**Step 5: Commit**

```bash
git add api/tests/test_crypto_envelope.py api/app/core/crypto/envelope.py
git commit -m "test+fix: enforce optional kek_version checks in envelope decryption"
```

### Task 3: Wire strict decryption at secrets-store boundary

**Files:**
- Modify: `api/app/core/crypto/secrets_store.py`
- Modify: `api/tests/test_wallet.py`

**Step 1: Write failing integration-adjacent test in `test_wallet.py`**

```python
# add a test that tampered secret.kek_version rejects decryption path used by wallet status
# expected API behavior: configured=False with detail indicating load failure
```

**Step 2: Run test to verify it fails**

Run: `cd api && ./.venv/bin/pytest -q tests/test_wallet.py -k "kek_version"`
Expected: FAIL (current `decrypt_secret` does not pass stored version for validation).

**Step 3: Implement minimal wiring in `secrets_store.py`**

```python
return get_envelope_crypto().decrypt_payload(
    ...,
    kek_version=secret.kek_version,
    enforce_kek_version=True,
)
```

**Step 4: Run test to verify it passes**

Run: `cd api && ./.venv/bin/pytest -q tests/test_wallet.py -k "kek_version"`
Expected: PASS.

**Step 5: Commit**

```bash
git add api/app/core/crypto/secrets_store.py api/tests/test_wallet.py
git commit -m "fix: enforce secret kek_version validation at decrypt boundary"
```

### Task 4: Add memory-lifetime hardening and compatibility notes

**Files:**
- Modify: `api/app/core/crypto/envelope.py`
- Modify: `docs/humans/security/SECURITY.md`

**Step 1: Write failing test for serialization compatibility contract note (doc gate)**

```python
# add a lightweight assertion in test_crypto_envelope.py that encrypted/decrypted unicode payload round-trips
# to guard accidental serializer changes.
```

**Step 2: Run test to verify baseline behavior**

Run: `cd api && ./.venv/bin/pytest -q tests/test_crypto_envelope.py -k "unicode or round"`
Expected: PASS (or FAIL if behavior differs and requires explicit handling).

**Step 3: Implement minimal hardening/documentation**

```python
# envelope.py: reduce lifetime of sensitive vars (scoped usage + explicit del)
# SECURITY.md: add short crypto compatibility note for JSON serialization settings.
```

**Step 4: Run tests to verify pass**

Run: `cd api && ./.venv/bin/pytest -q tests/test_crypto_envelope.py tests/test_crypto_redaction.py`
Expected: PASS.

**Step 5: Commit**

```bash
git add api/app/core/crypto/envelope.py api/tests/test_crypto_envelope.py docs/humans/security/SECURITY.md
git commit -m "hardening: reduce sensitive-memory lifetime and document crypto serialization contract"
```

### Task 5: Full targeted regression and policy checks

**Files:**
- No file modifications

**Step 1: Run targeted backend regression suite**

Run:
- `cd api && ./.venv/bin/pytest -q tests/test_crypto_redaction.py tests/test_crypto_envelope.py tests/test_wallet.py tests/test_provider_egress_transport.py`
Expected: PASS.

**Step 2: Run docs consistency checks after security doc update**

Run:
- `bash infra/tests/test_docs_pointers.sh`
- `bash infra/tests/test_docs_consistency.sh`
Expected: PASS.

**Step 3: Commit (if additional fixes needed from regression)**

```bash
git add <any-updated-files>
git commit -m "test: complete crypto/redaction hardening regression verification"
```

### Task 6: Audit handoff and next-phase hook

**Files:**
- Modify: `CHANGELOG.md`

**Step 1: Add commit-based changelog entries for behavior changes**

```md
- [<sha>] Hardened redaction to mask PAN/header/string payload leaks.
- [<sha>] Enforced `kek_version` validation during secret decryption.
```

**Step 2: Run changelog validation**

Run: `bash infra/tests/test_changelog.sh`
Expected: PASS.

**Step 3: Commit changelog**

```bash
git add CHANGELOG.md
git commit -m "docs: record crypto/redaction hardening changes in changelog"
```

**Step 4: Prepare next audit phase input**

Run: `git show --name-only --oneline -1`
Expected: final commit metadata ready for downstream worker/wallet path audit.
