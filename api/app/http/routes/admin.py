from fastapi import APIRouter, Depends

from app.http.deps import get_current_admin
from app.schemas.auth import MessageResponse

router = APIRouter(dependencies=[Depends(get_current_admin)])


@router.get("", response_model=MessageResponse)
async def admin_only() -> MessageResponse:
    return MessageResponse(message="Admin access granted")
