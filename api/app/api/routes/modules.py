from fastapi import APIRouter, Depends

from app.api.deps import get_current_user
from app.db.models import User
from app.schemas.module import ModuleListResponse, ModuleOut

router = APIRouter()


@router.get("/modules", response_model=ModuleListResponse)
async def list_modules(_: User = Depends(get_current_user)) -> ModuleListResponse:
    modules = [
        ModuleOut(slug="train", name="Train", coming_soon=False),
        ModuleOut(slug="restaurant", name="Restaurant", coming_soon=True),
        ModuleOut(slug="calendar", name="Calendar", coming_soon=True),
    ]
    return ModuleListResponse(modules=modules)
