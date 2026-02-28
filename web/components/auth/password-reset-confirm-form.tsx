"use client";

import { FormEvent, useMemo, useState } from "react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { z } from "zod";

import { useLocale } from "@/components/locale-provider";
import { clientApiBaseUrl } from "@/lib/api-base";
import { ROUTES } from "@/lib/routes";
import { clearSupabaseAccessToken, getSupabaseAccessToken } from "@/lib/supabase-auth";
import { UI_BUTTON_PRIMARY, UI_FIELD } from "@/lib/ui";

type ResetMode = "otp" | "supabase";

type ResetConfirmFormState = {
  email: string;
  code: string;
  newPassword: string;
  confirmNewPassword: string;
};

export function PasswordResetConfirmForm({
  initialEmail = "",
  initialCode = "",
  mode = "otp",
}: {
  initialEmail?: string;
  initialCode?: string;
  mode?: ResetMode;
}) {
  const router = useRouter();
  const { t } = useLocale();
  const [form, setForm] = useState<ResetConfirmFormState>({
    email: initialEmail,
    code: initialCode,
    newPassword: "",
    confirmNewPassword: "",
  });
  const [fieldErrors, setFieldErrors] = useState<Partial<Record<keyof ResetConfirmFormState, string>>>({});
  const [formError, setFormError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  const hasPrefilledLinkParams = useMemo(
    () => mode === "otp" && Boolean(initialEmail.trim()) && Boolean(initialCode.trim()),
    [initialCode, initialEmail, mode],
  );

  const onSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setFieldErrors({});
    setFormError(null);
    setNotice(null);

    const otpMode = mode === "otp";
    const schema = z
      .object({
        email: z.string().email(t("auth.invalidEmail")).optional(),
        code: z.string().trim().min(4, t("auth.passwordResetCodeRequired")).optional(),
        newPassword: z.string().min(8, t("auth.passwordMin")),
        confirmNewPassword: z.string().min(8, t("auth.passwordMin")),
      })
      .refine((value) => value.newPassword === value.confirmNewPassword, {
        path: ["confirmNewPassword"],
        message: t("auth.passwordConfirmMismatch"),
      })
      .refine((value) => !otpMode || Boolean(value.email), {
        path: ["email"],
        message: t("auth.invalidEmail"),
      })
      .refine((value) => !otpMode || Boolean(value.code), {
        path: ["code"],
        message: t("auth.passwordResetCodeRequired"),
      });

    const parsed = schema.safeParse({
      email: otpMode ? form.email.trim() : undefined,
      code: otpMode ? form.code.trim() : undefined,
      newPassword: form.newPassword,
      confirmNewPassword: form.confirmNewPassword,
    });

    if (!parsed.success) {
      const nextErrors: Partial<Record<keyof ResetConfirmFormState, string>> = {};
      for (const issue of parsed.error.issues) {
        const key = issue.path[0] as keyof ResetConfirmFormState;
        if (!nextErrors[key]) {
          nextErrors[key] = issue.message;
        }
      }
      setFieldErrors(nextErrors);
      return;
    }

    setSubmitting(true);
    try {
      const endpoint = otpMode ? "/api/auth/reset-password" : "/api/auth/reset-password/supabase";
      const headers: Record<string, string> = { "Content-Type": "application/json" };
      const body = otpMode
        ? JSON.stringify({
            email: parsed.data.email,
            code: parsed.data.code,
            new_password: parsed.data.newPassword,
          })
        : JSON.stringify({ new_password: parsed.data.newPassword });
      if (!otpMode) {
        const accessToken = await getSupabaseAccessToken();
        if (!accessToken) {
          setFormError(t("auth.supabaseRecoveryMissing"));
          return;
        }
        headers.Authorization = `Bearer ${accessToken}`;
      }

      const response = await fetch(`${clientApiBaseUrl}${endpoint}`, {
        method: "POST",
        credentials: "include",
        headers,
        body,
      });
      if (!response.ok) {
        const body = (await response.json().catch(() => null)) as { detail?: string } | null;
        setFormError(body?.detail ?? t("auth.passwordResetFailed"));
        return;
      }

      clearSupabaseAccessToken();
      setNotice(t("auth.passwordResetComplete"));
      setTimeout(() => {
        router.push(`${ROUTES.authPasskeySetup}?source=reset&next=${encodeURIComponent(ROUTES.modules.train)}`);
      }, 700);
    } catch {
      setFormError(t("auth.apiUnreachable"));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <form onSubmit={onSubmit} className="space-y-4">
      {hasPrefilledLinkParams ? (
        <p
          data-testid="password-reset-link-detected"
          className="rounded-xl bg-blossom-50 px-3 py-2 text-sm text-blossom-700"
        >
          {t("auth.passwordResetLinkDetected")}
        </p>
      ) : null}
      {mode === "supabase" ? (
        <p className="rounded-xl bg-blossom-50 px-3 py-2 text-sm text-blossom-700">{t("auth.supabaseResetReady")}</p>
      ) : null}

      {mode === "otp" ? (
        <>
          <div>
            <label className="mb-1 block text-sm font-medium text-slate-700" htmlFor="reset-confirm-email">
              {t("auth.email")}
            </label>
            <input
              id="reset-confirm-email"
              type="email"
              autoComplete="email"
              value={form.email}
              onChange={(event) => setForm((prev) => ({ ...prev, email: event.target.value }))}
              className={UI_FIELD}
              required
            />
            {fieldErrors.email ? <p className="mt-1 text-xs text-rose-600">{fieldErrors.email}</p> : null}
          </div>

          <div>
            <label className="mb-1 block text-sm font-medium text-slate-700" htmlFor="reset-confirm-code">
              {t("auth.passwordResetCode")}
            </label>
            <input
              id="reset-confirm-code"
              type="text"
              autoComplete="one-time-code"
              value={form.code}
              onChange={(event) => setForm((prev) => ({ ...prev, code: event.target.value }))}
              className={UI_FIELD}
              required
            />
            {fieldErrors.code ? <p className="mt-1 text-xs text-rose-600">{fieldErrors.code}</p> : null}
          </div>
        </>
      ) : null}

      <div>
        <label className="mb-1 block text-sm font-medium text-slate-700" htmlFor="reset-new-password">
          {t("settings.newPassword")}
        </label>
        <input
          id="reset-new-password"
          type="password"
          autoComplete="new-password"
          value={form.newPassword}
          onChange={(event) => setForm((prev) => ({ ...prev, newPassword: event.target.value }))}
          className={UI_FIELD}
          required
        />
        {fieldErrors.newPassword ? <p className="mt-1 text-xs text-rose-600">{fieldErrors.newPassword}</p> : null}
      </div>

      <div>
        <label className="mb-1 block text-sm font-medium text-slate-700" htmlFor="reset-confirm-password">
          {t("settings.confirmNewPassword")}
        </label>
        <input
          id="reset-confirm-password"
          type="password"
          autoComplete="new-password"
          value={form.confirmNewPassword}
          onChange={(event) => setForm((prev) => ({ ...prev, confirmNewPassword: event.target.value }))}
          className={UI_FIELD}
          required
        />
        {fieldErrors.confirmNewPassword ? (
          <p data-testid="password-reset-confirm-mismatch" className="mt-1 text-xs text-rose-600">
            {fieldErrors.confirmNewPassword}
          </p>
        ) : null}
      </div>

      {formError ? <p className="rounded-xl bg-rose-50 px-3 py-2 text-sm text-rose-700">{formError}</p> : null}
      {notice ? <p className="rounded-xl bg-emerald-50 px-3 py-2 text-sm text-emerald-700">{notice}</p> : null}

      <button type="submit" disabled={submitting} className={`w-full ${UI_BUTTON_PRIMARY}`}>
        {submitting ? t("auth.resettingPassword") : t("auth.resetPassword")}
      </button>

      <p className="text-sm text-slate-600">
        <Link href={ROUTES.forgotPassword} className="font-medium text-blossom-600 hover:text-blossom-700">
          {t("auth.requestAnotherResetCode")}
        </Link>
      </p>
    </form>
  );
}
