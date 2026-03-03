"use client";

import { FormEvent, useMemo, useState } from "react";

import { InternalApiClientError, createInternalApiClient, isInternalCompatibilityAliasEnabled } from "@/lib/internal-api/client";
import { UI_BUTTON_PRIMARY, UI_CARD_MD, UI_FIELD, UI_KICKER, UI_TITLE_MD } from "@/lib/ui";

export function CredentialsForm() {
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [internalApiKey, setInternalApiKey] = useState("");
  const [internalServiceToken, setInternalServiceToken] = useState("");
  const [preferAlias, setPreferAlias] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

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

  const onSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setSubmitting(true);
    setError(null);
    setNotice(null);

    try {
      const payload = await client.upsertProviderCredentials({
        username: username.trim(),
        password,
      });
      setNotice(
        payload.verified
          ? `Credentials verified (${payload.credential_status}) via ${client.compatibilityMode}.`
          : `Credentials saved (${payload.credential_status}) via ${client.compatibilityMode}.`,
      );
    } catch (err) {
      if (err instanceof InternalApiClientError) {
        setError(`${err.message}${err.requestId ? ` (request_id: ${err.requestId})` : ""}`);
      } else {
        setError("Could not submit credentials.");
      }
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <section className={UI_CARD_MD}>
      <p className={UI_KICKER}>internal</p>
      <h2 className={`mt-1 ${UI_TITLE_MD}`}>SRT credentials</h2>
      <p className="mt-2 text-sm text-slate-600">PUT /internal/v1/providers/srt/credentials</p>

      {error ? <p className="mt-3 rounded-xl bg-rose-50 px-3 py-2 text-sm text-rose-700">{error}</p> : null}
      {notice ? <p className="mt-3 rounded-xl bg-emerald-50 px-3 py-2 text-sm text-emerald-700">{notice}</p> : null}

      <form onSubmit={onSubmit} className="mt-4 grid gap-3">
        <label className="text-sm text-slate-700">
          Username
          <input
            className={`mt-1 ${UI_FIELD}`}
            autoComplete="username"
            value={username}
            onChange={(event) => setUsername(event.target.value)}
            required
          />
        </label>

        <label className="text-sm text-slate-700">
          Password
          <input
            type="password"
            className={`mt-1 ${UI_FIELD}`}
            autoComplete="current-password"
            value={password}
            onChange={(event) => setPassword(event.target.value)}
            required
          />
        </label>

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
          <textarea
            className="mt-1 min-h-24 w-full rounded-2xl border border-blossom-200 bg-white px-3 py-2 text-sm text-slate-700 shadow-sm outline-none transition focus:border-blossom-300 focus:ring-2 focus:ring-blossom-100"
            value={internalServiceToken}
            onChange={(event) => setInternalServiceToken(event.target.value)}
          />
        </label>

        {aliasModeAllowed ? (
          <label className="inline-flex items-center gap-2 text-sm text-slate-700">
            <input
              type="checkbox"
              className="h-4 w-4 rounded border-slate-300"
              checked={preferAlias}
              onChange={(event) => setPreferAlias(event.target.checked)}
            />
            Use compatibility alias adapters (non-production + debug mode only)
          </label>
        ) : null}

        <div>
          <button type="submit" className={UI_BUTTON_PRIMARY} disabled={submitting}>
            {submitting ? "Submitting..." : "Submit credentials"}
          </button>
        </div>
      </form>
    </section>
  );
}
