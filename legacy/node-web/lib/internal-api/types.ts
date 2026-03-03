export type InternalProvider = "srt";

export type InternalProviderCredentialsUpsertRequest = {
  username: string;
  password: string;
};

export type InternalProviderCredentialsUpsertResponse = {
  provider: InternalProvider;
  verified: boolean;
  credential_status: "pending" | "verified" | "invalid";
  updated_at: string;
  detail: string | null;
};

export type InternalProviderPaymentMethodUpsertRequest = {
  encrypted_card_number: string;
  encrypted_pin2: string;
  encrypted_birth_date: string;
  encrypted_expiry: string;
  last4: string;
  brand?: string | null;
};

export type InternalProviderPaymentMethodUpsertResponse = {
  provider: InternalProvider;
  configured: boolean;
  source: "evervault";
  brand: string | null;
  last4: string | null;
  updated_at: string | null;
  detail: string | null;
};

export type InternalProviderJobKind =
  | "srt.login"
  | "srt.logout"
  | "srt.search_train"
  | "srt.reserve"
  | "srt.reserve_standby"
  | "srt.reserve_standby_option_settings"
  | "srt.get_reservations"
  | "srt.ticket_info"
  | "srt.cancel"
  | "srt.pay_with_card"
  | "srt.reserve_info"
  | "srt.refund"
  | "srt.clear"
  | (string & {});

export type InternalProviderJobStatus = "queued" | "running" | "completed" | "failed" | "dead_letter";

export type InternalProviderJobCreateRequest = {
  provider: InternalProvider;
  kind: InternalProviderJobKind;
  payload: Record<string, unknown>;
  idempotency_key?: string;
};

export type InternalProviderJobCreateResponse = {
  queued: boolean;
  job_id: string;
  status: InternalProviderJobStatus;
  accepted_at: string;
  queue_key?: string | null;
};

export type InternalProviderJob = {
  job_id: string;
  provider: InternalProvider;
  kind: InternalProviderJobKind;
  status: InternalProviderJobStatus;
  payload: Record<string, unknown> | null;
  result: Record<string, unknown> | null;
  error_code: string | null;
  error_message: string | null;
  created_at: string;
  updated_at: string;
  processed_at: string | null;
};

export type InternalProviderJobEventType =
  | "job.queued"
  | "job.running"
  | "job.retry_scheduled"
  | "job.completed"
  | "job.failed"
  | "job.dead_lettered"
  | (string & {});

export type InternalProviderJobEvent = {
  event_id: string;
  job_id: string;
  sequence: number;
  event_type: InternalProviderJobEventType;
  status: InternalProviderJobStatus;
  created_at: string;
  message: string | null;
  detail: Record<string, unknown> | null;
};

export type InternalProviderJobEventsResponse = {
  job_id: string;
  events: InternalProviderJobEvent[];
  next_cursor: string | null;
};

export type InternalProviderJobEventsQuery = {
  cursor?: string;
  limit?: number;
};

export type InternalApiErrorEnvelope = {
  code?: string;
  message?: string;
  detail?: string;
  request_id?: string;
  details?: Record<string, unknown> | null;
};

export type InternalApiAuthHeaders = {
  internalApiKey?: string;
  internalServiceToken?: string;
};
