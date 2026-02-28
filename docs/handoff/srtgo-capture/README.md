# srtgo Capture Fork

Instrumented local copy of `srtgo` CLI for provider contract capture.

## What It Does

- Preserves the original interactive CLI flow.
- Captures each HTTP request/response event from `requests` and `curl_cffi` sessions.
- Writes per-event JSON files and a `run_index.json` timeline.

## Run

Use one of these two modes:

1. Active virtualenv mode (recommended if you already installed deps in `.venv`):

```bash
source .venv/bin/activate
python run_cli.py
```

2. `uv` mode:

```bash
uv run run_cli.py
```

If you want `uv` to use your currently active venv explicitly:

```bash
uv run --active run_cli.py
```

Manual dependency install (same set used by upstream `srtgo`) if needed:

```bash
python -m pip install click curl_cffi requests inquirer keyring PyCryptodome prompt_toolkit python-telegram-bot termcolor
```

Canonical command path: `python /Users/jasonlee/bominal/docs/handoff/srtgo-capture/run_cli.py`

If your local macOS keychain backend fails, the launcher automatically falls back to an in-memory keyring for that process and prints:

- `[capture] keyring backend error detected; using in-memory keyring fallback for this run.`

Optional output root override:

```bash
SRTGO_CAPTURE_OUTPUT_DIR=/absolute/path/to/output python /Users/jasonlee/bominal/docs/handoff/srtgo-capture/run_cli.py
```

## Output

Per run output is written under:

- `/Users/jasonlee/bominal/docs/handoff/output/<run_id>/`

Artifacts:

- `run_index.json`
- `0001_<component>_<method>_<endpoint>.json`
- `0002_<component>_<method>_<endpoint>.json`
- ...

`run_index.json` includes:

- ordered event list
- step boundaries (grouped by contiguous operation)
- summary counts (components, operations, status codes, error count)

## Redaction Policy

The capture keeps payloads as raw as possible and redacts only high-risk secrets.

Redacted keys/headers/cookies:

- Password/credential values:
  - `password`, `pass`, `hmpgPwdCphd`, `txtPwd`
- Card number (PAN):
  - `card_number`, `stlCrCrdNo*`, `hidStlCrCrdNo*`, `number`
- Card PIN/password:
  - `vanPwd*`, `hidVanPwd*`
- Identity/auth validation values:
  - `athnVal*`, `hidAthnVal*`
- Session cookies:
  - request `Cookie`, response `Set-Cookie`, serialized cookie jars
- Auth/token headers:
  - `Authorization`, `Proxy-Authorization`, `X-Auth-Token`, `X-Access-Token`, `X-Api-Key`/`X-API-Key`, `Id-Token`, and any header containing `token`

Redacted value marker:

- `"<redacted>"`

## Suggested Manual Coverage

From the CLI run, cover at least:

1. Login success
2. Search trains
3. Reserve ticket
4. Check reservations list
5. Ticket detail lookup
6. Cancel reservation
7. Logout and re-login
8. Standby reservation path (if available)
9. One intentional failure case (login/search/reserve)
10. Payment path only if safe/intended
