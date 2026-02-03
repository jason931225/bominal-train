from __future__ import annotations

from datetime import datetime
from typing import Any

from pydantic import BaseModel, EmailStr, Field, field_validator


class EmailJobPayload(BaseModel):
    to_email: EmailStr
    subject: str = Field(min_length=1, max_length=200)
    text_body: str = Field(min_length=1, max_length=10000)
    html_body: str | None = Field(default=None, max_length=50000)
    tags: list[str] = Field(default_factory=list, max_length=10)
    metadata: dict[str, str] = Field(default_factory=dict)

    @field_validator("subject")
    @classmethod
    def normalize_subject(cls, value: str) -> str:
        normalized = value.strip()
        if not normalized:
            raise ValueError("subject cannot be blank")
        return normalized


class EmailStatusResponse(BaseModel):
    enabled: bool
    provider: str
    from_name: str
    from_address: str


class EmailTestRequest(BaseModel):
    to_email: EmailStr | None = None
    subject: str | None = Field(default=None, max_length=200)
    message: str | None = Field(default=None, max_length=5000)


class EmailTestResponse(BaseModel):
    queued: bool
    job_id: str | None = None
    recipient: EmailStr
    provider: str
    detail: str
    queued_at: datetime


class EmailSendResult(BaseModel):
    status: str
    recipient: EmailStr
    provider: str
    metadata: dict[str, Any] = Field(default_factory=dict)
