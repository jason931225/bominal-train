/* eslint-disable @typescript-eslint/consistent-type-definitions */

export type JsonObject = Record<string, unknown>;

export interface ApiError {
  status?: string;
  code?: string;
  message: string;
  request_id?: string;
}

export interface CreateProviderJobRequest {
  provider: string;
  operation: string;
  idempotency_key?: string;
  payload: JsonObject;
}

export interface CreateProviderJobResult {
  accepted: boolean;
  job_id: string;
  status: string;
}

export interface ProviderJobResult {
  job_id: string;
  provider: string;
  operation: string;
  status: string;
}

export interface ProviderJobEvent {
  sequence: number;
  event_type: string;
  occurred_at: string;
  detail: JsonObject;
}

export interface CursorPage {
  limit: number;
  has_more: boolean;
  next_cursor?: string;
}

export interface ProviderJobEventsResponse {
  items: ProviderJobEvent[];
  page: CursorPage;
}

export interface PutProviderCredentialsRequest {
  subject_ref?: string;
  identity_ciphertext: string;
  password_ciphertext: string;
}

export interface PutProviderCredentialsResult {
  accepted: boolean;
  provider: string;
  credential_ref: string;
  contract: string;
  auth_probe_status: string;
  auth_probe_message: string;
}

export interface PutProviderPaymentMethodRequest {
  owner_ref?: string;
  payment_method_ref?: string;
  card_brand?: string;
  card_last4: string;
  pan_ciphertext: string;
  expiry_month_ciphertext: string;
  expiry_year_ciphertext: string;
  birth_or_business_number_ciphertext: string;
  card_password_two_digits_ciphertext: string;
}

export interface PutProviderPaymentMethodResult {
  accepted: boolean;
  provider: string;
  payment_method_ref: string;
  contract: string;
}

export interface CreateInviteRequest {
  email: string;
  expires_in_seconds?: number;
}

export interface CreateInviteResponse {
  invite_url: string;
  expires_at: string;
}

export interface InternalSdkOptions {
  baseUrl: string;
  serviceToken: string;
  fetchImpl?: typeof fetch;
}

export interface ListProviderJobEventsParams {
  cursor?: string;
  limit?: number;
}

export class InternalApiError extends Error {
  readonly statusCode: number;
  readonly body: ApiError | null;

  constructor(statusCode: number, body: ApiError | null, fallbackMessage: string) {
    super(body?.message || fallbackMessage);
    this.name = "InternalApiError";
    this.statusCode = statusCode;
    this.body = body;
  }
}

export class InternalSdkClient {
  private readonly baseUrl: string;
  private readonly serviceToken: string;
  private readonly fetchImpl: typeof fetch;

  constructor(options: InternalSdkOptions) {
    this.baseUrl = options.baseUrl.replace(/\/+$/, "");
    this.serviceToken = options.serviceToken;
    this.fetchImpl = options.fetchImpl || fetch;
  }

  async createProviderJob(
    payload: CreateProviderJobRequest,
  ): Promise<CreateProviderJobResult> {
    return this.request<CreateProviderJobResult>("/internal/v1/provider-jobs", {
      method: "POST",
      body: JSON.stringify(payload),
    });
  }

  async getProviderJob(jobId: string): Promise<ProviderJobResult> {
    return this.request<ProviderJobResult>(
      `/internal/v1/provider-jobs/${encodeURIComponent(jobId)}`,
      { method: "GET" },
    );
  }

  async listProviderJobEvents(
    jobId: string,
    params: ListProviderJobEventsParams = {},
  ): Promise<ProviderJobEventsResponse> {
    const query = new URLSearchParams();
    if (params.cursor) {
      query.set("cursor", params.cursor);
    }
    if (typeof params.limit === "number") {
      query.set("limit", String(params.limit));
    }
    const suffix = query.toString();
    const path = `/internal/v1/provider-jobs/${encodeURIComponent(jobId)}/events${
      suffix ? `?${suffix}` : ""
    }`;
    return this.request<ProviderJobEventsResponse>(path, { method: "GET" });
  }

  async putProviderCredentials(
    provider: string,
    payload: PutProviderCredentialsRequest,
  ): Promise<PutProviderCredentialsResult> {
    return this.request<PutProviderCredentialsResult>(
      `/internal/v1/providers/${encodeURIComponent(provider)}/credentials`,
      {
        method: "PUT",
        body: JSON.stringify(payload),
      },
    );
  }

  async putProviderPaymentMethod(
    provider: string,
    payload: PutProviderPaymentMethodRequest,
  ): Promise<PutProviderPaymentMethodResult> {
    return this.request<PutProviderPaymentMethodResult>(
      `/internal/v1/providers/${encodeURIComponent(provider)}/payment-method`,
      {
        method: "PUT",
        body: JSON.stringify(payload),
      },
    );
  }

  async createInvite(payload: CreateInviteRequest): Promise<CreateInviteResponse> {
    return this.request<CreateInviteResponse>("/internal/v1/auth/invites", {
      method: "POST",
      body: JSON.stringify(payload),
    });
  }

  private async request<T>(path: string, init: RequestInit): Promise<T> {
    const headers = new Headers(init.headers);
    headers.set("accept", "application/json");
    headers.set("x-bominal-service-token", this.serviceToken);
    if (init.body && !headers.has("content-type")) {
      headers.set("content-type", "application/json");
    }

    const response = await this.fetchImpl(`${this.baseUrl}${path}`, {
      ...init,
      headers,
    });

    if (response.status === 204) {
      return undefined as T;
    }

    const text = await response.text();
    const parsed = text ? (JSON.parse(text) as unknown) : null;
    if (!response.ok) {
      throw new InternalApiError(
        response.status,
        isApiError(parsed) ? parsed : null,
        `request failed with status ${response.status}`,
      );
    }

    return parsed as T;
  }
}

function isApiError(value: unknown): value is ApiError {
  if (!value || typeof value !== "object") {
    return false;
  }
  const maybe = value as Partial<ApiError>;
  return typeof maybe.message === "string";
}
