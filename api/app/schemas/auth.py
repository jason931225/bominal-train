from datetime import date, datetime
from uuid import UUID

from pydantic import BaseModel, EmailStr, Field, field_validator


class RegisterRequest(BaseModel):
    email: EmailStr
    password: str = Field(min_length=8, max_length=128)
    display_name: str = Field(min_length=1, max_length=255)

    @field_validator("display_name")
    @classmethod
    def validate_display_name(cls, value: str) -> str:
        normalized = value.strip()
        if not normalized:
            raise ValueError("Display name is required")
        return normalized


class LoginRequest(BaseModel):
    email: EmailStr
    password: str = Field(min_length=8, max_length=128)
    remember_me: bool = False


class UserOut(BaseModel):
    id: UUID
    email: EmailStr
    display_name: str | None
    phone_number: str | None
    billing_address: str | None
    billing_address_line1: str | None
    billing_address_line2: str | None
    billing_city: str | None
    billing_state_province: str | None
    billing_country: str | None
    billing_postal_code: str | None
    birthday: date | None
    role: str
    created_at: datetime


class AuthResponse(BaseModel):
    user: UserOut


class MessageResponse(BaseModel):
    message: str


class AccountUpdateRequest(BaseModel):
    email: EmailStr | None = None
    display_name: str | None = Field(default=None, max_length=255)
    phone_number: str | None = Field(default=None, max_length=32)
    billing_address: str | None = Field(default=None, max_length=1000)
    billing_address_line1: str | None = Field(default=None, max_length=255)
    billing_address_line2: str | None = Field(default=None, max_length=255)
    billing_city: str | None = Field(default=None, max_length=128)
    billing_state_province: str | None = Field(default=None, max_length=128)
    billing_country: str | None = Field(default=None, max_length=128)
    billing_postal_code: str | None = Field(default=None, max_length=32)
    birthday: date | None = None
    new_password: str | None = Field(default=None, min_length=8, max_length=128)
    current_password: str | None = Field(default=None, min_length=8, max_length=128)
