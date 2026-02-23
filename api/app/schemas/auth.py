from datetime import date, datetime
from typing import Any, Literal
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
    supabase_user_id: str | None = None
    email: EmailStr
    display_name: str | None
    phone_number: str | None
    ui_locale: str
    billing_address: str | None
    billing_address_line1: str | None
    billing_address_line2: str | None
    billing_city: str | None
    billing_state_province: str | None
    billing_country: str | None
    billing_postal_code: str | None
    birthday: date | None
    role: str
    access_status: Literal["pending", "approved", "rejected"]
    access_reviewed_at: datetime | None
    created_at: datetime


class AuthResponse(BaseModel):
    user: UserOut
    notice: str | None = None
    pending_email_change_to: EmailStr | None = None


class MessageResponse(BaseModel):
    message: str


class EmailVerificationRequest(BaseModel):
    email: EmailStr


class EmailVerificationConfirmRequest(BaseModel):
    email: EmailStr
    code: str = Field(min_length=4, max_length=32)


class PasswordResetRequest(BaseModel):
    email: EmailStr


class PasswordResetConfirmRequest(BaseModel):
    email: EmailStr
    code: str = Field(min_length=4, max_length=32)
    new_password: str = Field(min_length=8, max_length=128)


class PasswordVerifyRequest(BaseModel):
    current_password: str = Field(min_length=8, max_length=128)


class EmailChangeConfirmRequest(BaseModel):
    email: EmailStr
    code: str = Field(min_length=4, max_length=32)


class AccountUpdateRequest(BaseModel):
    email: EmailStr | None = None
    display_name: str | None = Field(default=None, max_length=255)
    phone_number: str | None = Field(default=None, max_length=32)
    ui_locale: Literal["en", "ko"] | None = None
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
    passkey_step_up_token: str | None = Field(default=None, min_length=16, max_length=256)


class PasskeyCredentialOut(BaseModel):
    id: UUID
    created_at: datetime
    last_used_at: datetime | None


class PasskeyCredentialListResponse(BaseModel):
    credentials: list[PasskeyCredentialOut]


class PasskeyRegistrationOptionsResponse(BaseModel):
    challenge_id: UUID
    public_key: dict[str, Any]


class PasskeyRegistrationVerifyRequest(BaseModel):
    challenge_id: UUID
    credential: dict[str, Any]


class PasskeyAuthenticationOptionsRequest(BaseModel):
    email: EmailStr


class PasskeyAuthenticationOptionsResponse(BaseModel):
    challenge_id: UUID
    public_key: dict[str, Any]


class PasskeyAuthenticationVerifyRequest(BaseModel):
    email: EmailStr
    challenge_id: UUID
    credential: dict[str, Any]
    remember_me: bool = False


class PasskeyStepUpVerifyRequest(BaseModel):
    challenge_id: UUID
    credential: dict[str, Any]


class PasskeyStepUpVerifyResponse(BaseModel):
    step_up_token: str
