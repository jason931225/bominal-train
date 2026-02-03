from __future__ import annotations

from fastapi import APIRouter, Depends
from sqlalchemy.ext.asyncio import AsyncSession

from app.api.deps import get_current_user
from app.db.models import User
from app.db.session import get_db
from app.schemas.wallet import PaymentCardSetRequest, PaymentCardStatusResponse
from app.services.wallet import get_payment_card_status, set_payment_card

router = APIRouter(prefix="/api/wallet", tags=["wallet"])


@router.get("/payment-card", response_model=PaymentCardStatusResponse)
async def get_wallet_payment_card(
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
) -> PaymentCardStatusResponse:
    return await get_payment_card_status(db, user=user)


@router.post("/payment-card", response_model=PaymentCardStatusResponse)
async def save_wallet_payment_card(
    payload: PaymentCardSetRequest,
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
) -> PaymentCardStatusResponse:
    return await set_payment_card(db, user=user, payload=payload)
