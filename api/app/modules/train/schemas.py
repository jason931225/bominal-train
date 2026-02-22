from __future__ import annotations

from datetime import date, datetime
from typing import Literal
from uuid import UUID

from pydantic import BaseModel, Field, field_validator, model_validator

TrainProvider = Literal["SRT", "KTX"]
SeatClass = Literal["general", "special", "general_preferred", "special_preferred"]
TaskState = Literal[
    "QUEUED",
    "RUNNING",
    "POLLING",
    "RESERVING",
    "PAYING",
    "COMPLETED",
    "EXPIRED",
    "PAUSED",
    "CANCELLED",
    "FAILED",
]


class TimeWindow(BaseModel):
    start: str = Field(default="00:00", pattern=r"^\d{2}:\d{2}$")
    end: str = Field(default="23:59", pattern=r"^\d{2}:\d{2}$")

    @model_validator(mode="after")
    def validate_window(self) -> "TimeWindow":
        if self.start > self.end:
            raise ValueError("time_window.start must be <= time_window.end")
        return self


class TrainSearchRequest(BaseModel):
    providers: list[TrainProvider] = Field(min_length=1)
    dep: str = Field(min_length=1, max_length=64)
    arr: str = Field(min_length=1, max_length=64)
    date: date
    time_window: TimeWindow = Field(default_factory=TimeWindow)


class ScheduleOut(BaseModel):
    schedule_id: str
    provider: TrainProvider
    departure_at: datetime
    arrival_at: datetime
    train_no: str
    dep: str
    arr: str
    availability: dict[str, bool]
    metadata: dict[str, str | int | bool | None] = Field(default_factory=dict)


class TrainSearchResponse(BaseModel):
    schedules: list[ScheduleOut]


class TrainStationOut(BaseModel):
    name: str
    srt_code: str | None
    srt_supported: bool


class TrainStationsResponse(BaseModel):
    stations: list[TrainStationOut]


class ProviderCredentialsSetRequest(BaseModel):
    username: str = Field(min_length=2, max_length=128)
    password: str = Field(min_length=4, max_length=256)


class ProviderCredentialStatus(BaseModel):
    configured: bool
    verified: bool = False
    username: str | None = None
    verified_at: datetime | None = None
    detail: str | None = None


class ProviderCredentialsStatusResponse(BaseModel):
    ktx: ProviderCredentialStatus
    srt: ProviderCredentialStatus


class SRTCredentialsSetRequest(ProviderCredentialsSetRequest):
    pass


class KTXCredentialsSetRequest(ProviderCredentialsSetRequest):
    pass


class SRTCredentialStatusResponse(ProviderCredentialStatus):
    pass


class KTXCredentialStatusResponse(ProviderCredentialStatus):
    pass


class TrainPassengers(BaseModel):
    adults: int = Field(ge=1, le=9)
    children: int = Field(default=0, ge=0, le=9)


class RankedTrainSelection(BaseModel):
    schedule_id: str = Field(min_length=4, max_length=128)
    departure_at: datetime
    rank: int = Field(ge=1, le=99)
    provider: TrainProvider | None = None


class TrainTaskCreateRequest(BaseModel):
    provider: TrainProvider | None = None
    dep: str = Field(min_length=1, max_length=64)
    arr: str = Field(min_length=1, max_length=64)
    date: date
    selected_trains_ranked: list[RankedTrainSelection] = Field(min_length=1)
    passengers: TrainPassengers
    seat_class: SeatClass
    auto_pay: bool = True
    notify: bool = False

    @field_validator("selected_trains_ranked")
    @classmethod
    def validate_unique_rank_and_schedule(
        cls, value: list[RankedTrainSelection]
    ) -> list[RankedTrainSelection]:
        seen_ranks: set[int] = set()
        seen_schedules: set[str] = set()
        for item in value:
            if item.rank in seen_ranks:
                raise ValueError("selected_trains_ranked ranks must be unique")
            if item.schedule_id in seen_schedules:
                raise ValueError("selected_trains_ranked schedule_id must be unique")
            seen_ranks.add(item.rank)
            seen_schedules.add(item.schedule_id)
        return sorted(value, key=lambda item: item.rank)


class TaskSummaryOut(BaseModel):
    id: UUID
    module: str
    state: TaskState
    deadline_at: datetime
    created_at: datetime
    updated_at: datetime
    paused_at: datetime | None
    cancelled_at: datetime | None
    completed_at: datetime | None
    failed_at: datetime | None
    hidden_at: datetime | None
    last_attempt_at: datetime | None
    last_attempt_action: str | None
    last_attempt_ok: bool | None
    last_attempt_error_code: str | None
    last_attempt_error_message_safe: str | None
    last_attempt_finished_at: datetime | None
    next_run_at: datetime | None
    retry_now_allowed: bool
    retry_now_reason: str | None
    retry_now_available_at: datetime | None
    spec_json: dict
    ticket_status: str | None = None
    ticket_paid: bool | None = None
    ticket_payment_deadline_at: datetime | None = None
    ticket_reservation_id: str | None = None


class TaskAttemptOut(BaseModel):
    id: UUID
    action: str
    provider: str
    ok: bool
    retryable: bool
    error_code: str | None
    error_message_safe: str | None
    duration_ms: int
    meta_json_safe: dict | None
    started_at: datetime
    finished_at: datetime


class ArtifactOut(BaseModel):
    id: UUID
    module: str
    kind: str
    data_json_safe: dict
    storage_provider: str | None = None
    storage_bucket: str | None = None
    storage_object_path: str | None = None
    storage_content_type: str | None = None
    storage_size_bytes: int | None = None
    storage_checksum_sha256: str | None = None
    created_at: datetime


class TaskDetailOut(BaseModel):
    task: TaskSummaryOut
    attempts: list[TaskAttemptOut]
    artifacts: list[ArtifactOut]


class TaskListResponse(BaseModel):
    tasks: list[TaskSummaryOut]


class TrainTaskCreateResponse(BaseModel):
    task: TaskSummaryOut
    queued: bool
    deduplicated: bool


class TaskActionResponse(BaseModel):
    task: TaskSummaryOut


class TicketCancelResponse(BaseModel):
    status: Literal["cancelled", "not_supported", "not_found", "already_cancelled"]
    detail: str


class ProviderTicketOut(BaseModel):
    car_no: str | None = None
    seat_no: str | None = None
    seat_no_end: str | None = None
    seat_count: int | None = None
    seat_class_code: str | None = None
    seat_class_name: str | None = None
    passenger_type_name: str | None = None
    discount_type_code: str | None = None
    price: int | None = None
    original_price: int | None = None
    discount_amount: int | None = None
    waiting: bool = False


class ProviderReservationOut(BaseModel):
    reservation_id: str
    provider: TrainProvider
    paid: bool
    waiting: bool = False
    expired: bool = False
    running: bool | None = None
    train_no: str | None = None
    train_code: str | None = None
    train_type_code: str | None = None
    train_type_name: str | None = None
    dep: str | None = None
    arr: str | None = None
    departure_at: datetime | None = None
    arrival_at: datetime | None = None
    payment_deadline_at: datetime | None = None
    seat_count: int | None = None
    total_cost: int | None = None
    journey_no: str | None = None
    journey_cnt: str | None = None
    rsv_chg_no: str | None = None
    wct_no: str | None = None
    tickets: list[ProviderTicketOut] = Field(default_factory=list)
    metadata: dict = Field(default_factory=dict)


class ProviderReservationsResponse(BaseModel):
    reservations: list[ProviderReservationOut]


class ProviderTicketInfoResponse(BaseModel):
    reservation_id: str
    tickets: list[ProviderTicketOut]
    wct_no: str | None = None


class ProviderReservationCancelResponse(BaseModel):
    status: Literal["cancelled", "not_found", "failed"]
    detail: str
