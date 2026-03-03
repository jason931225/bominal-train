"use client";

import { FormEvent, useCallback, useEffect, useState } from "react";

import { useLocale } from "@/components/locale-provider";
import { clientApiBaseUrl } from "@/lib/api-base";
import { encryptPaymentFields } from "@/lib/evervault";
import { UI_BUTTON_OUTLINE, UI_BUTTON_PRIMARY, UI_CARD_MD, UI_FIELD, UI_KICKER, UI_TITLE_MD } from "@/lib/ui";
import type { WalletPaymentCardStatus } from "@/lib/types";

type PaymentFormState = {
  cardNumber: string;
  expiryMonth: string;
  expiryYear: string;
  birthDate: string;
  pin2: string;
};

const EMPTY_FORM: PaymentFormState = {
  cardNumber: "",
  expiryMonth: "",
  expiryYear: "",
  birthDate: "",
  pin2: "",
};

const EVERVAULT_TEAM_ID = (process.env.NEXT_PUBLIC_EVERVAULT_TEAM_ID ?? "").trim();
const EVERVAULT_APP_ID = (process.env.NEXT_PUBLIC_EVERVAULT_APP_ID ?? "").trim();

function digitsOnly(value: string): string {
  return value.replace(/\D/g, "");
}

function birthDateToYyMmDd(value: string): string | null {
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(value.trim());
  if (!match) {
    return null;
  }
  const [, year, month, day] = match;
  return `${year.slice(-2)}${month}${day}`;
}

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
  const { t } = useLocale();
  const [status, setStatus] = useState<WalletPaymentCardStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [submitting, setSubmitting] = useState(false);
  const [removing, setRemoving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [form, setForm] = useState<PaymentFormState>(EMPTY_FORM);

  const loadStatus = useCallback(async () => {
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
      setError(t("wallet.loadError"));
    } finally {
      setLoading(false);
    }
  }, [t]);

  useEffect(() => {
    void loadStatus();
  }, [loadStatus]);

  const onSave = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const expiryMonth = Number(form.expiryMonth);
    const expiryYear = Number(form.expiryYear);
    const cardNumberDigits = digitsOnly(form.cardNumber);
    const pin2Digits = digitsOnly(form.pin2);
    const birthDateYyMmDd = birthDateToYyMmDd(form.birthDate);
    if (!Number.isInteger(expiryMonth) || expiryMonth < 1 || expiryMonth > 12) {
      setError(t("wallet.expiryMonthError"));
      return;
    }
    if (!Number.isInteger(expiryYear) || expiryYear < 2000 || expiryYear > 2100) {
      setError(t("wallet.expiryYearError"));
      return;
    }
    if (cardNumberDigits.length < 12 || cardNumberDigits.length > 19 || pin2Digits.length !== 2 || !birthDateYyMmDd) {
      setError(t("wallet.saveError"));
      return;
    }

    setSubmitting(true);
    setError(null);
    setNotice(null);
    try {
      let requestBody: Record<string, string | number | null>;
      if (EVERVAULT_TEAM_ID && EVERVAULT_APP_ID) {
        const encryptedPayload = await encryptPaymentFields(
          {
            card_number: cardNumberDigits,
            pin2: pin2Digits,
            birth_date: birthDateYyMmDd,
            expiry: `${String(expiryYear % 100).padStart(2, "0")}${String(expiryMonth).padStart(2, "0")}`,
            last4: cardNumberDigits.slice(-4),
          },
          { teamId: EVERVAULT_TEAM_ID, appId: EVERVAULT_APP_ID },
        );
        requestBody = encryptedPayload;
      } else {
        requestBody = {
          card_number: cardNumberDigits,
          expiry_month: expiryMonth,
          expiry_year: expiryYear,
          birth_date: form.birthDate,
          pin2: pin2Digits,
        };
      }

      const response = await fetch(`${clientApiBaseUrl}/api/wallet/payment-card`, {
        method: "POST",
        credentials: "include",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(requestBody),
      });
      if (!response.ok) {
        setError(await parseApiErrorMessage(response, t("wallet.saveError")));
        return;
      }

      const payload = (await response.json()) as WalletPaymentCardStatus;
      setStatus(payload);
      setForm(EMPTY_FORM);
      setNotice(t("wallet.saveNotice"));
    } catch {
      setError(t("wallet.saveError"));
    } finally {
      setSubmitting(false);
    }
  };

  const onRemove = async () => {
    const confirmed = window.confirm(t("wallet.confirmRemove"));
    if (!confirmed) return;

    setRemoving(true);
    setError(null);
    setNotice(null);
    try {
      const response = await fetch(`${clientApiBaseUrl}/api/wallet/payment-card`, {
        method: "DELETE",
        credentials: "include",
      });
      if (!response.ok) {
        setError(await parseApiErrorMessage(response, t("wallet.removeError")));
        return;
      }

      const payload = (await response.json()) as WalletPaymentCardStatus;
      setStatus(payload);
      setForm(EMPTY_FORM);
      setNotice(t("wallet.removedNotice"));
    } catch {
      setError(t("wallet.removeError"));
    } finally {
      setRemoving(false);
    }
  };

  return (
    <div className={UI_CARD_MD}>
      <div className="flex items-center justify-between gap-3">
        <div>
          <p className={UI_KICKER}>{t("settings.kicker")}</p>
          <h2 className={`mt-1 ${UI_TITLE_MD}`}>{t("wallet.title")}</h2>
        </div>
        {loading ? (
          <span className="text-xs text-slate-500">{t("common.loading")}</span>
        ) : status?.configured ? (
          <span className="rounded-full border border-emerald-200 bg-emerald-50 px-2 py-0.5 text-xs text-emerald-700">
            {t("common.saved")}
          </span>
        ) : (
          <span className="rounded-full border border-amber-200 bg-amber-50 px-2 py-0.5 text-xs text-amber-700">
            {t("common.notSet")}
          </span>
        )}
      </div>

      <p className="mt-2 text-sm text-slate-600">{t("wallet.body")}</p>

      {status?.configured ? (
        <div className="mt-3 space-y-1 text-sm text-slate-600">
          <p>{t("wallet.cardEndingIn", { last4: status.card_masked?.match(/(\\d{4})$/)?.[1] ?? "****" })}</p>
        </div>
      ) : null}

      {error ? <p className="mt-3 rounded-xl bg-rose-50 px-3 py-2 text-sm text-rose-700">{error}</p> : null}
      {notice ? <p className="mt-3 rounded-xl bg-emerald-50 px-3 py-2 text-sm text-emerald-700">{notice}</p> : null}

      <form onSubmit={onSave} className="mt-4 grid gap-3 md:grid-cols-2">
        <label className="text-sm text-slate-700">
          {t("wallet.cardNumber")}
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
            {t("wallet.expiryMonth")}
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
            {t("wallet.expiryYear")}
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
          {t("wallet.dob")}
          <input
            type="date"
            value={form.birthDate}
            onChange={(event) => setForm((current) => ({ ...current, birthDate: event.target.value }))}
            className={`mt-1 ${UI_FIELD}`}
            required
          />
        </label>

        <label className="text-sm text-slate-700 md:col-span-2">
          {t("wallet.pin2")}
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
          <div className="flex flex-wrap items-center gap-2">
            <button type="submit" disabled={submitting || removing} className={UI_BUTTON_PRIMARY}>
              {submitting ? t("wallet.saving") : t("wallet.saveButton")}
            </button>
            {status?.configured ? (
              <button
                type="button"
                onClick={() => void onRemove()}
                disabled={submitting || removing}
                className={`${UI_BUTTON_OUTLINE} border-rose-200 text-rose-700 hover:bg-rose-50 focus:ring-rose-100`}
              >
                {removing ? t("wallet.removing") : t("wallet.removeButton")}
              </button>
            ) : null}
          </div>
        </div>
      </form>
    </div>
  );
}
