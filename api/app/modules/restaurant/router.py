from __future__ import annotations

from fastapi import APIRouter, Depends

from app.db.models import User
from app.http.deps import get_current_user
from app.modules.restaurant.capabilities import (
    RESTAURANT_CAPABILITIES_COMING_SOON,
    RESTAURANT_CAPABILITIES_EXPOSED,
)

router = APIRouter(prefix="/api/restaurant", tags=["restaurant"])


@router.get("/health")
async def restaurant_health(_: User = Depends(get_current_user)) -> dict[str, object]:
    return {
        "status": "ok",
        "module": "restaurant",
        "enabled": False,
        "coming_soon": True,
        "capabilities": {
            "exposed": list(RESTAURANT_CAPABILITIES_EXPOSED),
            "coming_soon": list(RESTAURANT_CAPABILITIES_COMING_SOON),
        },
    }
