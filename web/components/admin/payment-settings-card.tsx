"use client";

import { FormEvent, useCallback, useEffect, useState } from "react";

import { clientApiBaseUrl } from "@/lib/api-base";
import {
  UI_BUTTON_OUTLINE,
  UI_BUTTON_OUTLINE_SM,
  UI_BUTTON_PRIMARY,
  UI_CARD_MD,
  UI_FIELD,
  UI_KICKER,
  UI_TITLE_MD,
} from "@/lib/ui";

type AdminPaymentSettingsResponse = {
  payment_enabled: boolean;
  payment_enabled_env: boolean;
  payment_enabled_override: boolean;
  configured: boolean;
  source: "server_override" | "pay_env" | "none";
  card_masked: string | null;
  updated_at: string | null;
  updated_by_user_id: string | null;
};

type PaymentFormState = {
  cardNumber: string;
  expiryMm: string;
  expiryYy: string;
  dob: string;
  pin2: string;
};

const EMPTY_FORM: PaymentFormState = {
  cardNumber: "",
  expiryMm: "",
  expiryYy: "",
  dob: "",
  pin2: "",
};

function sourceLabel(source: AdminPaymentSettingsResponse["source"]): string {
  if (source === "server_override") return "Admin override";
  if (source === "pay_env") return "pay.env fallback";
  return "Not configured";
}

async function parseApiDetail(response: Response, fallback: string): Promise<string> {
  const contentType = response.headers.get("content-type") ?? "";
  if (contentType.includes("application/json")) {
    const payload = (await response.json().catch(() => null)) as { detail?: string } | null;
    if (payload?.detail) return payload.detail;
  }
  return fallback;
}

export function PaymentSettingsCard() {
  const [status, setStatus] = useState<AdminPaymentSettingsResponse | null>(null);
  const [form, setForm] = useState<PaymentFormState>(EMPTY_FORM);
  const [loading, setLoading] = useState(true);
  const [submitting, setSubmitting] = useState(false);
  const [removing, setRemoving] = useState(false);
  const [toggling, setToggling] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const response = await fetch(`${clientApiBaseUrl}/api/admin/payment-settings`, {
        credentials: "include",
        cache: "no-store",
      });
      if (!response.ok) {
        setError(await parseApiDetail(response, "Failed to load payment settings"));
        return;
      }
      const payload = (await response.json()) as AdminPaymentSettingsResponse;
      setStatus(payload);
    } catch {
      setError("Failed to load payment settings");
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  const onSaveCard = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setSubmitting(true);
    setError(null);
    setNotice(null);
    try {
      const response = await fetch(`${clientApiBaseUrl}/api/admin/payment-settings/card`, {
        method: "PUT",
        credentials: "include",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          card_number: form.cardNumber,
          expiry_mm: form.expiryMm,
          expiry_yy: form.expiryYy,
          dob: form.dob,
          pin2: form.pin2,
        }),
      });
      if (!response.ok) {
        setError(await parseApiDetail(response, "Failed to update payment card"));
        return;
      }
      const payload = (await response.json()) as AdminPaymentSettingsResponse;
      setStatus(payload);
      setForm(EMPTY_FORM);
      setNotice("Server-wide payment settings updated.");
    } catch {
      setError("Failed to update payment card");
    } finally {
      setSubmitting(false);
    }
  };

  const onClearCard = async () => {
    if (!window.confirm("Remove admin override card and fall back to pay.env?")) {
      return;
    }

    setRemoving(true);
    setError(null);
    setNotice(null);
    try {
      const response = await fetch(`${clientApiBaseUrl}/api/admin/payment-settings/card`, {
        method: "DELETE",
        credentials: "include",
      });
      if (!response.ok) {
        setError(await parseApiDetail(response, "Failed to clear payment card override"));
        return;
      }
      const payload = (await response.json()) as AdminPaymentSettingsResponse;
      setStatus(payload);
      setNotice("Server-wide payment override removed.");
    } catch {
      setError("Failed to clear payment card override");
    } finally {
      setRemoving(false);
    }
  };

  const onToggleEnabled = async () => {
    if (!status) return;

    const nextEnabled = !status.payment_enabled_override;
    const prompt = nextEnabled
      ? "Enable payment actions globally?"
      : "Disable payment actions globally?";
    if (!window.confirm(prompt)) {
      return;
    }

    setToggling(true);
    setError(null);
    setNotice(null);
    try {
      const response = await fetch(`${clientApiBaseUrl}/api/admin/payment-settings/enabled`, {
        method: "PATCH",
        credentials: "include",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ enabled: nextEnabled }),
      });
      if (!response.ok) {
        setError(await parseApiDetail(response, "Failed to update payment toggle"));
        return;
      }
      const payload = (await response.json()) as AdminPaymentSettingsResponse;
      setStatus(payload);
      setNotice(nextEnabled ? "Payments enabled." : "Payments disabled.");
    } catch {
      setError("Failed to update payment toggle");
    } finally {
      setToggling(false);
    }
  };

  return (
    <section className={UI_CARD_MD}>
      <div className="flex items-start justify-between gap-3">
        <div>
          <p className={UI_KICKER}>Payment</p>
          <h2 className={`mt-2 ${UI_TITLE_MD}`}>Server-wide Payment Settings</h2>
          <p className="mt-2 text-sm text-slate-500">
            All train payments use backend server settings.
          </p>
        </div>
        <button
          type="button"
          className={UI_BUTTON_OUTLINE_SM}
          onClick={() => void load()}
          disabled={loading || submitting || removing || toggling}
        >
          Refresh
        </button>
      </div>

      {error ? <p className="mt-4 rounded-xl bg-rose-50 px-3 py-2 text-sm text-rose-700">{error}</p> : null}
      {notice ? <p className="mt-4 rounded-xl bg-emerald-50 px-3 py-2 text-sm text-emerald-700">{notice}</p> : null}

      {loading ? (
        <p className="mt-4 text-sm text-slate-500">Loading...</p>
      ) : status ? (
        <>
          <div className="mt-5 grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
            <div className="rounded-xl border border-blossom-100 bg-white p-3">
              <p className="text-xs uppercase tracking-wide text-slate-400">Payment Status</p>
              <p className="mt-1 text-sm font-semibold text-slate-800">
                {status.payment_enabled ? "Enabled" : "Disabled"}
              </p>
            </div>
            <div className="rounded-xl border border-blossom-100 bg-white p-3">
              <p className="text-xs uppercase tracking-wide text-slate-400">Runtime Toggle</p>
              <p className="mt-1 text-sm font-semibold text-slate-800">
                {status.payment_enabled_override ? "Enabled" : "Disabled"}
              </p>
            </div>
            <div className="rounded-xl border border-blossom-100 bg-white p-3">
              <p className="text-xs uppercase tracking-wide text-slate-400">Card Source</p>
              <p className="mt-1 text-sm font-semibold text-slate-800">{sourceLabel(status.source)}</p>
            </div>
            <div className="rounded-xl border border-blossom-100 bg-white p-3">
              <p className="text-xs uppercase tracking-wide text-slate-400">Card</p>
              <p className="mt-1 text-sm font-semibold text-slate-800">{status.card_masked ?? "Not set"}</p>
            </div>
          </div>

          <div className="mt-4 flex flex-wrap gap-2">
            <button
              type="button"
              onClick={() => void onToggleEnabled()}
              disabled={loading || submitting || removing || toggling || !status.payment_enabled_env}
              className={`${UI_BUTTON_OUTLINE} ${
                status.payment_enabled_override
                  ? "border-rose-200 text-rose-700 hover:bg-rose-50"
                  : "border-emerald-200 text-emerald-700 hover:bg-emerald-50"
              }`}
            >
              {toggling
                ? "Updating..."
                : status.payment_enabled_override
                  ? "Disable Payments"
                  : "Enable Payments"}
            </button>
            {!status.payment_enabled_env ? (
              <p className="self-center text-xs text-amber-700">
                `PAYMENT_ENABLED=false` in environment overrides runtime toggle.
              </p>
            ) : null}
          </div>

          <form onSubmit={onSaveCard} className="mt-5 grid gap-3 md:grid-cols-2">
            <label className="text-sm text-slate-700">
              Card Number
              <input
                type="text"
                inputMode="numeric"
                value={form.cardNumber}
                onChange={(event) => setForm((current) => ({ ...current, cardNumber: event.target.value }))}
                className={`mt-1 ${UI_FIELD}`}
                placeholder="1234 5678 9012 3456"
                required
              />
            </label>

            <div className="grid grid-cols-2 gap-2">
              <label className="text-sm text-slate-700">
                EXPIRYMM
                <input
                  type="text"
                  inputMode="numeric"
                  maxLength={2}
                  value={form.expiryMm}
                  onChange={(event) => setForm((current) => ({ ...current, expiryMm: event.target.value }))}
                  className={`mt-1 ${UI_FIELD}`}
                  placeholder="MM"
                  required
                />
              </label>
              <label className="text-sm text-slate-700">
                EXPIRYYY
                <input
                  type="text"
                  inputMode="numeric"
                  maxLength={2}
                  value={form.expiryYy}
                  onChange={(event) => setForm((current) => ({ ...current, expiryYy: event.target.value }))}
                  className={`mt-1 ${UI_FIELD}`}
                  placeholder="YY"
                  required
                />
              </label>
            </div>

            <label className="text-sm text-slate-700">
              DOB (YYYYMMDD)
              <input
                type="text"
                inputMode="numeric"
                maxLength={8}
                value={form.dob}
                onChange={(event) => setForm((current) => ({ ...current, dob: event.target.value }))}
                className={`mt-1 ${UI_FIELD}`}
                placeholder="YYYYMMDD"
                required
              />
            </label>

            <label className="text-sm text-slate-700">
              NN (2 digits)
              <input
                type="password"
                inputMode="numeric"
                maxLength={2}
                value={form.pin2}
                onChange={(event) => setForm((current) => ({ ...current, pin2: event.target.value }))}
                className={`mt-1 ${UI_FIELD}`}
                placeholder="12"
                required
              />
            </label>

            <div className="md:col-span-2 flex flex-wrap gap-2">
              <button type="submit" className={UI_BUTTON_PRIMARY} disabled={loading || submitting || removing || toggling}>
                {submitting ? "Saving..." : "Save Override"}
              </button>
              <button
                type="button"
                className={`${UI_BUTTON_OUTLINE} border-rose-200 text-rose-700 hover:bg-rose-50`}
                onClick={() => void onClearCard()}
                disabled={loading || submitting || removing || toggling}
              >
                {removing ? "Removing..." : "Remove Override"}
              </button>
            </div>
          </form>

          {status.updated_at ? (
            <p className="mt-4 text-xs text-slate-500">
              Last updated: {new Date(status.updated_at).toLocaleString()}
            </p>
          ) : null}
        </>
      ) : null}
    </section>
  );
}
