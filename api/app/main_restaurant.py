"""bominal Restaurant Domain API server."""

from app.http.app_common import create_base_app
from app.modules.restaurant.router import router as restaurant_router

app = create_base_app(description="bominal Restaurant API - restaurant-domain routes")
app.include_router(restaurant_router)
