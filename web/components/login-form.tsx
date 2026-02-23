"use client";

import { FormEvent, useState } from "react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { z } from "zod";

import { useLocale } from "@/components/locale-provider";
import { clientApiBaseUrl } from "@/lib/api-base";
import { isPasskeySupported, signInWithPasskey } from "@/lib/passkey";
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
  const [step, setStep] = useState<"email" | "password">("email");
  const [form, setForm] = useState<LoginFormData>({
    email: "",
    password: "",
    remember_me: false,
  });
  const [fieldErrors, setFieldErrors] = useState<Partial<Record<keyof LoginFormData, string>>>({});
  const [formError, setFormError] = useState<string | null>(null);
  const [continueSubmitting, setContinueSubmitting] = useState(false);
  const [signInSubmitting, setSignInSubmitting] = useState(false);
  const showingPassword = step === "password";

  const navigateAfterAuth = () => {
    if (typeof window !== "undefined") {
      // Root layout is persistent in App Router; full navigation refreshes top-nav auth state.
      try {
        window.location.assign(ROUTES.dashboard);
      } catch {
        // jsdom/limited runtimes may not implement navigation methods.
        router.push(ROUTES.dashboard);
      }
      return;
    }
    router.push(ROUTES.dashboard);
  };

  const expectedPasskeyFallback = (message: string | undefined): boolean => {
    if (!message) return true;
    const normalized = message.toLowerCase();
    return (
      normalized.includes("no passkey registered") ||
      normalized.includes("cancel") ||
      normalized.includes("not supported") ||
      normalized.includes("security check failed")
    );
  };

  const onContinue = async () => {
    setFieldErrors({});
    setFormError(null);
    const emailSchema = z.string().email(t("auth.invalidEmail"));
    const parsed = emailSchema.safeParse(form.email.trim());
    if (!parsed.success) {
      setFieldErrors({ email: parsed.error.issues[0]?.message ?? t("auth.invalidEmail") });
      return;
    }

    const normalizedEmail = parsed.data.toLowerCase();
    setContinueSubmitting(true);
    try {
      if (!isPasskeySupported()) {
        setStep("password");
        return;
      }
      const result = await signInWithPasskey(clientApiBaseUrl, {
        email: normalizedEmail,
        rememberMe: form.remember_me,
      });
      if (result.ok) {
        navigateAfterAuth();
        return;
      }

      if (!expectedPasskeyFallback(result.error)) {
        setFormError(result.error ?? t("auth.passkeySignInFailed"));
      }
      setStep("password");
    } catch {
      setFormError(t("auth.passkeySignInFailed"));
      setStep("password");
    } finally {
      setContinueSubmitting(false);
    }
  };

  const onPasswordSignIn = async () => {
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

    setSignInSubmitting(true);
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

      navigateAfterAuth();
    } catch {
      setFormError(t("auth.apiUnreachable"));
    } finally {
      setSignInSubmitting(false);
    }
  };

  const onSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setFieldErrors({});
    setFormError(null);

    if (showingPassword) {
      await onPasswordSignIn();
      return;
    }

    await onContinue();
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
          onChange={(event) => {
            const nextEmail = event.target.value;
            setForm((prev) => ({ ...prev, email: nextEmail, ...(step === "password" ? { password: "" } : {}) }));
            if (step === "password") {
              setStep("email");
            }
          }}
          className={UI_FIELD}
          autoComplete="email"
          required
        />
        {fieldErrors.email ? <p className="mt-1 text-xs text-rose-600">{fieldErrors.email}</p> : null}
      </div>

      {showingPassword ? (
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
      ) : null}

      <label className="flex items-center gap-2 text-sm text-slate-600">
        <input
          type="checkbox"
          checked={form.remember_me}
          onChange={(event) => setForm((prev) => ({ ...prev, remember_me: event.target.checked }))}
          className="h-4 w-4 rounded border-blossom-300 text-blossom-500 focus:ring-blossom-300"
        />
        {t("auth.rememberMe")}
      </label>

      {showingPassword ? (
        <p className="text-right text-sm">
          <Link href={ROUTES.forgotPassword} className="font-medium text-blossom-600 hover:text-blossom-700">
            {t("auth.forgotPassword")}
          </Link>
        </p>
      ) : null}

      {formError ? <p className="rounded-xl bg-rose-50 px-3 py-2 text-sm text-rose-700">{formError}</p> : null}

      <button
        type="submit"
        disabled={continueSubmitting || signInSubmitting}
        className={`w-full ${UI_BUTTON_PRIMARY}`}
      >
        {showingPassword
          ? signInSubmitting
            ? t("auth.signingIn")
            : t("auth.signIn")
          : continueSubmitting
            ? t("auth.continuing")
            : t("auth.continue")}
      </button>
    </form>
  );
}
