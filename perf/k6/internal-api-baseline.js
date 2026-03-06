import http from "k6/http";
import { check, sleep } from "k6";

const baseUrl = (__ENV.K6_BASE_URL || "").replace(/\/+$/, "");
const serviceToken = __ENV.K6_SERVICE_TOKEN || "";
const adminCookie = __ENV.K6_ADMIN_COOKIE || "";

if (!baseUrl) {
  throw new Error("K6_BASE_URL is required");
}
if (!serviceToken) {
  throw new Error("K6_SERVICE_TOKEN is required");
}

export const options = {
  vus: Number(__ENV.K6_VUS || 3),
  iterations: Number(__ENV.K6_ITERATIONS || 30),
  thresholds: {
    http_req_failed: ["rate<0.10"],
    http_req_duration: ["p(95)<2500"],
    checks: ["rate>0.90"],
  },
};

export default function () {
  const id = `perf-${Date.now()}-${__VU}-${__ITER}`;
  const jobId = createProviderJob(id);
  if (jobId) {
    getProviderJob(jobId);
    listProviderJobEvents(jobId);
  }
  putProviderCredentials(id);
  putProviderPaymentMethod(id);
  getObservabilityTimeseries();
  getAdminRuntimeJobsSse();
  sleep(0.2);
}

function createProviderJob(id) {
  const payload = {
    provider: "srt",
    operation: "search",
    idempotency_key: `${id}-job`,
    payload: {
      user_id: `perf-user-${id}`,
      route: {
        departure: "NAT010000",
        arrival: "NAT014445",
      },
    },
  };

  const res = http.post(
    `${baseUrl}/internal/v1/provider-jobs`,
    JSON.stringify(payload),
    {
      headers: internalHeaders(),
      tags: { endpoint: "internal_provider_jobs_create" },
    },
  );

  check(res, {
    "provider-job create accepted": (r) => r.status === 202,
  });

  const body = safeJson(res.body);
  return body && typeof body.job_id === "string" ? body.job_id : null;
}

function getProviderJob(jobId) {
  const res = http.get(
    `${baseUrl}/internal/v1/provider-jobs/${encodeURIComponent(jobId)}`,
    {
      headers: internalHeaders(),
      tags: { endpoint: "internal_provider_jobs_get" },
    },
  );

  check(res, {
    "provider-job get ok": (r) => r.status === 200,
  });
}

function listProviderJobEvents(jobId) {
  const res = http.get(
    `${baseUrl}/internal/v1/provider-jobs/${encodeURIComponent(jobId)}/events?limit=20`,
    {
      headers: internalHeaders(),
      tags: { endpoint: "internal_provider_jobs_events" },
    },
  );

  check(res, {
    "provider-job events ok": (r) => r.status === 200,
  });
}

function putProviderCredentials(id) {
  const payload = {
    subject_ref: `perf-subject-${id}`,
    identity_ciphertext: "perf_identity_ciphertext",
    password_ciphertext: "perf_password_ciphertext",
  };

  const res = http.put(
    `${baseUrl}/internal/v1/providers/srt/credentials`,
    JSON.stringify(payload),
    {
      headers: internalHeaders(),
      tags: { endpoint: "internal_provider_credentials_put" },
    },
  );

  check(res, {
    "provider credentials accepted": (r) => r.status === 202,
  });
}

function putProviderPaymentMethod(id) {
  const payload = {
    owner_ref: `perf-owner-${id}`,
    payment_method_ref: `pm_perf_${id}`,
    card_brand: "visa",
    card_last4: "4242",
    pan_ciphertext: "perf_pan_ciphertext",
    expiry_month_ciphertext: "perf_expiry_month_ciphertext",
    expiry_year_ciphertext: "perf_expiry_year_ciphertext",
    birth_or_business_number_ciphertext: "perf_birth_ciphertext",
    card_password_two_digits_ciphertext: "perf_pwd2_ciphertext",
  };

  const res = http.put(
    `${baseUrl}/internal/v1/providers/srt/payment-method`,
    JSON.stringify(payload),
    {
      headers: internalHeaders(),
      tags: { endpoint: "internal_provider_payment_method_put" },
    },
  );

  check(res, {
    "provider payment-method accepted": (r) => r.status === 202,
  });
}

function getObservabilityTimeseries() {
  if (!adminCookie) {
    return;
  }
  const res = http.get(
    `${baseUrl}/api/admin/observability/timeseries?window=1h&resolution=5m`,
    {
      headers: adminHeaders(),
      tags: { endpoint: "admin_observability_timeseries" },
    },
  );

  check(res, {
    "observability timeseries ok": (r) => r.status === 200,
  });
}

function getAdminRuntimeJobsSse() {
  if (!adminCookie) {
    return;
  }
  const res = http.get(`${baseUrl}/api/admin/runtime/jobs/stream?limit=20`, {
    headers: {
      ...adminHeaders(),
      accept: "text/event-stream",
    },
    timeout: "8s",
    tags: { endpoint: "admin_runtime_jobs_sse" },
  });

  check(res, {
    "admin jobs sse endpoint reachable": (r) => r.status === 200,
  });
}

function internalHeaders() {
  return {
    "content-type": "application/json",
    accept: "application/json",
    "x-internal-service-token": serviceToken,
  };
}

function adminHeaders() {
  return {
    accept: "application/json",
    cookie: adminCookie,
  };
}

function safeJson(value) {
  if (!value) {
    return null;
  }
  try {
    return JSON.parse(value);
  } catch (_error) {
    return null;
  }
}
