from __future__ import annotations

from app.main import app as default_app
from app.main_gateway import app as gateway_app
from app.main_restaurant import app as restaurant_app
from app.main_train import app as train_app


def _paths(app) -> set[str]:
    return {route.path for route in app.routes}


def test_default_app_exposes_compatibility_route_surface():
    default_paths = _paths(default_app)
    assert "/api/auth/login" in default_paths
    assert "/api/train/stations" in default_paths


def test_gateway_routes_exclude_train_domain_routes():
    gateway_paths = _paths(gateway_app)
    assert "/api/auth/login" in gateway_paths
    assert "/api/train/stations" not in gateway_paths


def test_train_app_routes_include_train_domain_only():
    train_paths = _paths(train_app)
    assert "/api/train/stations" in train_paths
    assert "/api/auth/login" not in train_paths


def test_restaurant_app_exposes_restaurant_health_route():
    restaurant_paths = _paths(restaurant_app)
    assert "/api/restaurant/health" in restaurant_paths
    assert "/api/train/stations" not in restaurant_paths
