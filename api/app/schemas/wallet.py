from __future__ import annotations

from typing import Any
from datetime import datetime

from pydantic import BaseModel, Field, model_validator


class PaymentCardSetRequest(BaseModel):
    # Evervault payload is the only accepted wallet contract.
    encrypted_card_number: str | None = Field(default=None, min_length=4, max_length=4096)
    encrypted_pin2: str | None = Field(default=None, min_length=4, max_length=4096)
    encrypted_birth_date: str | None = Field(default=None, min_length=4, max_length=4096)
    encrypted_expiry: str | None = Field(default=None, min_length=4, max_length=4096)
    last4: str | None = Field(default=None, pattern=r"^\d{4}$")
    brand: str | None = Field(default=None, min_length=1, max_length=32)

    @model_validator(mode="before")
    @classmethod
    def reject_legacy_cvv_field(cls, data: Any) -> Any:
        if isinstance(data, dict):
            forbidden_fields = {"cvv", "cvc", "security_code"}
            matched = [field for field in forbidden_fields if field in data]
            if matched:
                raise ValueError(f"{matched[0]} field is no longer accepted")
            legacy_fields = {"card_number", "expiry_month", "expiry_year", "birth_date", "pin2"}
            if any(field in data for field in legacy_fields):
                raise ValueError("plaintext card fields are not accepted when PAYMENT_PROVIDER=evervault")
        return data

    @model_validator(mode="after")
    def validate_mode_specific_shape(self) -> "PaymentCardSetRequest":
        if not all(
            (
                self.encrypted_card_number,
                self.encrypted_pin2,
                self.encrypted_birth_date,
                self.encrypted_expiry,
                self.last4,
            )
        ):
            raise ValueError("evervault payload requires encrypted fields and last4")
        return self

    @property
    def source(self) -> str:
        return "evervault"


class PaymentCardStatusResponse(BaseModel):
    configured: bool
    card_masked: str | None = None
    expiry_month: int | None = None
    expiry_year: int | None = None
    source: str | None = None
    brand: str | None = None
    updated_at: datetime | None = None
    detail: str | None = None


class PaymentCardConfiguredResponse(BaseModel):
    configured: bool
