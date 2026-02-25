from fastapi import APIRouter, Depends

from app.core.config import get_settings
from app.http.deps import get_current_approved_user
from app.db.models import User
from app.modules.restaurant.capabilities import RESTAURANT_CAPABILITIES_EXPOSED
from app.schemas.module import ModuleListResponse, ModuleOut

router = APIRouter()
settings = get_settings()

TRAIN_CAPABILITIES_EXPOSED: tuple[str, ...] = (
    "train.search",
    "train.tasks.create",
    "train.tasks.control",
    "train.credentials.manage",
    "train.tickets.manage",
)

CALENDAR_CAPABILITIES_EXPOSED: tuple[str, ...] = ()


@router.get("/modules", response_model=ModuleListResponse)
async def list_modules(_: User = Depends(get_current_approved_user)) -> ModuleListResponse:
    train_capabilities = list(TRAIN_CAPABILITIES_EXPOSED)
    if settings.payment_enabled:
        train_capabilities.append("wallet.payment_card")

    modules: list[ModuleOut] = [
        ModuleOut(
            slug="train",
            name="Train",
            coming_soon=False,
            enabled=True,
            capabilities=train_capabilities,
        ),
        ModuleOut(
            slug="calendar",
            name="Calendar",
            coming_soon=True,
            enabled=False,
            capabilities=list(CALENDAR_CAPABILITIES_EXPOSED),
        ),
    ]
    if settings.restaurant_module_enabled:
        modules.insert(
            1,
            ModuleOut(
                slug="restaurant",
                name="Restaurant",
                coming_soon=True,
                enabled=False,
                capabilities=list(RESTAURANT_CAPABILITIES_EXPOSED),
            ),
        )
    return ModuleListResponse(modules=modules)
