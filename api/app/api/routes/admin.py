from fastapi import APIRouter, Depends

from app.api.deps import require_role
from app.db.models import User
from app.schemas.auth import MessageResponse

router = APIRouter()


@router.get("", response_model=MessageResponse)
async def admin_only(_: User = Depends(require_role("admin"))) -> MessageResponse:
    return MessageResponse(message="Admin access granted")
