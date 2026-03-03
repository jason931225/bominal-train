"use client";

import { FormEvent, useMemo, useState } from "react";

import { InternalApiClientError, createInternalApiClient, isInternalCompatibilityAliasEnabled } from "@/lib/internal-api/client";
import type { InternalProviderJob, InternalProviderJobEventsResponse } from "@/lib/internal-api/types";
import { UI_BUTTON_OUTLINE, UI_BUTTON_PRIMARY, UI_CARD_MD, UI_FIELD, UI_KICKER, UI_TITLE_MD } from "@/lib/ui";

function safeJsonParse(value: string): Record<string, unknown> {
  const parsed = JSON.parse(value) as unknown;
  if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
    throw new Error("Payload must be a JSON object.");
  }
  return parsed as Record<string, unknown>;
}

export function ProviderJobConsole() {
  const [kind, setKind] = useState("srt.search_train");
  const [payloadJson, setPayloadJson] = useState('{"example": true}');
  const [jobId, setJobId] = useState("");
  const [eventsLimit, setEventsLimit] = useState("50");

  const [internalApiKey, setInternalApiKey] = useState("");
  const [internalServiceToken, setInternalServiceToken] = useState("");
  const [preferAlias, setPreferAlias] = useState(false);

  const [busyAction, setBusyAction] = useState<"create" | "status" | "events" | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [jobSnapshot, setJobSnapshot] = useState<InternalProviderJob | null>(null);
  const [eventsSnapshot, setEventsSnapshot] = useState<InternalProviderJobEventsResponse | null>(null);

  const aliasModeAllowed = isInternalCompatibilityAliasEnabled();

  const client = useMemo(
    () =>
      createInternalApiClient({
        auth: {
          internalApiKey,
          internalServiceToken,
        },
        preferCompatibilityAlias: preferAlias,
      }),
    [internalApiKey, internalServiceToken, preferAlias],
  );

  const setClientError = (err: unknown, fallback: string) => {
    if (err instanceof InternalApiClientError) {
      setError(`${err.message}${err.requestId ? ` (request_id: ${err.requestId})` : ""}`);
      return;
    }
    if (err instanceof Error) {
      setError(err.message);
      return;
    }
    setError(fallback);
  };

  const onCreateJob = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setBusyAction("create");
    setError(null);
    setNotice(null);

    try {
      const payload = safeJsonParse(payloadJson);
      const response = await client.createProviderJob({
        provider: "srt",
        kind,
        payload,
      });
      setJobId(response.job_id);
      setNotice(`Job queued as ${response.job_id} via ${client.compatibilityMode}.`);
    } catch (err) {
      setClientError(err, "Could not create provider job.");
    } finally {
      setBusyAction(null);
    }
  };

  const onLoadStatus = async () => {
    if (!jobId.trim()) {
      setError("Job ID is required.");
      return;
    }

    setBusyAction("status");
    setError(null);
    setNotice(null);

    try {
      const response = await client.getProviderJob(jobId.trim());
      setJobSnapshot(response);
      setNotice(`Loaded job status (${response.status}) via ${client.compatibilityMode}.`);
    } catch (err) {
      setClientError(err, "Could not load job status.");
    } finally {
      setBusyAction(null);
    }
  };

  const onLoadEvents = async () => {
    if (!jobId.trim()) {
      setError("Job ID is required.");
      return;
    }

    setBusyAction("events");
    setError(null);
    setNotice(null);

    try {
      const limit = Number(eventsLimit);
      const response = await client.listProviderJobEvents(jobId.trim(), {
        limit: Number.isFinite(limit) && limit > 0 ? limit : undefined,
      });
      setEventsSnapshot(response);
      setNotice(`Loaded ${response.events.length} job events via ${client.compatibilityMode}.`);
    } catch (err) {
      setClientError(err, "Could not load job events.");
    } finally {
      setBusyAction(null);
    }
  };

  return (
    <section className={UI_CARD_MD}>
      <p className={UI_KICKER}>internal</p>
      <h2 className={`mt-1 ${UI_TITLE_MD}`}>Provider job console</h2>
      <p className="mt-2 text-sm text-slate-600">POST/GET /internal/v1/provider-jobs + /events</p>

      {error ? <p className="mt-3 rounded-xl bg-rose-50 px-3 py-2 text-sm text-rose-700">{error}</p> : null}
      {notice ? <p className="mt-3 rounded-xl bg-emerald-50 px-3 py-2 text-sm text-emerald-700">{notice}</p> : null}

      <div className="mt-4 grid gap-3 md:grid-cols-2">
        <label className="text-sm text-slate-700">
          `X-Internal-Api-Key` (optional)
          <input
            type="password"
            className={`mt-1 ${UI_FIELD}`}
            value={internalApiKey}
            onChange={(event) => setInternalApiKey(event.target.value)}
          />
        </label>

        <label className="text-sm text-slate-700">
          `X-Internal-Service-Token` (optional)
          <input
            type="password"
            className={`mt-1 ${UI_FIELD}`}
            value={internalServiceToken}
            onChange={(event) => setInternalServiceToken(event.target.value)}
          />
        </label>

        {aliasModeAllowed ? (
          <label className="inline-flex items-center gap-2 text-sm text-slate-700 md:col-span-2">
            <input
              type="checkbox"
              className="h-4 w-4 rounded border-slate-300"
              checked={preferAlias}
              onChange={(event) => setPreferAlias(event.target.checked)}
            />
            Use compatibility alias adapters (non-production + debug mode only)
          </label>
        ) : null}
      </div>

      <form onSubmit={onCreateJob} className="mt-4 grid gap-3">
        <label className="text-sm text-slate-700">
          Job kind
          <input className={`mt-1 ${UI_FIELD}`} value={kind} onChange={(event) => setKind(event.target.value)} required />
        </label>

        <label className="text-sm text-slate-700">
          Payload JSON
          <textarea
            className="mt-1 min-h-28 w-full rounded-2xl border border-blossom-200 bg-white px-3 py-2 font-mono text-xs text-slate-700 shadow-sm outline-none transition focus:border-blossom-300 focus:ring-2 focus:ring-blossom-100"
            value={payloadJson}
            onChange={(event) => setPayloadJson(event.target.value)}
          />
        </label>

        <div>
          <button type="submit" className={UI_BUTTON_PRIMARY} disabled={busyAction !== null}>
            {busyAction === "create" ? "Queueing..." : "Create provider job"}
          </button>
        </div>
      </form>

      <div className="mt-5 grid gap-3 md:grid-cols-[2fr_1fr]">
        <label className="text-sm text-slate-700">
          Job ID
          <input className={`mt-1 ${UI_FIELD}`} value={jobId} onChange={(event) => setJobId(event.target.value)} />
        </label>

        <label className="text-sm text-slate-700">
          Events limit
          <input
            className={`mt-1 ${UI_FIELD}`}
            inputMode="numeric"
            value={eventsLimit}
            onChange={(event) => setEventsLimit(event.target.value)}
          />
        </label>
      </div>

      <div className="mt-3 flex flex-wrap gap-2">
        <button type="button" className={UI_BUTTON_OUTLINE} disabled={busyAction !== null} onClick={() => void onLoadStatus()}>
          {busyAction === "status" ? "Loading status..." : "Get job status"}
        </button>
        <button type="button" className={UI_BUTTON_OUTLINE} disabled={busyAction !== null} onClick={() => void onLoadEvents()}>
          {busyAction === "events" ? "Loading events..." : "Get job events"}
        </button>
      </div>

      {jobSnapshot ? (
        <div className="mt-4">
          <p className="text-xs font-medium uppercase tracking-[0.16em] text-slate-500">Job snapshot</p>
          <pre className="mt-2 overflow-x-auto rounded-2xl bg-slate-950/90 p-3 text-xs text-slate-100">
            {JSON.stringify(jobSnapshot, null, 2)}
          </pre>
        </div>
      ) : null}

      {eventsSnapshot ? (
        <div className="mt-4">
          <p className="text-xs font-medium uppercase tracking-[0.16em] text-slate-500">Event snapshot</p>
          <pre className="mt-2 max-h-72 overflow-auto rounded-2xl bg-slate-950/90 p-3 text-xs text-slate-100">
            {JSON.stringify(eventsSnapshot, null, 2)}
          </pre>
        </div>
      ) : null}
    </section>
  );
}
