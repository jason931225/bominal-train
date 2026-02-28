"use client";

import { FormEvent, useEffect, useRef, useState } from "react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { z } from "zod";

import { useLocale } from "@/components/locale-provider";
import { clientApiBaseUrl } from "@/lib/api-base";
import { isPasskeySupported, signInWithPasskey } from "@/lib/passkey";
import { ROUTES } from "@/lib/routes";
import { UI_BUTTON_PRIMARY, UI_FIELD } from "@/lib/ui";

type LoginStep = "email_entry" | "passkey_waiting" | "password" | "alternatives" | "otp_verify";

type LoginFormData = {
  email: string;
  password: string;
  otp_code: string;
  remember_me: boolean;
};

type AuthMethods = {
  password: boolean;
  passkey: boolean;
  magic_link: boolean;
  otp: boolean;
};

const DEFAULT_AUTH_METHODS: AuthMethods = {
  password: true,
  passkey: true,
  magic_link: true,
  otp: false,
};

export function LoginForm() {
  const router = useRouter();
  const { t } = useLocale();
  const [step, setStep] = useState<LoginStep>("email_entry");
  const [methods, setMethods] = useState<AuthMethods>(DEFAULT_AUTH_METHODS);
  const [methodsLoaded, setMethodsLoaded] = useState(false);
  const [form, setForm] = useState<LoginFormData>({
    email: "",
    password: "",
    otp_code: "",
    remember_me: false,
  });
  const [fieldErrors, setFieldErrors] = useState<Partial<Record<keyof LoginFormData, string>>>({});
  const [formError, setFormError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [passkeySubmitting, setPasskeySubmitting] = useState(false);
  const [signInSubmitting, setSignInSubmitting] = useState(false);
  const [requestingMagicLink, setRequestingMagicLink] = useState(false);
  const [requestingOtp, setRequestingOtp] = useState(false);
  const passkeyAttemptIdRef = useRef(0);
  const authResolvedRef = useRef(false);

  const forgotPasswordHref = (() => {
    const email = form.email.trim();
    if (!email) return ROUTES.forgotPassword;
    return `${ROUTES.forgotPassword}?email=${encodeURIComponent(email)}`;
  })();

  const navigateAfterAuth = () => {
    if (typeof window !== "undefined") {
      try {
        window.location.assign(ROUTES.modules.train);
      } catch {
        router.push(ROUTES.modules.train);
      }
      return;
    }
    router.push(ROUTES.modules.train);
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

  const cancelPasskeyAttempt = () => {
    passkeyAttemptIdRef.current += 1;
    setPasskeySubmitting(false);
  };

  useEffect(() => {
    return () => {
      passkeyAttemptIdRef.current += 1;
    };
  }, []);

  const loadAuthMethods = async (): Promise<AuthMethods> => {
    if (methodsLoaded) {
      return methods;
    }
    try {
      const response = await fetch(`${clientApiBaseUrl}/api/auth/methods`, {
        method: "GET",
        credentials: "include",
      });
      if (!response.ok) {
        setMethodsLoaded(true);
        return methods;
      }
      const body = (await response.json().catch(() => null)) as Partial<AuthMethods> | null;
      const nextMethods: AuthMethods = {
        password: body?.password !== false,
        passkey: body?.passkey !== false,
        magic_link: body?.magic_link !== false,
        otp: body?.otp === true,
      };
      setMethods(nextMethods);
      setMethodsLoaded(true);
      return nextMethods;
    } catch {
      setMethodsLoaded(true);
      return methods;
    }
  };

  const startPasskeyAttempt = async (normalizedEmail: string) => {
    const nextMethods = await loadAuthMethods();
    setFormError(null);
    setNotice(null);

    if (!nextMethods.passkey || !isPasskeySupported()) {
      cancelPasskeyAttempt();
      setStep("alternatives");
      return;
    }

    const attemptId = passkeyAttemptIdRef.current + 1;
    passkeyAttemptIdRef.current = attemptId;
    setPasskeySubmitting(true);
    setStep("passkey_waiting");

    void (async () => {
      try {
        const result = await signInWithPasskey(
          clientApiBaseUrl,
          {
            email: normalizedEmail,
            rememberMe: form.remember_me,
          },
          {},
        );

        if (authResolvedRef.current || passkeyAttemptIdRef.current !== attemptId) {
          return;
        }

        setPasskeySubmitting(false);
        if (result.ok) {
          authResolvedRef.current = true;
          navigateAfterAuth();
          return;
        }

        if (!expectedPasskeyFallback(result.error)) {
          setFormError(result.error ?? t("auth.passkeySignInFailed"));
        }
        setStep("alternatives");
      } catch {
        if (authResolvedRef.current || passkeyAttemptIdRef.current !== attemptId) {
          return;
        }
        setPasskeySubmitting(false);
        setFormError(t("auth.passkeySignInFailed"));
        setStep("alternatives");
      }
    })();
  };

  const onContinue = async () => {
    setFieldErrors({});
    setFormError(null);
    setNotice(null);

    const emailSchema = z.string().email(t("auth.invalidEmail"));
    const parsed = emailSchema.safeParse(form.email.trim());
    if (!parsed.success) {
      setFieldErrors({ email: parsed.error.issues[0]?.message ?? t("auth.invalidEmail") });
      return;
    }

    const normalizedEmail = parsed.data.toLowerCase();
    setForm((prev) => ({ ...prev, email: normalizedEmail }));
    await startPasskeyAttempt(normalizedEmail);
  };

  const returnToSignIn = () => {
    cancelPasskeyAttempt();
    setStep("email_entry");
    setFieldErrors({});
    setFormError(null);
    setNotice(null);
    setForm((prev) => ({ ...prev, password: "", otp_code: "" }));
  };

  const onPasswordSignIn = async () => {
    if (authResolvedRef.current) {
      return;
    }
    const loginSchema = z.object({
      email: z.string().email(t("auth.invalidEmail")),
      password: z.string().min(8, t("auth.passwordMin")),
      remember_me: z.boolean(),
    });

    const payload = {
      email: form.email.trim().toLowerCase(),
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

    cancelPasskeyAttempt();
    setSignInSubmitting(true);
    try {
      const response = await fetch(`${clientApiBaseUrl}/api/auth/login`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        credentials: "include",
        body: JSON.stringify(parsed.data),
      });

      if (authResolvedRef.current) {
        return;
      }

      if (!response.ok) {
        const body = (await response.json().catch(() => null)) as { detail?: string } | null;
        setFormError(body?.detail ?? t("auth.invalidLogin"));
        return;
      }

      authResolvedRef.current = true;
      navigateAfterAuth();
    } catch {
      if (authResolvedRef.current) {
        return;
      }
      setFormError(t("auth.apiUnreachable"));
    } finally {
      setSignInSubmitting(false);
    }
  };

  const onRequestMagicLink = async () => {
    setFormError(null);
    setNotice(null);
    const emailSchema = z.string().email(t("auth.invalidEmail"));
    const parsed = emailSchema.safeParse(form.email.trim());
    if (!parsed.success) {
      setFieldErrors({ email: parsed.error.issues[0]?.message ?? t("auth.invalidEmail") });
      return;
    }

    setRequestingMagicLink(true);
    try {
      const response = await fetch(`${clientApiBaseUrl}/api/auth/request-magic-link`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        credentials: "include",
        body: JSON.stringify({ email: parsed.data.toLowerCase() }),
      });
      if (!response.ok) {
        const body = (await response.json().catch(() => null)) as { detail?: string } | null;
        setFormError(body?.detail ?? t("auth.passwordResetRequestFailed"));
        return;
      }
      setNotice(t("auth.magicLinkSent"));
    } catch {
      setFormError(t("auth.apiUnreachable"));
    } finally {
      setRequestingMagicLink(false);
    }
  };

  const onRequestOtp = async () => {
    setFormError(null);
    setNotice(null);
    const emailSchema = z.string().email(t("auth.invalidEmail"));
    const parsed = emailSchema.safeParse(form.email.trim());
    if (!parsed.success) {
      setFieldErrors({ email: parsed.error.issues[0]?.message ?? t("auth.invalidEmail") });
      return;
    }

    setRequestingOtp(true);
    try {
      const response = await fetch(`${clientApiBaseUrl}/api/auth/request-signin-otp`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        credentials: "include",
        body: JSON.stringify({ email: parsed.data.toLowerCase() }),
      });
      if (!response.ok) {
        const body = (await response.json().catch(() => null)) as { detail?: string } | null;
        setFormError(body?.detail ?? t("auth.apiUnreachable"));
        return;
      }
      setNotice(t("auth.otpRequestSent"));
      setStep("otp_verify");
    } catch {
      setFormError(t("auth.apiUnreachable"));
    } finally {
      setRequestingOtp(false);
    }
  };

  const onVerifyOtp = async () => {
    const payloadSchema = z.object({
      email: z.string().email(t("auth.invalidEmail")),
      code: z.string().trim().min(4, t("auth.otpCodeRequired")),
      remember_me: z.boolean(),
    });

    const parsed = payloadSchema.safeParse({
      email: form.email.trim().toLowerCase(),
      code: form.otp_code.trim(),
      remember_me: form.remember_me,
    });
    if (!parsed.success) {
      const nextErrors: Partial<Record<keyof LoginFormData, string>> = {};
      for (const issue of parsed.error.issues) {
        const key = issue.path[0] === "code" ? "otp_code" : (issue.path[0] as keyof LoginFormData);
        if (!nextErrors[key]) {
          nextErrors[key] = issue.message;
        }
      }
      setFieldErrors(nextErrors);
      return;
    }

    setSignInSubmitting(true);
    setFormError(null);
    try {
      const response = await fetch(`${clientApiBaseUrl}/api/auth/verify-signin-otp`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        credentials: "include",
        body: JSON.stringify({
          email: parsed.data.email,
          code: parsed.data.code,
          remember_me: parsed.data.remember_me,
        }),
      });
      if (!response.ok) {
        const body = (await response.json().catch(() => null)) as { detail?: string } | null;
        setFormError(body?.detail ?? t("auth.invalidLogin"));
        return;
      }
      authResolvedRef.current = true;
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
    setNotice(null);

    if (step === "password") {
      await onPasswordSignIn();
      return;
    }
    if (step === "otp_verify") {
      await onVerifyOtp();
      return;
    }
    if (step === "email_entry") {
      await onContinue();
    }
  };

  const showPassword = step === "password";
  const showOtp = step === "otp_verify";
  const showSignInForm = step === "email_entry" || showPassword || showOtp;

  return (
    <form onSubmit={onSubmit} className="space-y-4">
      {showSignInForm ? (
        <div>
          <label className="mb-1 block text-sm font-medium text-slate-700" htmlFor="email">
            {t("auth.email")}
          </label>
          <input
            id="email"
            type="email"
            value={form.email}
            onChange={(event) => {
              cancelPasskeyAttempt();
              setForm((prev) => ({ ...prev, email: event.target.value, password: "", otp_code: "" }));
              if (step !== "email_entry") {
                setStep("email_entry");
              }
            }}
            className={UI_FIELD}
            autoComplete="email"
            required
          />
          {fieldErrors.email ? <p className="mt-1 text-xs text-rose-600">{fieldErrors.email}</p> : null}
        </div>
      ) : null}

      {showPassword ? (
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

      {showOtp ? (
        <div>
          <label className="mb-1 block text-sm font-medium text-slate-700" htmlFor="otp_code">
            {t("auth.otpCode")}
          </label>
          <input
            id="otp_code"
            type="text"
            value={form.otp_code}
            onChange={(event) => setForm((prev) => ({ ...prev, otp_code: event.target.value }))}
            className={UI_FIELD}
            autoComplete="one-time-code"
            required
          />
          {fieldErrors.otp_code ? <p className="mt-1 text-xs text-rose-600">{fieldErrors.otp_code}</p> : null}
        </div>
      ) : null}

      {showSignInForm ? (
        <label className="flex items-center gap-2 text-sm text-slate-600">
          <input
            type="checkbox"
            checked={form.remember_me}
            onChange={(event) => setForm((prev) => ({ ...prev, remember_me: event.target.checked }))}
            className="h-4 w-4 rounded border-blossom-300 text-blossom-500 focus:ring-blossom-300"
          />
          {t("auth.rememberMe")}
        </label>
      ) : null}

      {step === "passkey_waiting" ? (
        <div className="rounded-2xl border border-blossom-100 bg-white p-5 text-center shadow-sm">
          <div className="mx-auto flex h-12 w-12 items-center justify-center rounded-full bg-blossom-50 text-blossom-700">
            <svg viewBox="0 0 24 24" className="h-6 w-6" fill="none" stroke="currentColor" strokeWidth="1.8" aria-hidden="true">
              <rect x="5" y="10" width="14" height="10" rx="2" />
              <path d="M8 10V8a4 4 0 1 1 8 0v2" />
            </svg>
          </div>
          <p className="mt-3 text-sm font-medium text-slate-900">{t("auth.waitingForPasskey")}</p>
          <button
            type="button"
            className="mt-3 text-sm font-medium text-slate-700 underline underline-offset-4 hover:text-slate-900"
            onClick={() => {
              cancelPasskeyAttempt();
              setStep("alternatives");
            }}
          >
            {t("auth.showAlternativeMethods")}
          </button>
          <button
            type="button"
            className="mt-2 block w-full text-center text-sm font-medium text-slate-700 underline underline-offset-4 hover:text-slate-900"
            onClick={returnToSignIn}
          >
            {t("auth.returnToSignIn")}
          </button>
        </div>
      ) : null}

      {step === "alternatives" ? (
        <div className="space-y-2 rounded-2xl border border-slate-200 bg-slate-50 p-4">
          <p className="text-sm font-medium text-slate-800">{t("auth.alternativeMethods")}</p>
          <button
            type="button"
            className="w-full rounded-xl border border-slate-300 bg-white px-4 py-3 text-sm font-medium text-slate-700 transition hover:bg-slate-100"
            onClick={() => setStep("password")}
          >
            {t("auth.signInWithPassword")}
          </button>
          {methods.magic_link ? (
            <button
              type="button"
              className="w-full rounded-xl border border-slate-300 bg-white px-4 py-3 text-sm font-medium text-slate-700 transition hover:bg-slate-100 disabled:opacity-60"
              onClick={() => {
                void onRequestMagicLink();
              }}
              disabled={requestingMagicLink}
            >
              {requestingMagicLink ? t("auth.requestingMagicLink") : t("auth.sendMagicLink")}
            </button>
          ) : null}
          {methods.otp ? (
            <button
              type="button"
              className="w-full rounded-xl border border-slate-300 bg-white px-4 py-3 text-sm font-medium text-slate-700 transition hover:bg-slate-100 disabled:opacity-60"
              onClick={() => {
                void onRequestOtp();
              }}
              disabled={requestingOtp}
            >
              {requestingOtp ? t("auth.requestingOtp") : t("auth.signInWithOtp")}
            </button>
          ) : null}
          <button
            type="button"
            className="w-full rounded-xl border border-slate-300 bg-white px-4 py-3 text-sm font-medium text-slate-700 transition hover:bg-slate-100"
            onClick={() => {
              void startPasskeyAttempt(form.email.trim().toLowerCase());
            }}
          >
            {t("auth.returnToPasskey")}
          </button>
          <button
            type="button"
            className="w-full rounded-xl border border-slate-300 bg-white px-4 py-3 text-sm font-medium text-slate-700 transition hover:bg-slate-100"
            onClick={returnToSignIn}
          >
            {t("auth.returnToSignIn")}
          </button>
        </div>
      ) : null}

      {showPassword ? (
        <p className="text-right text-sm">
          <Link href={forgotPasswordHref} className="font-medium text-blossom-600 hover:text-blossom-700">
            {t("auth.forgotPassword")}
          </Link>
        </p>
      ) : null}

      {showOtp ? (
        <button
          type="button"
          className="w-full rounded-xl border border-slate-300 bg-white px-4 py-3 text-sm font-medium text-slate-700 transition hover:bg-slate-100"
          onClick={() => {
            cancelPasskeyAttempt();
            setStep("passkey_waiting");
            void startPasskeyAttempt(form.email.trim().toLowerCase());
          }}
        >
          {t("auth.returnToPasskey")}
        </button>
      ) : null}

      {notice ? <p className="rounded-xl bg-emerald-50 px-3 py-2 text-sm text-emerald-700">{notice}</p> : null}
      {formError ? <p className="rounded-xl bg-rose-50 px-3 py-2 text-sm text-rose-700">{formError}</p> : null}

      {step === "email_entry" || step === "password" || step === "otp_verify" ? (
        <button
          type="submit"
          disabled={passkeySubmitting || signInSubmitting || requestingMagicLink || requestingOtp}
          className={`w-full ${UI_BUTTON_PRIMARY}`}
        >
          {step === "password"
            ? signInSubmitting
              ? t("auth.signingIn")
              : t("auth.signIn")
            : step === "otp_verify"
              ? signInSubmitting
                ? t("auth.verifyingOtp")
                : t("auth.verifyOtp")
              : passkeySubmitting
                ? t("auth.continuing")
                : t("auth.continue")}
        </button>
      ) : null}
    </form>
  );
}
