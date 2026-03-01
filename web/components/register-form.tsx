"use client";

import { FormEvent, useMemo, useState } from "react";
import { useRouter } from "next/navigation";
import { z } from "zod";

import { useLocale } from "@/components/locale-provider";
import { clientApiBaseUrl } from "@/lib/api-base";
import { ROUTES } from "@/lib/routes";
import { UI_BUTTON_PRIMARY, UI_FIELD } from "@/lib/ui";

type RegisterFormData = {
  email: string;
  password: string;
  confirm_password: string;
  display_name: string;
};

export function RegisterForm() {
  const router = useRouter();
  const { t } = useLocale();
  const [form, setForm] = useState<RegisterFormData>({
    email: "",
    password: "",
    confirm_password: "",
    display_name: "",
  });
  const [fieldErrors, setFieldErrors] = useState<Partial<Record<keyof RegisterFormData, string>>>({});
  const [formError, setFormError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  const canSubmit = useMemo(() => !submitting, [submitting]);

  const navigateAfterAuth = () => {
    const destination = `${ROUTES.authPasskeyAdd}?source=signup&next=${encodeURIComponent(ROUTES.modules.train)}`;
    if (typeof window !== "undefined") {
      // Full navigation ensures persistent root layout re-resolves authenticated user state.
      try {
        window.location.assign(destination);
      } catch {
        router.push(destination);
      }
      return;
    }
    router.push(destination);
  };

  const onSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setFieldErrors({});
    setFormError(null);

    const registerSchema = z.object({
      email: z.string().email(t("auth.invalidEmail")),
      password: z.string().min(8, t("auth.passwordMin")),
      confirm_password: z.string().min(8, t("auth.passwordMin")),
      display_name: z
        .string()
        .trim()
        .min(1, t("auth.displayNameRequired"))
        .max(255, t("auth.displayNameMax")),
    }).refine((value) => value.password === value.confirm_password, {
      path: ["confirm_password"],
      message: t("auth.passwordConfirmMismatch"),
    });

    const payload = {
      email: form.email.trim(),
      password: form.password,
      confirm_password: form.confirm_password,
      display_name: form.display_name.trim(),
    };

    const parsed = registerSchema.safeParse(payload);
    if (!parsed.success) {
      const nextErrors: Partial<Record<keyof RegisterFormData, string>> = {};
      for (const issue of parsed.error.issues) {
        const key = issue.path[0] as keyof RegisterFormData;
        if (!nextErrors[key]) {
          nextErrors[key] = issue.message;
        }
      }
      setFieldErrors(nextErrors);
      return;
    }

    setSubmitting(true);
    try {
      const registerPayload = {
        email: parsed.data.email,
        password: parsed.data.password,
        display_name: parsed.data.display_name,
      };
      const response = await fetch(`${clientApiBaseUrl}/api/auth/register`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        credentials: "include",
        body: JSON.stringify(registerPayload),
      });

      if (!response.ok) {
        const body = (await response.json().catch(() => null)) as { detail?: string } | null;
        setFormError(body?.detail ?? t("auth.createAccountFailed"));
        return;
      }

      const loginResponse = await fetch(`${clientApiBaseUrl}/api/auth/login`, {
        method: "POST",
        headers: { "Content-Type": "application/json", "x-bominal-flow-source": "signup" },
        credentials: "include",
        body: JSON.stringify({
          email: parsed.data.email,
          password: parsed.data.password,
          remember_me: true,
        }),
      });
      if (!loginResponse.ok) {
        router.push("/login?registered=1");
        return;
      }
      navigateAfterAuth();
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
        <label className="mb-1 block text-sm font-medium text-slate-700" htmlFor="display_name">
          {t("auth.displayName")}
        </label>
        <input
          id="display_name"
          type="text"
          value={form.display_name}
          onChange={(event) => setForm((prev) => ({ ...prev, display_name: event.target.value }))}
          className={UI_FIELD}
          autoComplete="name"
          required
        />
        {fieldErrors.display_name ? <p className="mt-1 text-xs text-rose-600">{fieldErrors.display_name}</p> : null}
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
          autoComplete="new-password"
          required
        />
        {fieldErrors.password ? <p className="mt-1 text-xs text-rose-600">{fieldErrors.password}</p> : null}
      </div>

      <div>
        <label className="mb-1 block text-sm font-medium text-slate-700" htmlFor="confirm_password">
          {t("settings.confirmNewPassword")}
        </label>
        <input
          id="confirm_password"
          type="password"
          value={form.confirm_password}
          onChange={(event) => setForm((prev) => ({ ...prev, confirm_password: event.target.value }))}
          className={UI_FIELD}
          autoComplete="new-password"
          required
        />
        {fieldErrors.confirm_password ? (
          <p data-testid="register-confirm-password-error" className="mt-1 text-xs text-rose-600">
            {fieldErrors.confirm_password}
          </p>
        ) : null}
      </div>

      {formError ? <p className="rounded-xl bg-rose-50 px-3 py-2 text-sm text-rose-700">{formError}</p> : null}

      <button
        type="submit"
        disabled={!canSubmit}
        className={`w-full ${UI_BUTTON_PRIMARY}`}
      >
        {submitting ? t("auth.creatingAccount") : t("auth.register")}
      </button>
    </form>
  );
}
