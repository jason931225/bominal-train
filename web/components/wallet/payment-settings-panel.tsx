"use client";

import { FormEvent, useEffect, useState } from "react";

import { clientApiBaseUrl } from "@/lib/api-base";
import { UI_BUTTON_PRIMARY, UI_CARD_MD, UI_FIELD, UI_KICKER, UI_TITLE_MD } from "@/lib/ui";
import type { WalletPaymentCardStatus } from "@/lib/types";

type PaymentFormState = {
  cardNumber: string;
  expiryMonth: string;
  expiryYear: string;
  cvv: string;
  birthDate: string;
  pin2: string;
};

const EMPTY_FORM: PaymentFormState = {
  cardNumber: "",
  expiryMonth: "",
  expiryYear: "",
  cvv: "",
  birthDate: "",
  pin2: "",
};

async function parseApiErrorMessage(response: Response, fallback: string): Promise<string> {
  const contentType = response.headers.get("content-type") ?? "";
  if (contentType.includes("application/json")) {
    const payload = (await response.json().catch(() => null)) as { detail?: string } | null;
    if (payload?.detail) {
      return payload.detail;
    }
    return fallback;
  }

  const text = await response.text().catch(() => "");
  return text.trim() || fallback;
}

export function PaymentSettingsPanel() {
  const [status, setStatus] = useState<WalletPaymentCardStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [form, setForm] = useState<PaymentFormState>(EMPTY_FORM);

  const loadStatus = async () => {
    setLoading(true);
    try {
      const response = await fetch(`${clientApiBaseUrl}/api/wallet/payment-card`, {
        credentials: "include",
        cache: "no-store",
      });
      if (!response.ok) {
        throw new Error("failed");
      }
      const payload = (await response.json()) as WalletPaymentCardStatus;
      setStatus(payload);
    } catch {
      setError("Could not load wallet settings.");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void loadStatus();
  }, []);

  const onSave = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const expiryMonth = Number(form.expiryMonth);
    const expiryYear = Number(form.expiryYear);
    if (!Number.isInteger(expiryMonth) || expiryMonth < 1 || expiryMonth > 12) {
      setError("Expiry month must be between 01 and 12.");
      return;
    }
    if (!Number.isInteger(expiryYear) || expiryYear < 2000 || expiryYear > 2100) {
      setError("Expiry year must be a 4-digit year.");
      return;
    }

    setSubmitting(true);
    setError(null);
    setNotice(null);
    try {
      const response = await fetch(`${clientApiBaseUrl}/api/wallet/payment-card`, {
        method: "POST",
        credentials: "include",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          card_number: form.cardNumber,
          expiry_month: expiryMonth,
          expiry_year: expiryYear,
          cvv: form.cvv,
          birth_date: form.birthDate,
          pin2: form.pin2,
        }),
      });
      if (!response.ok) {
        setError(await parseApiErrorMessage(response, "Could not save wallet settings."));
        return;
      }

      const payload = (await response.json()) as WalletPaymentCardStatus;
      setStatus(payload);
      setForm(EMPTY_FORM);
      setNotice("Wallet settings saved securely.");
    } catch {
      setError("Could not save wallet settings.");
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className={UI_CARD_MD}>
      <div className="flex items-center justify-between gap-3">
        <div>
          <p className={UI_KICKER}>Settings</p>
          <h2 className={`mt-1 ${UI_TITLE_MD}`}>Payment settings</h2>
        </div>
        {loading ? (
          <span className="text-xs text-slate-500">Loading...</span>
        ) : status?.configured ? (
          <span className="rounded-full border border-emerald-200 bg-emerald-50 px-2 py-0.5 text-xs text-emerald-700">
            Saved
          </span>
        ) : (
          <span className="rounded-full border border-amber-200 bg-amber-50 px-2 py-0.5 text-xs text-amber-700">
            Not set
          </span>
        )}
      </div>

      <p className="mt-2 text-sm text-slate-600">Wallet is shared across bominal services.</p>

      {status?.configured ? (
        <div className="mt-3 space-y-1 text-sm text-slate-600">
          <p>Card ending in {status.card_masked?.match(/(\d{4})$/)?.[1] ?? "****"}</p>
        </div>
      ) : null}

      {error ? <p className="mt-3 rounded-xl bg-rose-50 px-3 py-2 text-sm text-rose-700">{error}</p> : null}
      {notice ? <p className="mt-3 rounded-xl bg-emerald-50 px-3 py-2 text-sm text-emerald-700">{notice}</p> : null}

      <form onSubmit={onSave} className="mt-4 grid gap-3 md:grid-cols-2">
        <label className="text-sm text-slate-700">
          Card number
          <input
            type="text"
            inputMode="numeric"
            autoComplete="cc-number"
            value={form.cardNumber}
            onChange={(event) => setForm((current) => ({ ...current, cardNumber: event.target.value }))}
            className={`mt-1 ${UI_FIELD}`}
            placeholder="1234 5678 9012 3456"
            required
          />
        </label>

        <div className="grid grid-cols-2 gap-2">
          <label className="text-sm text-slate-700">
            Expiry month
            <input
              type="text"
              inputMode="numeric"
              autoComplete="cc-exp-month"
              value={form.expiryMonth}
              onChange={(event) => setForm((current) => ({ ...current, expiryMonth: event.target.value }))}
              className={`mt-1 ${UI_FIELD}`}
              placeholder="MM"
              maxLength={2}
              required
            />
          </label>
          <label className="text-sm text-slate-700">
            Expiry year
            <input
              type="text"
              inputMode="numeric"
              autoComplete="cc-exp-year"
              value={form.expiryYear}
              onChange={(event) => setForm((current) => ({ ...current, expiryYear: event.target.value }))}
              className={`mt-1 ${UI_FIELD}`}
              placeholder="YYYY"
              maxLength={4}
              required
            />
          </label>
        </div>

        <label className="text-sm text-slate-700">
          CVV
          <input
            type="password"
            inputMode="numeric"
            autoComplete="cc-csc"
            value={form.cvv}
            onChange={(event) => setForm((current) => ({ ...current, cvv: event.target.value }))}
            className={`mt-1 ${UI_FIELD}`}
            placeholder="3-4 digits"
            maxLength={4}
            required
          />
        </label>

        <label className="text-sm text-slate-700">
          Date of birth
          <input
            type="date"
            value={form.birthDate}
            onChange={(event) => setForm((current) => ({ ...current, birthDate: event.target.value }))}
            className={`mt-1 ${UI_FIELD}`}
            required
          />
        </label>

        <label className="text-sm text-slate-700 md:col-span-2">
          Card PIN (first 2 digits)
          <input
            type="password"
            inputMode="numeric"
            value={form.pin2}
            onChange={(event) => setForm((current) => ({ ...current, pin2: event.target.value }))}
            className={`mt-1 ${UI_FIELD}`}
            placeholder="••"
            maxLength={2}
            required
          />
        </label>

        <div className="md:col-span-2">
          <button type="submit" disabled={submitting} className={UI_BUTTON_PRIMARY}>
            {submitting ? "Saving..." : "Save payment settings"}
          </button>
        </div>
      </form>
    </div>
  );
}
