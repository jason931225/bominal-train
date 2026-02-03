from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

from app.api.routes import admin, auth, modules, notifications, wallet
from app.modules.train.router import router as train_router
from app.core.config import get_settings

settings = get_settings()

app = FastAPI(title=settings.app_name)

app.add_middleware(
    CORSMiddleware,
    allow_origins=settings.cors_origins,
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

app.include_router(auth.router, prefix="/api/auth", tags=["auth"])
app.include_router(modules.router, prefix="/api", tags=["modules"])
app.include_router(admin.router, prefix="/api/admin", tags=["admin"])
app.include_router(wallet.router)
app.include_router(notifications.router)
app.include_router(train_router)


@app.get("/health")
async def healthcheck() -> dict[str, str]:
    return {"status": "ok", "app": settings.app_name}


@app.get("/healthz")
async def healthcheck_legacy() -> dict[str, str]:
    return {"status": "ok", "app": settings.app_name}
