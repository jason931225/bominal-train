from fastapi import APIRouter, Depends

from app.http.deps import get_current_user
from app.db.models import User
from app.modules.restaurant.capabilities import RESTAURANT_CAPABILITIES_EXPOSED
from app.schemas.module import ModuleListResponse, ModuleOut

router = APIRouter()

TRAIN_CAPABILITIES_EXPOSED: tuple[str, ...] = (
    "train.search",
    "train.tasks.create",
    "train.tasks.control",
    "train.credentials.manage",
    "train.tickets.manage",
    "wallet.payment_card",
)

CALENDAR_CAPABILITIES_EXPOSED: tuple[str, ...] = ()


@router.get("/modules", response_model=ModuleListResponse)
async def list_modules(_: User = Depends(get_current_user)) -> ModuleListResponse:
    modules = [
        ModuleOut(
            slug="train",
            name="Train",
            coming_soon=False,
            enabled=True,
            capabilities=list(TRAIN_CAPABILITIES_EXPOSED),
        ),
        ModuleOut(
            slug="restaurant",
            name="Restaurant",
            coming_soon=True,
            enabled=False,
            capabilities=list(RESTAURANT_CAPABILITIES_EXPOSED),
        ),
        ModuleOut(
            slug="calendar",
            name="Calendar",
            coming_soon=True,
            enabled=False,
            capabilities=list(CALENDAR_CAPABILITIES_EXPOSED),
        ),
    ]
    return ModuleListResponse(modules=modules)
