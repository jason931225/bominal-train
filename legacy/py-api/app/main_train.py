"""bominal Train Domain API server."""

from fastapi import Depends

from app.http.app_common import create_base_app
from app.http.deps import get_current_approved_user
from app.modules.train.router import router as train_router

app = create_base_app(description="bominal Train API - train-domain routes")
app.include_router(
    train_router,
    tags=["train", "authenticated"],
    dependencies=[Depends(get_current_approved_user)],
)
