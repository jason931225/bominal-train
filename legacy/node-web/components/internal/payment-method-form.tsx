"use client";

import { FormEvent, useMemo, useState } from "react";

import { InternalApiClientError, createInternalApiClient, isInternalCompatibilityAliasEnabled } from "@/lib/internal-api/client";
import { assertEvervaultEncryptedPaymentPayload, encryptPaymentFields, getEvervaultBrowserCredentialsFromEnv } from "@/lib/evervault";
import { UI_BUTTON_PRIMARY, UI_CARD_MD, UI_FIELD, UI_KICKER, UI_TITLE_MD } from "@/lib/ui";

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

export function PaymentMethodForm() {
  const [cardNumber, setCardNumber] = useState("");
  const [expiryMonth, setExpiryMonth] = useState("");
  const [expiryYear, setExpiryYear] = useState("");
  const [birthDate, setBirthDate] = useState("");
  const [pin2, setPin2] = useState("");

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

    const cardDigits = digitsOnly(cardNumber);
    const pin2Digits = digitsOnly(pin2);
    const month = Number(expiryMonth);
    const year = Number(expiryYear);
    const birthDateYyMmDd = birthDateToYyMmDd(birthDate);

    if (!Number.isInteger(month) || month < 1 || month > 12 || !Number.isInteger(year) || year < 2000 || year > 2100) {
      setError("Expiry date is invalid.");
      return;
    }
    if (!birthDateYyMmDd || cardDigits.length < 12 || cardDigits.length > 19 || pin2Digits.length !== 2) {
      setError("Payment fields are invalid.");
      return;
    }

    const evervaultCreds = getEvervaultBrowserCredentialsFromEnv();
    if (!evervaultCreds) {
      setError("Evervault browser credentials are required for internal payment-method submission.");
      return;
    }

    setSubmitting(true);
    setError(null);
    setNotice(null);

    try {
      const encryptedPayload = await encryptPaymentFields(
        {
          card_number: cardDigits,
          pin2: pin2Digits,
          birth_date: birthDateYyMmDd,
          expiry: `${String(year % 100).padStart(2, "0")}${String(month).padStart(2, "0")}`,
          last4: cardDigits.slice(-4),
        },
        evervaultCreds,
      );

      const ciphertextOnlyPayload = assertEvervaultEncryptedPaymentPayload(encryptedPayload);
      const response = await client.upsertProviderPaymentMethod(ciphertextOnlyPayload);
      setNotice(`Payment method updated (${response.source}) via ${client.compatibilityMode}.`);

      setCardNumber("");
      setExpiryMonth("");
      setExpiryYear("");
      setBirthDate("");
      setPin2("");
    } catch (err) {
      if (err instanceof InternalApiClientError) {
        setError(`${err.message}${err.requestId ? ` (request_id: ${err.requestId})` : ""}`);
      } else if (err instanceof Error) {
        setError(err.message);
      } else {
        setError("Could not submit payment method.");
      }
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <section className={UI_CARD_MD}>
      <p className={UI_KICKER}>internal</p>
      <h2 className={`mt-1 ${UI_TITLE_MD}`}>SRT payment method</h2>
      <p className="mt-2 text-sm text-slate-600">PUT /internal/v1/providers/srt/payment-method (Evervault ciphertext only)</p>

      {error ? <p className="mt-3 rounded-xl bg-rose-50 px-3 py-2 text-sm text-rose-700">{error}</p> : null}
      {notice ? <p className="mt-3 rounded-xl bg-emerald-50 px-3 py-2 text-sm text-emerald-700">{notice}</p> : null}

      <form onSubmit={onSubmit} className="mt-4 grid gap-3 md:grid-cols-2">
        <label className="text-sm text-slate-700 md:col-span-2">
          Card number
          <input
            className={`mt-1 ${UI_FIELD}`}
            inputMode="numeric"
            autoComplete="cc-number"
            placeholder="1234 5678 9012 3456"
            value={cardNumber}
            onChange={(event) => setCardNumber(event.target.value)}
            required
          />
        </label>

        <label className="text-sm text-slate-700">
          Expiry month
          <input
            className={`mt-1 ${UI_FIELD}`}
            inputMode="numeric"
            autoComplete="cc-exp-month"
            maxLength={2}
            placeholder="MM"
            value={expiryMonth}
            onChange={(event) => setExpiryMonth(event.target.value)}
            required
          />
        </label>

        <label className="text-sm text-slate-700">
          Expiry year
          <input
            className={`mt-1 ${UI_FIELD}`}
            inputMode="numeric"
            autoComplete="cc-exp-year"
            maxLength={4}
            placeholder="YYYY"
            value={expiryYear}
            onChange={(event) => setExpiryYear(event.target.value)}
            required
          />
        </label>

        <label className="text-sm text-slate-700">
          Date of birth
          <input
            type="date"
            className={`mt-1 ${UI_FIELD}`}
            value={birthDate}
            onChange={(event) => setBirthDate(event.target.value)}
            required
          />
        </label>

        <label className="text-sm text-slate-700">
          Card PIN (first 2 digits)
          <input
            type="password"
            className={`mt-1 ${UI_FIELD}`}
            inputMode="numeric"
            maxLength={2}
            value={pin2}
            onChange={(event) => setPin2(event.target.value)}
            required
          />
        </label>

        <label className="text-sm text-slate-700 md:col-span-2">
          `X-Internal-Api-Key` (optional)
          <input
            type="password"
            className={`mt-1 ${UI_FIELD}`}
            value={internalApiKey}
            onChange={(event) => setInternalApiKey(event.target.value)}
          />
        </label>

        <label className="text-sm text-slate-700 md:col-span-2">
          `X-Internal-Service-Token` (optional)
          <textarea
            className="mt-1 min-h-24 w-full rounded-2xl border border-blossom-200 bg-white px-3 py-2 text-sm text-slate-700 shadow-sm outline-none transition focus:border-blossom-300 focus:ring-2 focus:ring-blossom-100"
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

        <div className="md:col-span-2">
          <button type="submit" className={UI_BUTTON_PRIMARY} disabled={submitting}>
            {submitting ? "Submitting..." : "Submit payment method"}
          </button>
        </div>
      </form>
    </section>
  );
}
