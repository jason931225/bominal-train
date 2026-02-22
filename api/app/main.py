"""bominal API server.

FastAPI application providing REST API for the bominal platform.
Includes authentication, train booking, wallet, and admin functionality.
"""

import logging
from contextlib import asynccontextmanager

from fastapi import Depends, FastAPI, Request
from fastapi.middleware.cors import CORSMiddleware
from fastapi.openapi.docs import get_swagger_ui_html
from fastapi.responses import HTMLResponse, JSONResponse
from sqlalchemy import text

from app.http.deps import get_current_user, get_current_admin
from app.http.routes import admin, auth, internal, modules, notifications, wallet
from app.core.config import get_settings
from app.core.logging import setup_logging, get_logger
from app.core.crypto.redaction import redact_sensitive
from app.db.session import SessionLocal
from app.modules.train.router import router as train_router

settings = get_settings()
logger = get_logger(__name__)


def _redis_save_is_disabled(value: dict) -> bool:
    save_value = str(value.get("save", "")).strip()
    return save_value == ""


def _redis_appendonly_is_disabled(value: dict) -> bool:
    appendonly_value = str(value.get("appendonly", "")).strip().lower()
    return appendonly_value in {"", "no", "0", "false"}


async def _enforce_production_security_guards() -> None:
    if not settings.is_production:
        return

    from app.core.redis import get_redis_client

    redis = await get_redis_client()
    try:
        save_cfg = await redis.config_get("save")
        appendonly_cfg = await redis.config_get("appendonly")
    except Exception as exc:
        raise RuntimeError("Failed to verify Redis persistence security guard") from exc

    if not _redis_save_is_disabled(save_cfg) or not _redis_appendonly_is_disabled(appendonly_cfg):
        raise RuntimeError("Redis persistence must be disabled for payment CDE runtime in production")


@asynccontextmanager
async def lifespan(app: FastAPI):
    """Application lifespan events."""
    setup_logging()
    await _enforce_production_security_guards()
    logger.info("Application starting", extra={"app": settings.app_name})
    yield
    logger.info("Application shutting down")


# Disable default docs, we'll add admin-protected versions
app = FastAPI(
    title=settings.app_name,
    description="bominal API - Train booking and automation platform",
    version="1.0.0",
    lifespan=lifespan,
    docs_url=None,
    redoc_url=None,
)

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


@app.exception_handler(Exception)
async def global_exception_handler(request: Request, exc: Exception) -> JSONResponse:
    """Handle uncaught exceptions with structured logging."""
    safe_extra = redact_sensitive(
        {
            "path": request.url.path,
            "method": request.method,
            "error_type": type(exc).__name__,
        }
    )
    logger.exception(
        "Unhandled exception",
        extra=safe_extra,
    )
    return JSONResponse(
        status_code=500,
        content={"detail": "Internal server error"},
    )


@app.get("/health")
async def healthcheck() -> dict[str, str | bool]:
    """Health check with dependency verification.
    
    Returns status of API and its dependencies (database, Redis).
    Used by container orchestration for liveness/readiness probes.
    """
    health: dict[str, str | bool] = {"status": "ok", "app": settings.app_name}
    
    # Check database connectivity
    try:
        async with SessionLocal() as session:
            await session.execute(text("SELECT 1"))
        health["db"] = True
    except Exception:
        health["db"] = False
        health["status"] = "degraded"
    
    # Check Redis connectivity
    try:
        from app.core.redis import get_redis_client
        redis = await get_redis_client()
        await redis.ping()
        health["redis"] = True
    except Exception:
        health["redis"] = False
        health["status"] = "degraded"
    
    return health


# Admin-only API documentation
@app.get("/api/docs", response_class=HTMLResponse, include_in_schema=False)
async def admin_swagger_ui(_=Depends(get_current_admin)):
    """Swagger UI for API documentation. Admin access only."""
    return get_swagger_ui_html(
        openapi_url="/api/openapi.json",
        title=f"{settings.app_name} API Docs",
    )


@app.get("/api/openapi.json", include_in_schema=False)
async def admin_openapi(_=Depends(get_current_admin)):
    """OpenAPI schema. Admin access only."""
    return app.openapi()
