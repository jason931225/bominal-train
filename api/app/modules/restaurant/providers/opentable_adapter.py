from __future__ import annotations

from app.modules.restaurant.providers.constants import RESTAURANT_PROVIDER_OPENTABLE
from app.modules.restaurant.providers.scaffold import ScaffoldRestaurantProviderClient


class OpenTableProviderClient(ScaffoldRestaurantProviderClient):
    provider_name = RESTAURANT_PROVIDER_OPENTABLE
