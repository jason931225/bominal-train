from fastapi import APIRouter, Depends

from app.http.deps import require_internal_access
from app.core.config import get_settings
from app.schemas.auth import MessageResponse

settings = get_settings()
router = APIRouter(prefix="/api/internal", tags=["internal"], dependencies=[Depends(require_internal_access)])


@router.get("/health", response_model=MessageResponse)
async def internal_health() -> MessageResponse:
    return MessageResponse(message=f"{settings.app_name} internal access granted")
