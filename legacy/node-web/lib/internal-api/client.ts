import { clientApiBaseUrl } from "@/lib/api-base";

import type {
  InternalApiAuthHeaders,
  InternalApiErrorEnvelope,
  InternalProviderCredentialsUpsertRequest,
  InternalProviderCredentialsUpsertResponse,
  InternalProviderJob,
  InternalProviderJobCreateRequest,
  InternalProviderJobCreateResponse,
  InternalProviderJobEventsQuery,
  InternalProviderJobEventsResponse,
  InternalProviderPaymentMethodUpsertRequest,
  InternalProviderPaymentMethodUpsertResponse,
} from "./types";

const CANONICAL_ENDPOINTS = {
  providerCredentials: "/internal/v1/providers/srt/credentials",
  providerPaymentMethod: "/internal/v1/providers/srt/payment-method",
  providerJobs: "/internal/v1/provider-jobs",
} as const;

const COMPATIBILITY_ALIAS_ENDPOINTS = {
  providerCredentials: "/api/internal/providers/srt/credentials",
  providerPaymentMethod: "/api/internal/providers/srt/payment-method",
  providerJobs: "/api/internal/provider-jobs",
} as const;

type EndpointKey = keyof typeof CANONICAL_ENDPOINTS;

type InternalApiRuntimeEnv = {
  NODE_ENV?: string;
  NEXT_PUBLIC_INTERNAL_DEBUG_MODE?: string;
};

export type InternalApiClientOptions = {
  baseUrl?: string;
  auth?: InternalApiAuthHeaders;
  preferCompatibilityAlias?: boolean;
  fetchImpl?: typeof fetch;
};

export type InternalApiCompatibilityMode = "canonical" | "compatibility_alias";

export class InternalApiClientError extends Error {
  status: number;
  code: string | null;
  requestId: string | null;
  details: Record<string, unknown> | null;

  constructor(message: string, params: { status: number; code: string | null; requestId: string | null; details: Record<string, unknown> | null }) {
    super(message);
    this.name = "InternalApiClientError";
    this.status = params.status;
    this.code = params.code;
    this.requestId = params.requestId;
    this.details = params.details;
  }
}

function trimTrailingSlash(value: string): string {
  return value.replace(/\/+$/, "");
}

function normalizeBooleanEnv(value: string | undefined): boolean {
  return String(value ?? "")
    .trim()
    .toLowerCase() === "true";
}

// Compatibility aliases are adapter-only and can never be active in production builds.
export function isInternalCompatibilityAliasEnabled(env: InternalApiRuntimeEnv = process.env): boolean {
  return String(env.NODE_ENV ?? "").toLowerCase() !== "production" && normalizeBooleanEnv(env.NEXT_PUBLIC_INTERNAL_DEBUG_MODE);
}

export function resolveInternalEndpointPath(
  endpoint: EndpointKey,
  options: { preferCompatibilityAlias?: boolean; env?: InternalApiRuntimeEnv } = {},
): { path: string; mode: InternalApiCompatibilityMode } {
  const useCompatibilityAlias =
    Boolean(options.preferCompatibilityAlias) && isInternalCompatibilityAliasEnabled(options.env ?? process.env);

  if (useCompatibilityAlias) {
    return { path: COMPATIBILITY_ALIAS_ENDPOINTS[endpoint], mode: "compatibility_alias" };
  }

  return { path: CANONICAL_ENDPOINTS[endpoint], mode: "canonical" };
}

function toInternalErrorEnvelope(payload: unknown): InternalApiErrorEnvelope | null {
  if (!payload || typeof payload !== "object") {
    return null;
  }
  return payload as InternalApiErrorEnvelope;
}

async function parseFailureMessage(response: Response): Promise<{ message: string; envelope: InternalApiErrorEnvelope | null }> {
  const contentType = response.headers.get("content-type") ?? "";
  if (contentType.includes("application/json")) {
    const payload = await response.json().catch(() => null);
    const envelope = toInternalErrorEnvelope(payload);
    if (envelope?.message) {
      return { message: envelope.message, envelope };
    }
    if (envelope?.detail) {
      return { message: envelope.detail, envelope };
    }
    return { message: `request failed (${response.status})`, envelope };
  }

  const text = (await response.text().catch(() => "")).trim();
  if (text) {
    return { message: text, envelope: null };
  }
  return { message: `request failed (${response.status})`, envelope: null };
}

function buildAuthHeaders(auth?: InternalApiAuthHeaders): Record<string, string> {
  const headers: Record<string, string> = {};
  const apiKey = String(auth?.internalApiKey ?? "").trim();
  const serviceToken = String(auth?.internalServiceToken ?? "").trim();

  if (apiKey) {
    headers["X-Internal-Api-Key"] = apiKey;
  }
  if (serviceToken) {
    headers["X-Internal-Service-Token"] = serviceToken;
  }

  return headers;
}

function assertCiphertextOnlyPaymentPayload(payload: InternalProviderPaymentMethodUpsertRequest): void {
  const unsafe = payload as Record<string, unknown>;
  const forbiddenPlaintextFields = ["card_number", "pin2", "birth_date", "expiry_month", "expiry_year", "expiry"];

  for (const field of forbiddenPlaintextFields) {
    if (field in unsafe) {
      throw new Error(`plaintext payment field is forbidden for internal API: ${field}`);
    }
  }
}

function buildEventsPath(basePath: string, jobId: string, query: InternalProviderJobEventsQuery = {}): string {
  const params = new URLSearchParams();
  if (query.cursor) {
    params.set("cursor", query.cursor);
  }
  if (typeof query.limit === "number" && Number.isFinite(query.limit) && query.limit > 0) {
    params.set("limit", String(Math.floor(query.limit)));
  }
  const suffix = params.toString();
  return suffix ? `${basePath}/${encodeURIComponent(jobId)}/events?${suffix}` : `${basePath}/${encodeURIComponent(jobId)}/events`;
}

export function createInternalApiClient(options: InternalApiClientOptions = {}) {
  const fetchImpl = options.fetchImpl ?? fetch;
  const baseUrl = trimTrailingSlash(options.baseUrl ?? clientApiBaseUrl);

  async function requestJson<TResponse>(path: string, init?: RequestInit): Promise<TResponse> {
    const response = await fetchImpl(`${baseUrl}${path}`, {
      ...init,
      credentials: "include",
      cache: "no-store",
      headers: {
        ...buildAuthHeaders(options.auth),
        ...(init?.headers ?? {}),
      },
    });

    if (!response.ok) {
      const { message, envelope } = await parseFailureMessage(response);
      throw new InternalApiClientError(message, {
        status: response.status,
        code: envelope?.code ?? null,
        requestId: envelope?.request_id ?? null,
        details: envelope?.details ?? null,
      });
    }

    return (await response.json()) as TResponse;
  }

  const credentialsPath = resolveInternalEndpointPath("providerCredentials", {
    preferCompatibilityAlias: options.preferCompatibilityAlias,
  }).path;
  const paymentMethodPath = resolveInternalEndpointPath("providerPaymentMethod", {
    preferCompatibilityAlias: options.preferCompatibilityAlias,
  }).path;
  const providerJobsPath = resolveInternalEndpointPath("providerJobs", {
    preferCompatibilityAlias: options.preferCompatibilityAlias,
  }).path;

  return {
    compatibilityMode: resolveInternalEndpointPath("providerJobs", {
      preferCompatibilityAlias: options.preferCompatibilityAlias,
    }).mode,

    upsertProviderCredentials(payload: InternalProviderCredentialsUpsertRequest): Promise<InternalProviderCredentialsUpsertResponse> {
      return requestJson<InternalProviderCredentialsUpsertResponse>(credentialsPath, {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(payload),
      });
    },

    upsertProviderPaymentMethod(payload: InternalProviderPaymentMethodUpsertRequest): Promise<InternalProviderPaymentMethodUpsertResponse> {
      assertCiphertextOnlyPaymentPayload(payload);
      return requestJson<InternalProviderPaymentMethodUpsertResponse>(paymentMethodPath, {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(payload),
      });
    },

    createProviderJob(payload: InternalProviderJobCreateRequest): Promise<InternalProviderJobCreateResponse> {
      return requestJson<InternalProviderJobCreateResponse>(providerJobsPath, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(payload),
      });
    },

    getProviderJob(jobId: string): Promise<InternalProviderJob> {
      return requestJson<InternalProviderJob>(`${providerJobsPath}/${encodeURIComponent(jobId)}`);
    },

    listProviderJobEvents(jobId: string, query?: InternalProviderJobEventsQuery): Promise<InternalProviderJobEventsResponse> {
      return requestJson<InternalProviderJobEventsResponse>(buildEventsPath(providerJobsPath, jobId, query));
    },
  };
}
