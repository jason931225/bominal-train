from __future__ import annotations

from app.modules.restaurant.providers.constants import RESTAURANT_PROVIDER_RESY
from app.modules.restaurant.providers.scaffold import ScaffoldRestaurantProviderClient


class ResyProviderClient(ScaffoldRestaurantProviderClient):
    provider_name = RESTAURANT_PROVIDER_RESY
