from __future__ import annotations

RESTAURANT_PROVIDER_RESY = "RESY"
RESTAURANT_PROVIDER_OPENTABLE = "OPENTABLE"

RESTAURANT_SUPPORTED_PROVIDERS: tuple[str, ...] = (
    RESTAURANT_PROVIDER_RESY,
    RESTAURANT_PROVIDER_OPENTABLE,
)

RESTAURANT_CANONICAL_OPERATIONS: tuple[str, ...] = (
    "auth.start",
    "auth.complete",
    "auth.refresh",
    "profile.get",
    "search.availability",
    "reservation.create",
    "reservation.cancel",
)
