from __future__ import annotations

from typing import Any
from datetime import date, datetime

from pydantic import BaseModel, Field, field_validator, model_validator

from app.core.config import get_settings


class PaymentCardSetRequest(BaseModel):
    # Legacy plaintext payload (allowed only in PAYMENT_PROVIDER=legacy mode).
    card_number: str | None = Field(default=None, min_length=12, max_length=24)
    expiry_month: int | None = Field(default=None, ge=1, le=12)
    expiry_year: int | None = Field(default=None, ge=2000, le=2100)
    birth_date: date | None = None
    pin2: str | None = Field(default=None, pattern=r"^\d{2}$")
    # Evervault payload (required in PAYMENT_PROVIDER=evervault mode).
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
        return data

    @field_validator("card_number")
    @classmethod
    def normalize_card_number(cls, value: str | None) -> str | None:
        if value is None:
            return None
        digits_only = "".join(ch for ch in value if ch.isdigit())
        if len(digits_only) < 12 or len(digits_only) > 19:
            raise ValueError("card_number must be 12 to 19 digits")
        return digits_only

    @model_validator(mode="after")
    def validate_mode_specific_shape(self) -> "PaymentCardSetRequest":
        settings = get_settings()
        provider_mode = str(settings.payment_provider or "legacy").strip().lower()

        has_legacy = any(
            value is not None
            for value in (
                self.card_number,
                self.expiry_month,
                self.expiry_year,
                self.birth_date,
                self.pin2,
            )
        )
        has_evervault = any(
            value is not None
            for value in (
                self.encrypted_card_number,
                self.encrypted_pin2,
                self.encrypted_birth_date,
                self.encrypted_expiry,
                self.last4,
                self.brand,
            )
        )

        if has_legacy and has_evervault:
            raise ValueError("mixed plaintext and encrypted payloads are not allowed")

        if provider_mode == "evervault":
            if has_legacy:
                raise ValueError("plaintext card fields are not accepted when PAYMENT_PROVIDER=evervault")
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

        # Legacy mode
        if has_evervault:
            raise ValueError("encrypted card payload is not accepted when PAYMENT_PROVIDER=legacy")
        if not all((self.card_number, self.expiry_month, self.expiry_year, self.birth_date, self.pin2)):
            raise ValueError("legacy payload requires card_number, expiry_month, expiry_year, birth_date, and pin2")
        return self

    @property
    def source(self) -> str:
        return "evervault" if self.encrypted_card_number else "legacy"


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
