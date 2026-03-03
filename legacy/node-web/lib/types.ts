export type BominalUser = {
  id: string;
  email: string;
  display_name: string | null;
  phone_number: string | null;
  ui_locale?: "en" | "ko" | string | null;
  billing_address: string | null;
  billing_address_line1: string | null;
  billing_address_line2: string | null;
  billing_city: string | null;
  billing_state_province: string | null;
  billing_country: string | null;
  billing_postal_code: string | null;
  birthday: string | null;
  role: "admin" | "user" | string;
  access_status: "pending" | "approved" | "rejected" | string;
  access_reviewed_at: string | null;
  created_at: string;
};

export type AuthMeResponse = {
  user: BominalUser;
};

export type BominalModule = {
  slug: string;
  name: string;
  coming_soon: boolean;
  enabled: boolean;
  capabilities: string[];
};

export type ModulesResponse = {
  modules: BominalModule[];
};

export type TrainProvider = "SRT" | "KTX";
export type TrainSeatClass = "general" | "special" | "general_preferred" | "special_preferred";
export type TrainTaskState =
  | "QUEUED"
  | "RUNNING"
  | "POLLING"
  | "RESERVING"
  | "PAYING"
  | "COMPLETED"
  | "EXPIRED"
  | "PAUSED"
  | "CANCELLED"
  | "FAILED";

export type TrainSchedule = {
  schedule_id: string;
  provider: TrainProvider;
  departure_at: string;
  arrival_at: string;
  train_no: string;
  dep: string;
  arr: string;
  availability: {
    general: boolean;
    special: boolean;
  };
  metadata: Record<string, string | number | boolean | null>;
};

export type TrainStation = {
  name: string;
  srt_code: string | null;
  srt_supported: boolean;
};

export type TrainProviderCredentialStatus = {
  configured: boolean;
  verified: boolean;
  detail: string | null;
};

export type TrainCredentialStatusResponse = {
  ktx: TrainProviderCredentialStatus;
  srt: TrainProviderCredentialStatus;
};

export type WalletPaymentCardStatus = {
  configured: boolean;
  card_masked: string | null;
  expiry_month: number | null;
  expiry_year: number | null;
  source?: "legacy" | "evervault" | string | null;
  brand?: string | null;
  updated_at: string | null;
  detail: string | null;
};

export type WalletPaymentCardConfigured = {
  configured: boolean;
};

export type TrainTaskSummary = {
  id: string;
  module: string;
  state: TrainTaskState;
  deadline_at: string;
  created_at: string;
  updated_at: string;
  paused_at: string | null;
  cancelled_at: string | null;
  completed_at: string | null;
  failed_at: string | null;
  hidden_at: string | null;
  last_attempt_at: string | null;
  last_attempt_action: string | null;
  last_attempt_ok: boolean | null;
  last_attempt_error_code: string | null;
  last_attempt_error_message_safe: string | null;
  last_attempt_finished_at: string | null;
  next_run_at: string | null;
  retry_now_allowed: boolean;
  retry_now_reason: string | null;
  retry_now_available_at: string | null;
  spec_json: Record<string, unknown>;
  ticket_status?: string | null;
  ticket_paid?: boolean | null;
  ticket_payment_deadline_at?: string | null;
  ticket_reservation_id?: string | null;
  ticket_train_no?: string | null;
  ticket_seat_count?: number | null;
  ticket_seats?: string[] | null;
  ticket_seat_classes?: string[] | null;
};

export type TrainTaskAttempt = {
  id: string;
  action: "SEARCH" | "RESERVE" | "PAY" | "CANCEL" | string;
  provider: string;
  ok: boolean;
  retryable: boolean;
  error_code: string | null;
  error_message_safe: string | null;
  duration_ms: number;
  meta_json_safe: Record<string, unknown> | null;
  started_at: string;
  finished_at: string;
};

export type TrainArtifact = {
  id: string;
  module: string;
  kind: string;
  data_json_safe: Record<string, unknown>;
  created_at: string;
};

export type TrainTaskLastAttemptRuntime = {
  task_id: string;
  last_attempt_at: string | null;
  source: "runtime_redis" | string;
};
