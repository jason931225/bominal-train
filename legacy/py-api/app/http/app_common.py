from __future__ import annotations

from contextlib import asynccontextmanager
import json
import os
from pathlib import Path

from fastapi import Depends, FastAPI, Request
from fastapi.middleware.cors import CORSMiddleware
from fastapi.openapi.docs import get_swagger_ui_html
from fastapi.responses import HTMLResponse, JSONResponse
from sqlalchemy import text

from app.core.config import get_settings
from app.core.logging import get_logger, setup_logging
from app.db.session import SessionLocal
from app.http.deps import get_current_admin

settings = get_settings()
logger = get_logger(__name__)
BUILD_INFO_PATH = Path(__file__).resolve().parents[1] / "build_info.json"


def _normalized_env_version(name: str, *, fallback: str) -> str:
    value = os.getenv(name, "").strip()
    return value or fallback


def _read_build_info_file() -> dict[str, str]:
    if not BUILD_INFO_PATH.exists():
        return {}
    try:
        payload = json.loads(BUILD_INFO_PATH.read_text(encoding="utf-8"))
    except Exception:
        return {}
    if not isinstance(payload, dict):
        return {}
    app_version = str(payload.get("app_version", "")).strip()
    build_version = str(payload.get("build_version", "")).strip()
    result: dict[str, str] = {}
    if app_version:
        result["app_version"] = app_version
    if build_version:
        result["build_version"] = build_version
    return result


def build_version_payload() -> dict[str, str]:
    build_info = _read_build_info_file()
    return {
        "app": settings.app_name,
        "app_version": build_info.get("app_version") or _normalized_env_version("APP_VERSION", fallback="0.0.0"),
        "build_version": build_info.get("build_version") or _normalized_env_version("BUILD_VERSION", fallback="unknown"),
    }


@asynccontextmanager
async def app_lifespan(_app: FastAPI):
    setup_logging()
    logger.info("Application starting", extra={"app": settings.app_name})
    yield
    logger.info("Application shutting down")


def create_base_app(*, description: str) -> FastAPI:
    app = FastAPI(
        title=settings.app_name,
        description=description,
        version="1.0.0",
        lifespan=app_lifespan,
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

    @app.exception_handler(Exception)
    async def global_exception_handler(request: Request, exc: Exception) -> JSONResponse:
        logger.exception(
            "Unhandled exception",
            extra={
                "path": request.url.path,
                "method": request.method,
                "error_type": type(exc).__name__,
            },
        )
        return JSONResponse(
            status_code=500,
            content={"detail": "Internal server error"},
        )

    @app.get("/health")
    async def healthcheck() -> dict[str, str | bool]:
        health: dict[str, str | bool] = {"status": "ok", "app": settings.app_name}

        try:
            async with SessionLocal() as session:
                await session.execute(text("SELECT 1"))
            health["db"] = True
        except Exception:
            health["db"] = False
            health["status"] = "degraded"

        try:
            from app.core.redis import get_redis_client

            redis = await get_redis_client()
            await redis.ping()
            health["redis"] = True
        except Exception:
            health["redis"] = False
            health["status"] = "degraded"

        return health

    @app.get("/api/version")
    @app.get("/version")
    async def version_info() -> dict[str, str]:
        return build_version_payload()

    return app


def add_admin_docs(app: FastAPI) -> None:
    @app.get("/api/docs", response_class=HTMLResponse, include_in_schema=False)
    async def admin_swagger_ui(_=Depends(get_current_admin)):
        return get_swagger_ui_html(
            openapi_url="/api/openapi.json",
            title=f"{settings.app_name} API Docs",
        )

    @app.get("/api/openapi.json", include_in_schema=False)
    async def admin_openapi(_=Depends(get_current_admin)):
        return app.openapi()
