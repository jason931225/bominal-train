import "jsr:@supabase/functions-js/edge-runtime.d.ts";

type NotifyPayload = {
  to_email?: unknown;
  subject?: unknown;
  text_body?: unknown;
  html_body?: unknown;
  tags?: unknown;
  headers?: unknown;
  metadata?: unknown;
  message_id?: unknown;
  idempotency_key?: unknown;
};

function json(status: number, body: Record<string, unknown>): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { "Content-Type": "application/json" },
  });
}

function readString(value: unknown): string | null {
  if (typeof value !== "string") return null;
  const normalized = value.trim();
  return normalized.length > 0 ? normalized : null;
}

function normalizeTags(value: unknown): Array<{ name: string; value: string }> {
  if (!Array.isArray(value)) return [];
  const rows: Array<{ name: string; value: string }> = [];
  for (const row of value) {
    if (typeof row !== "object" || row === null || Array.isArray(row)) continue;
    const name = readString((row as Record<string, unknown>).name);
    const tagValue = readString((row as Record<string, unknown>).value);
    if (!name || !tagValue) continue;
    rows.push({ name, value: tagValue });
  }
  return rows.slice(0, 10);
}

Deno.serve(async (req: Request) => {
  if (req.method !== "POST") {
    return json(405, { ok: false, error: "method_not_allowed" });
  }

  const resendApiKey = readString(Deno.env.get("RESEND_API_KEY"));
  const fromAddress = readString(Deno.env.get("EMAIL_FROM_ADDRESS"));
  const fromName = readString(Deno.env.get("EMAIL_FROM_NAME")) ?? "bominal";
  if (!resendApiKey || !fromAddress) {
    return json(500, { ok: false, error: "misconfigured" });
  }

  const payload = (await req.json().catch(() => null)) as NotifyPayload | null;
  if (!payload) {
    return json(400, { ok: false, error: "invalid_json" });
  }

  const toEmail = readString(payload.to_email);
  const subject = readString(payload.subject);
  const textBody = readString(payload.text_body);
  const htmlBody = readString(payload.html_body);
  if (!toEmail || !subject || (!textBody && !htmlBody)) {
    return json(400, { ok: false, error: "invalid_payload" });
  }

  const resendHeaders: Record<string, string> = {
    Authorization: `Bearer ${resendApiKey}`,
    "Content-Type": "application/json",
  };
  const idempotencyKey = readString(payload.idempotency_key) ?? readString(payload.message_id);
  if (idempotencyKey) {
    resendHeaders["Idempotency-Key"] = idempotencyKey;
  }

  const body: Record<string, unknown> = {
    from: `${fromName} <${fromAddress}>`,
    to: [toEmail],
    subject,
    text: textBody ?? undefined,
    html: htmlBody ?? undefined,
    headers:
      typeof payload.headers === "object" && payload.headers !== null && !Array.isArray(payload.headers)
        ? payload.headers
        : undefined,
    metadata:
      typeof payload.metadata === "object" && payload.metadata !== null && !Array.isArray(payload.metadata)
        ? payload.metadata
        : undefined,
  };
  const tags = normalizeTags(payload.tags);
  if (tags.length > 0) {
    body.tags = tags;
  }

  const resendResponse = await fetch("https://api.resend.com/emails", {
    method: "POST",
    headers: resendHeaders,
    body: JSON.stringify(body),
  });

  if (!resendResponse.ok) {
    return json(502, {
      ok: false,
      error: "resend_rejected",
      status: resendResponse.status,
    });
  }

  const resendPayload = (await resendResponse.json().catch(() => null)) as { id?: unknown } | null;
  return json(200, {
    ok: true,
    provider_message_id: readString(resendPayload?.id) ?? null,
  });
});
