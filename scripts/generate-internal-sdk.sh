#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OPENAPI_FILE="${REPO_ROOT}/sdk/openapi/bominal-internal.v1.json"
OUTPUT_DIR="${REPO_ROOT}/sdk/ts/internal"
GENERATED_FILE="${OUTPUT_DIR}/generated.ts"
INDEX_FILE="${OUTPUT_DIR}/index.ts"

MODE="${1:-generate}"
if [[ "${MODE}" != "generate" && "${MODE}" != "--check" ]]; then
  echo "usage: scripts/generate-internal-sdk.sh [--check]" >&2
  exit 2
fi

python3 - <<'PY' "${OPENAPI_FILE}"
import json
import sys

path = sys.argv[1]
with open(path, "r", encoding="utf-8") as handle:
    spec = json.load(handle)

if spec.get("openapi") != "3.1.0":
    raise SystemExit(f"unexpected OpenAPI version in {path}")

if spec.get("info", {}).get("version") != "v1":
    raise SystemExit(f"unexpected internal schema version in {path}")

security_scheme = (
    spec.get("components", {})
    .get("securitySchemes", {})
    .get("InternalServiceToken", {})
)
if security_scheme.get("name") != "x-internal-service-token":
    raise SystemExit(
        "unexpected InternalServiceToken header name in "
        f"{path}: {security_scheme.get('name')!r}"
    )
PY

mkdir -p "${OUTPUT_DIR}"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "${TMP_DIR}"' EXIT

cat > "${TMP_DIR}/generated.ts" <<'TS'
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
    headers.set("x-internal-service-token", this.serviceToken);
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
TS

cat > "${TMP_DIR}/index.ts" <<'TS'
export * from "./generated";
TS

if [[ "${MODE}" == "--check" ]]; then
  if ! cmp -s "${TMP_DIR}/generated.ts" "${GENERATED_FILE}"; then
    echo "internal SDK generated file drift: ${GENERATED_FILE}" >&2
    diff -u "${GENERATED_FILE}" "${TMP_DIR}/generated.ts" || true
    exit 1
  fi
  if ! cmp -s "${TMP_DIR}/index.ts" "${INDEX_FILE}"; then
    echo "internal SDK generated file drift: ${INDEX_FILE}" >&2
    diff -u "${INDEX_FILE}" "${TMP_DIR}/index.ts" || true
    exit 1
  fi
  exit 0
fi

cp "${TMP_DIR}/generated.ts" "${GENERATED_FILE}"
cp "${TMP_DIR}/index.ts" "${INDEX_FILE}"
