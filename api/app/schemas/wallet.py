from __future__ import annotations

from typing import Any
from datetime import date, datetime

from pydantic import BaseModel, Field, field_validator, model_validator


class PaymentCardSetRequest(BaseModel):
    card_number: str = Field(min_length=12, max_length=24)
    expiry_month: int = Field(ge=1, le=12)
    expiry_year: int = Field(ge=2000, le=2100)
    birth_date: date
    pin2: str = Field(pattern=r"^\d{2}$")

    @model_validator(mode="before")
    @classmethod
    def reject_legacy_cvv_field(cls, data: Any) -> Any:
        if isinstance(data, dict) and "cvv" in data:
            raise ValueError("cvv field is no longer accepted")
        return data

    @field_validator("card_number")
    @classmethod
    def normalize_card_number(cls, value: str) -> str:
        digits_only = "".join(ch for ch in value if ch.isdigit())
        if len(digits_only) < 12 or len(digits_only) > 19:
            raise ValueError("card_number must be 12 to 19 digits")
        return digits_only


class PaymentCardStatusResponse(BaseModel):
    configured: bool
    card_masked: str | None = None
    expiry_month: int | None = None
    expiry_year: int | None = None
    updated_at: datetime | None = None
    detail: str | None = None


class PaymentCardConfiguredResponse(BaseModel):
    configured: bool
