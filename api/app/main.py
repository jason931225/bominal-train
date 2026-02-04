from fastapi import Depends, FastAPI
from fastapi.middleware.cors import CORSMiddleware

from app.http.deps import get_current_user
from app.http.routes import admin, auth, internal, modules, notifications, wallet
from app.core.config import get_settings
from app.modules.train.router import router as train_router

settings = get_settings()

app = FastAPI(title=settings.app_name)

app.add_middleware(
    CORSMiddleware,
    allow_origins=settings.cors_origins,
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

app.include_router(auth.public_router, prefix="/api/auth", tags=["auth", "public"])
app.include_router(auth.user_router, prefix="/api/auth", tags=["auth", "authenticated"])
app.include_router(
    modules.router,
    prefix="/api",
    tags=["modules", "authenticated"],
    dependencies=[Depends(get_current_user)],
)
app.include_router(admin.router, prefix="/api/admin", tags=["admin"])
app.include_router(
    wallet.router,
    tags=["wallet", "authenticated"],
    dependencies=[Depends(get_current_user)],
)
app.include_router(
    notifications.router,
    tags=["notifications", "authenticated"],
    dependencies=[Depends(get_current_user)],
)
app.include_router(
    train_router,
    tags=["train", "authenticated"],
    dependencies=[Depends(get_current_user)],
)
app.include_router(internal.router)


@app.get("/health")
async def healthcheck() -> dict[str, str]:
    return {"status": "ok", "app": settings.app_name}


@app.get("/healthz")
async def healthcheck_legacy() -> dict[str, str]:
    return {"status": "ok", "app": settings.app_name}
