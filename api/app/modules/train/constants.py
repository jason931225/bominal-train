TERMINAL_TASK_STATES = {"COMPLETED", "EXPIRED", "CANCELLED", "FAILED"}
ACTIVE_TASK_STATES = {"QUEUED", "RUNNING", "POLLING", "RESERVING", "PAYING", "PAUSED"}

TASK_MODULE = "train"
SECRET_KIND_SRT_CREDENTIALS = "srt_credentials"
SECRET_KIND_KTX_CREDENTIALS = "ktx_credentials"


def credential_kind(provider: str) -> str:
    """Return the secret kind for a given train provider."""
    if provider == "SRT":
        return SECRET_KIND_SRT_CREDENTIALS
    if provider == "KTX":
        return SECRET_KIND_KTX_CREDENTIALS
    raise ValueError(f"Unsupported provider: {provider}")


ATTEMPT_ACTION_SEARCH = "SEARCH"
ATTEMPT_ACTION_RESERVE = "RESERVE"
ATTEMPT_ACTION_PAY = "PAY"
ATTEMPT_ACTION_CANCEL = "CANCEL"

DEFAULT_BUCKET_CONFIG = {
    "global": {"capacity": 20.0, "refill_per_second": 10.0},
    "provider": {"capacity": 8.0, "refill_per_second": 4.0},
    "credential": {"capacity": 5.0, "refill_per_second": 2.5},
}
