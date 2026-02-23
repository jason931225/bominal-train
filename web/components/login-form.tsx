"use client";

import { FormEvent, useState } from "react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { z } from "zod";

import { useLocale } from "@/components/locale-provider";
import { clientApiBaseUrl } from "@/lib/api-base";
import { signInWithPasskey } from "@/lib/passkey";
import { ROUTES } from "@/lib/routes";
import { UI_BUTTON_PRIMARY, UI_FIELD } from "@/lib/ui";

type LoginFormData = {
  email: string;
  password: string;
  remember_me: boolean;
};

export function LoginForm() {
  const router = useRouter();
  const { t } = useLocale();
  const [form, setForm] = useState<LoginFormData>({
    email: "",
    password: "",
    remember_me: false,
  });
  const [fieldErrors, setFieldErrors] = useState<Partial<Record<keyof LoginFormData, string>>>({});
  const [formError, setFormError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const [passkeySubmitting, setPasskeySubmitting] = useState(false);

  const onPasskeySignIn = async () => {
    setFieldErrors({});
    setFormError(null);
    if (!form.email.trim()) {
      setFieldErrors({ email: t("auth.invalidEmail") });
      return;
    }

    setPasskeySubmitting(true);
    try {
      const result = await signInWithPasskey(clientApiBaseUrl, {
        email: form.email.trim().toLowerCase(),
        rememberMe: form.remember_me,
      });
      if (!result.ok) {
        setFormError(result.error ?? t("auth.passkeySignInFailed"));
        return;
      }
      router.push(ROUTES.dashboard);
    } catch {
      setFormError(t("auth.passkeySignInFailed"));
    } finally {
      setPasskeySubmitting(false);
    }
  };

  const onSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setFieldErrors({});
    setFormError(null);

    const loginSchema = z.object({
      email: z.string().email(t("auth.invalidEmail")),
      password: z.string().min(8, t("auth.passwordMin")),
      remember_me: z.boolean(),
    });

    const payload = {
      email: form.email.trim(),
      password: form.password,
      remember_me: form.remember_me,
    };

    const parsed = loginSchema.safeParse(payload);
    if (!parsed.success) {
      const nextErrors: Partial<Record<keyof LoginFormData, string>> = {};
      for (const issue of parsed.error.issues) {
        const key = issue.path[0] as keyof LoginFormData;
        if (!nextErrors[key]) {
          nextErrors[key] = issue.message;
        }
      }
      setFieldErrors(nextErrors);
      return;
    }

    setSubmitting(true);
    try {
      const response = await fetch(`${clientApiBaseUrl}/api/auth/login`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        credentials: "include",
        body: JSON.stringify(parsed.data),
      });

      if (!response.ok) {
        const body = (await response.json().catch(() => null)) as { detail?: string } | null;
        setFormError(body?.detail ?? t("auth.invalidLogin"));
        return;
      }

      router.push(ROUTES.dashboard);
    } catch {
      setFormError(t("auth.apiUnreachable"));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <form onSubmit={onSubmit} className="space-y-4">
      <div>
        <label className="mb-1 block text-sm font-medium text-slate-700" htmlFor="email">
          {t("auth.email")}
        </label>
        <input
          id="email"
          type="email"
          value={form.email}
          onChange={(event) => setForm((prev) => ({ ...prev, email: event.target.value }))}
          className={UI_FIELD}
          autoComplete="email"
          required
        />
        {fieldErrors.email ? <p className="mt-1 text-xs text-rose-600">{fieldErrors.email}</p> : null}
      </div>

      <div>
        <label className="mb-1 block text-sm font-medium text-slate-700" htmlFor="password">
          {t("auth.password")}
        </label>
        <input
          id="password"
          type="password"
          value={form.password}
          onChange={(event) => setForm((prev) => ({ ...prev, password: event.target.value }))}
          className={UI_FIELD}
          autoComplete="current-password"
          required
        />
        {fieldErrors.password ? <p className="mt-1 text-xs text-rose-600">{fieldErrors.password}</p> : null}
      </div>

      <label className="flex items-center gap-2 text-sm text-slate-600">
        <input
          type="checkbox"
          checked={form.remember_me}
          onChange={(event) => setForm((prev) => ({ ...prev, remember_me: event.target.checked }))}
          className="h-4 w-4 rounded border-blossom-300 text-blossom-500 focus:ring-blossom-300"
        />
        {t("auth.rememberMe")}
      </label>

      <p className="text-right text-sm">
        <Link href={ROUTES.forgotPassword} className="font-medium text-blossom-600 hover:text-blossom-700">
          {t("auth.forgotPassword")}
        </Link>
      </p>

      {formError ? <p className="rounded-xl bg-rose-50 px-3 py-2 text-sm text-rose-700">{formError}</p> : null}

      <button
        type="submit"
        disabled={submitting || passkeySubmitting}
        className={`w-full ${UI_BUTTON_PRIMARY}`}
      >
        {submitting ? t("auth.signingIn") : t("auth.signIn")}
      </button>

      <button
        type="button"
        disabled={submitting || passkeySubmitting}
        onClick={onPasskeySignIn}
        className="w-full rounded-full border border-blossom-200 bg-white px-4 py-2 text-sm font-medium text-blossom-700 transition hover:bg-blossom-50 focus:outline-none focus:ring-2 focus:ring-blossom-200 disabled:cursor-not-allowed disabled:opacity-60"
      >
        {passkeySubmitting ? t("auth.signingInWithPasskey") : t("auth.signInWithPasskey")}
      </button>
    </form>
  );
}
