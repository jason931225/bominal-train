from __future__ import annotations

from enum import StrEnum


class RestaurantAuthStep(StrEnum):
    REFRESH = "refresh"
    BOOTSTRAP = "bootstrap"
    FAIL = "fail"
