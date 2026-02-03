"use client";

import { FormEvent, useMemo, useState } from "react";
import { useRouter } from "next/navigation";
import { z } from "zod";

import { clientApiBaseUrl } from "@/lib/api-base";
import { UI_BUTTON_PRIMARY, UI_FIELD } from "@/lib/ui";

const registerSchema = z.object({
  email: z.string().email("Please enter a valid email."),
  password: z.string().min(8, "Password must be at least 8 characters."),
  display_name: z
    .string()
    .trim()
    .min(1, "Display name is required.")
    .max(255, "Display name must be at most 255 characters."),
});

type RegisterFormData = {
  email: string;
  password: string;
  display_name: string;
};

export function RegisterForm() {
  const router = useRouter();
  const [form, setForm] = useState<RegisterFormData>({ email: "", password: "", display_name: "" });
  const [fieldErrors, setFieldErrors] = useState<Partial<Record<keyof RegisterFormData, string>>>({});
  const [formError, setFormError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  const canSubmit = useMemo(() => !submitting, [submitting]);

  const onSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setFieldErrors({});
    setFormError(null);

    const payload = {
      email: form.email.trim(),
      password: form.password,
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
      const response = await fetch(`${clientApiBaseUrl}/api/auth/register`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        credentials: "include",
        body: JSON.stringify(parsed.data),
      });

      if (!response.ok) {
        const body = (await response.json().catch(() => null)) as { detail?: string } | null;
        setFormError(body?.detail ?? "Could not create account.");
        return;
      }

      router.push("/login?registered=1");
    } catch {
      setFormError("Could not reach bominal API.");
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <form onSubmit={onSubmit} className="space-y-4">
      <div>
        <label className="mb-1 block text-sm font-medium text-slate-700" htmlFor="email">
          Email
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
          Display name
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
          Password
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

      {formError ? <p className="rounded-xl bg-rose-50 px-3 py-2 text-sm text-rose-700">{formError}</p> : null}

      <button
        type="submit"
        disabled={!canSubmit}
        className={`w-full ${UI_BUTTON_PRIMARY}`}
      >
        {submitting ? "Creating account..." : "Create account"}
      </button>
    </form>
  );
}
