"use client";

import { FormEvent, useState } from "react";
import Link from "next/link";
import { z } from "zod";

import { useLocale } from "@/components/locale-provider";
import { clientApiBaseUrl } from "@/lib/api-base";
import { ROUTES } from "@/lib/routes";
import { UI_BUTTON_PRIMARY, UI_FIELD } from "@/lib/ui";

type RequestFormState = {
  email: string;
};

export function PasswordResetRequestForm({ initialEmail = "" }: { initialEmail?: string }) {
  const { t } = useLocale();
  const [form, setForm] = useState<RequestFormState>({ email: initialEmail });
  const [fieldError, setFieldError] = useState<string | null>(null);
  const [formError, setFormError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  const onSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setFieldError(null);
    setFormError(null);
    setNotice(null);

    const schema = z.object({
      email: z.string().email(t("auth.invalidEmail")),
    });
    const parsed = schema.safeParse({ email: form.email.trim() });
    if (!parsed.success) {
      setFieldError(parsed.error.issues[0]?.message ?? t("auth.invalidEmail"));
      return;
    }

    setSubmitting(true);
    try {
      const response = await fetch(`${clientApiBaseUrl}/api/auth/request-password-reset`, {
        method: "POST",
        credentials: "include",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(parsed.data),
      });
      if (!response.ok) {
        const body = (await response.json().catch(() => null)) as { detail?: string } | null;
        setFormError(body?.detail ?? t("auth.passwordResetRequestFailed"));
        return;
      }
      setNotice(t("auth.passwordResetRequestSent"));
    } catch {
      setFormError(t("auth.apiUnreachable"));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <form onSubmit={onSubmit} className="space-y-4">
      <div>
        <label className="mb-1 block text-sm font-medium text-slate-700" htmlFor="reset-email">
          {t("auth.email")}
        </label>
        <input
          id="reset-email"
          type="email"
          autoComplete="email"
          value={form.email}
          onChange={(event) => setForm({ email: event.target.value })}
          className={UI_FIELD}
          required
        />
        {fieldError ? <p className="mt-1 text-xs text-rose-600">{fieldError}</p> : null}
      </div>

      {formError ? <p className="rounded-xl bg-rose-50 px-3 py-2 text-sm text-rose-700">{formError}</p> : null}
      {notice ? <p className="rounded-xl bg-emerald-50 px-3 py-2 text-sm text-emerald-700">{notice}</p> : null}

      <button type="submit" disabled={submitting} className={`w-full ${UI_BUTTON_PRIMARY}`}>
        {submitting ? t("auth.requestingPasswordReset") : t("auth.requestPasswordReset")}
      </button>

      <p className="text-sm text-slate-600">
        <Link href={ROUTES.resetPassword} className="font-medium text-blossom-600 hover:text-blossom-700">
          {t("auth.haveResetCode")}
        </Link>
      </p>
    </form>
  );
}
