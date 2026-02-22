"""bominal API Gateway server.

Gateway app for shared/public API routes.
Train and restaurant domain routes run in isolated domain API services.
"""

from fastapi import Depends

from app.http.app_common import add_admin_docs, create_base_app
from app.http.deps import get_current_user
from app.http.routes import admin, auth, internal, modules, notifications, wallet

app = create_base_app(description="bominal API Gateway - shared auth, account, and module routes")

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
app.include_router(internal.router)

add_admin_docs(app)
