"use client";

import { FormEvent, useMemo, useState } from "react";
import { useRouter } from "next/navigation";

import { clientApiBaseUrl } from "@/lib/api-base";
import type { BominalUser } from "@/lib/types";
import { UI_BUTTON_PRIMARY, UI_CARD_MD, UI_FIELD, UI_KICKER, UI_TITLE_MD } from "@/lib/ui";

type AccountFormState = {
  email: string;
  display_name: string;
  phone_number: string;
  billing_address_line1: string;
  billing_address_line2: string;
  billing_city: string;
  billing_state_province: string;
  billing_country: string;
  billing_postal_code: string;
  birthday: string;
  new_password: string;
  new_password_confirm: string;
  current_password: string;
};

const DELETE_ACCOUNT_BUTTON_CLASS =
  "inline-flex h-10 items-center justify-center rounded-full border border-rose-200 bg-white px-4 text-sm font-medium text-rose-700 transition hover:bg-rose-50 focus:outline-none focus:ring-2 focus:ring-rose-100 disabled:cursor-not-allowed disabled:opacity-60";

function normalizeBirthdayInput(value: string | null): string {
  if (!value) return "";
  return value.slice(0, 10);
}

function normalize(value: string): string {
  return value.trim();
}

function passwordStrengthLabel(password: string): "weak" | "good" | "excellent" {
  let score = 0;
  if (password.length >= 8) score += 1;
  if (password.length >= 12) score += 1;
  if (/[a-z]/.test(password) && /[A-Z]/.test(password)) score += 1;
  if (/\d/.test(password)) score += 1;
  if (/[^A-Za-z0-9]/.test(password)) score += 1;

  if (score <= 2) return "weak";
  if (score <= 4) return "good";
  return "excellent";
}

function buildInitialForm(user: BominalUser): AccountFormState {
  return {
    email: user.email ?? "",
    display_name: user.display_name ?? "",
    phone_number: user.phone_number ?? "",
    billing_address_line1: user.billing_address_line1 ?? "",
    billing_address_line2: user.billing_address_line2 ?? "",
    billing_city: user.billing_city ?? "",
    billing_state_province: user.billing_state_province ?? "",
    billing_country: user.billing_country ?? "",
    billing_postal_code: user.billing_postal_code ?? "",
    birthday: normalizeBirthdayInput(user.birthday),
    new_password: "",
    new_password_confirm: "",
    current_password: "",
  };
}

async function parseApiErrorMessage(response: Response, fallback: string): Promise<string> {
  const contentType = response.headers.get("content-type") ?? "";
  if (contentType.includes("application/json")) {
    const payload = (await response.json().catch(() => null)) as { detail?: string } | null;
    return payload?.detail ?? fallback;
  }
  const text = await response.text().catch(() => "");
  return text.trim() || fallback;
}

export function AccountSettingsPanel({ initialUser }: { initialUser: BominalUser }) {
  const router = useRouter();
  const [form, setForm] = useState<AccountFormState>(buildInitialForm(initialUser));
  const [baseline, setBaseline] = useState<AccountFormState>(buildInitialForm(initialUser));
  const [submitting, setSubmitting] = useState(false);
  const [deletingAccount, setDeletingAccount] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  const passwordStrength = useMemo(() => {
    if (!form.new_password) return null;
    return passwordStrengthLabel(form.new_password);
  }, [form.new_password]);

  const hasChanges = useMemo(() => {
    return (
      normalize(form.email).toLowerCase() !== normalize(baseline.email).toLowerCase() ||
      normalize(form.display_name) !== normalize(baseline.display_name) ||
      normalize(form.phone_number) !== normalize(baseline.phone_number) ||
      normalize(form.billing_address_line1) !== normalize(baseline.billing_address_line1) ||
      normalize(form.billing_address_line2) !== normalize(baseline.billing_address_line2) ||
      normalize(form.billing_city) !== normalize(baseline.billing_city) ||
      normalize(form.billing_state_province) !== normalize(baseline.billing_state_province) ||
      normalize(form.billing_country) !== normalize(baseline.billing_country) ||
      normalize(form.billing_postal_code) !== normalize(baseline.billing_postal_code) ||
      form.birthday !== baseline.birthday ||
      form.new_password.length > 0
    );
  }, [form, baseline]);

  const onSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setError(null);
    setNotice(null);

    if (!hasChanges) {
      setNotice("No changes to save.");
      return;
    }

    if (form.new_password && form.new_password !== form.new_password_confirm) {
      setError("New password confirmation does not match.");
      return;
    }

    if (!form.current_password) {
      setError("Current password is required to apply changes.");
      return;
    }

    setSubmitting(true);
    try {
      const payload: Record<string, string | null> = {
        current_password: form.current_password,
      };

      const nextEmail = normalize(form.email).toLowerCase();
      if (nextEmail !== normalize(baseline.email).toLowerCase()) {
        payload.email = nextEmail;
      }

      const nextDisplayName = normalize(form.display_name);
      if (nextDisplayName !== normalize(baseline.display_name)) {
        payload.display_name = nextDisplayName || null;
      }

      const nextPhone = normalize(form.phone_number);
      if (nextPhone !== normalize(baseline.phone_number)) {
        payload.phone_number = nextPhone || null;
      }

      const nextAddressLine1 = normalize(form.billing_address_line1);
      if (nextAddressLine1 !== normalize(baseline.billing_address_line1)) {
        payload.billing_address_line1 = nextAddressLine1 || null;
      }

      const nextAddressLine2 = normalize(form.billing_address_line2);
      if (nextAddressLine2 !== normalize(baseline.billing_address_line2)) {
        payload.billing_address_line2 = nextAddressLine2 || null;
      }

      const nextCity = normalize(form.billing_city);
      if (nextCity !== normalize(baseline.billing_city)) {
        payload.billing_city = nextCity || null;
      }

      const nextStateProvince = normalize(form.billing_state_province);
      if (nextStateProvince !== normalize(baseline.billing_state_province)) {
        payload.billing_state_province = nextStateProvince || null;
      }

      const nextCountry = normalize(form.billing_country);
      if (nextCountry !== normalize(baseline.billing_country)) {
        payload.billing_country = nextCountry || null;
      }

      const nextPostalCode = normalize(form.billing_postal_code);
      if (nextPostalCode !== normalize(baseline.billing_postal_code)) {
        payload.billing_postal_code = nextPostalCode || null;
      }

      if (form.birthday !== baseline.birthday) {
        payload.birthday = form.birthday || null;
      }

      if (form.new_password) {
        payload.new_password = form.new_password;
      }

      const response = await fetch(`${clientApiBaseUrl}/api/auth/account`, {
        method: "PATCH",
        credentials: "include",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(payload),
      });
      if (!response.ok) {
        setError(await parseApiErrorMessage(response, "Could not update account settings."));
        return;
      }

      const body = (await response.json()) as { user: BominalUser };
      const refreshed = buildInitialForm(body.user);
      setBaseline(refreshed);
      setForm({
        ...refreshed,
        new_password: "",
        new_password_confirm: "",
        current_password: "",
      });
      setNotice("Account settings updated.");
      router.refresh();
    } catch {
      setError("Could not update account settings.");
    } finally {
      setSubmitting(false);
    }
  };

  const onDeleteAccount = async () => {
    setError(null);
    setNotice(null);
    const confirmed = window.confirm("All your data will be permanently deleted. Are you sure you want to continue?");
    if (!confirmed) return;

    setDeletingAccount(true);
    try {
      const response = await fetch(`${clientApiBaseUrl}/api/auth/account`, {
        method: "DELETE",
        credentials: "include",
      });
      if (!response.ok) {
        setError(await parseApiErrorMessage(response, "Could not delete account."));
        return;
      }
      router.push("/login");
      router.refresh();
    } catch {
      setError("Could not delete account.");
    } finally {
      setDeletingAccount(false);
    }
  };

  return (
    <section>
      <div className={UI_CARD_MD}>
        <p className={UI_KICKER}>Settings</p>
        <h1 className={`mt-1 ${UI_TITLE_MD}`}>Account settings</h1>
        <p className="mt-2 text-sm text-slate-600">Update profile details. Any change requires your current password.</p>

        {error ? <p className="mt-4 rounded-xl bg-rose-50 px-3 py-2 text-sm text-rose-700">{error}</p> : null}
        {notice ? <p className="mt-4 rounded-xl bg-emerald-50 px-3 py-2 text-sm text-emerald-700">{notice}</p> : null}

        <form onSubmit={onSubmit} className="mt-5 grid gap-3 md:grid-cols-2">
          <p className="mt-2 text-xs uppercase tracking-[0.16em] text-blossom-500 md:col-span-2">General</p>

          <label className="text-sm text-slate-700">
            Email
            <input
              type="email"
              value={form.email}
              onChange={(event) => setForm((current) => ({ ...current, email: event.target.value }))}
              className={`mt-1 ${UI_FIELD}`}
              required
            />
          </label>

          <label className="text-sm text-slate-700">
            Display name
            <input
              type="text"
              value={form.display_name}
              onChange={(event) => setForm((current) => ({ ...current, display_name: event.target.value }))}
              className={`mt-1 ${UI_FIELD}`}
              placeholder="Optional"
            />
          </label>

          <label className="text-sm text-slate-700">
            Phone number
            <input
              type="text"
              value={form.phone_number}
              onChange={(event) => setForm((current) => ({ ...current, phone_number: event.target.value }))}
              className={`mt-1 ${UI_FIELD}`}
              placeholder="Optional"
            />
          </label>

          <label className="text-sm text-slate-700">
            Birthday
            <input
              type="date"
              value={form.birthday}
              onChange={(event) => setForm((current) => ({ ...current, birthday: event.target.value }))}
              className={`mt-1 ${UI_FIELD}`}
            />
          </label>

          <p className="mt-2 text-xs uppercase tracking-[0.16em] text-blossom-500 md:col-span-2">Billing address</p>

          <label className="text-sm text-slate-700">
            Address 1
            <input
              type="text"
              value={form.billing_address_line1}
              onChange={(event) => setForm((current) => ({ ...current, billing_address_line1: event.target.value }))}
              className={`mt-1 ${UI_FIELD}`}
              placeholder="Optional"
            />
          </label>

          <label className="text-sm text-slate-700">
            Address 2
            <input
              type="text"
              value={form.billing_address_line2}
              onChange={(event) => setForm((current) => ({ ...current, billing_address_line2: event.target.value }))}
              className={`mt-1 ${UI_FIELD}`}
              placeholder="Optional"
            />
          </label>

          <label className="text-sm text-slate-700">
            City
            <input
              type="text"
              value={form.billing_city}
              onChange={(event) => setForm((current) => ({ ...current, billing_city: event.target.value }))}
              className={`mt-1 ${UI_FIELD}`}
              placeholder="Optional"
            />
          </label>

          <label className="text-sm text-slate-700">
            State/Province
            <input
              type="text"
              value={form.billing_state_province}
              onChange={(event) =>
                setForm((current) => ({ ...current, billing_state_province: event.target.value }))
              }
              className={`mt-1 ${UI_FIELD}`}
              placeholder="Optional"
            />
          </label>

          <label className="text-sm text-slate-700">
            Country
            <input
              type="text"
              value={form.billing_country}
              onChange={(event) => setForm((current) => ({ ...current, billing_country: event.target.value }))}
              className={`mt-1 ${UI_FIELD}`}
              placeholder="Optional"
            />
          </label>

          <label className="text-sm text-slate-700">
            ZIP/Postal code
            <input
              type="text"
              value={form.billing_postal_code}
              onChange={(event) => setForm((current) => ({ ...current, billing_postal_code: event.target.value }))}
              className={`mt-1 ${UI_FIELD}`}
              placeholder="Optional"
            />
          </label>

          <p className="mt-2 text-xs uppercase tracking-[0.16em] text-blossom-500 md:col-span-2">Security</p>

          <label className="text-sm text-slate-700">
            New password
            <input
              type="password"
              value={form.new_password}
              onChange={(event) => setForm((current) => ({ ...current, new_password: event.target.value }))}
              className={`mt-1 ${UI_FIELD}`}
              placeholder="Optional"
            />
            {passwordStrength ? (
              <span className="mt-2 block text-xs text-slate-500">
                <span className="mb-1 block">Strength: {passwordStrength}</span>
                <span className="block h-1.5 w-full overflow-hidden rounded-full bg-slate-100">
                  <span
                    className={`block h-full rounded-full transition-all ${
                      passwordStrength === "weak"
                        ? "w-1/3 bg-rose-400"
                        : passwordStrength === "good"
                          ? "w-2/3 bg-amber-400"
                          : "w-full bg-emerald-500"
                    }`}
                  />
                </span>
              </span>
            ) : null}
          </label>

          <label className="text-sm text-slate-700">
            Confirm new password
            <input
              type="password"
              value={form.new_password_confirm}
              onChange={(event) =>
                setForm((current) => ({ ...current, new_password_confirm: event.target.value }))
              }
              className={`mt-1 ${UI_FIELD}`}
              placeholder="Required when changing password"
            />
          </label>

          <label className="text-sm text-slate-700 md:col-span-2">
            Current password
            <input
              type="password"
              value={form.current_password}
              onChange={(event) => setForm((current) => ({ ...current, current_password: event.target.value }))}
              className={`mt-1 ${UI_FIELD}`}
              placeholder="Required to save changes"
            />
          </label>

          <div className="md:col-span-2">
            <button type="submit" disabled={submitting || deletingAccount} className={UI_BUTTON_PRIMARY}>
              {submitting ? "Saving..." : "Save account settings"}
            </button>
          </div>
        </form>

        <div className="mt-8 border-t border-rose-100 pt-5">
          <p className="text-xs uppercase tracking-[0.16em] text-rose-500">Danger zone</p>
          <p className="mt-2 text-sm text-slate-600">
            Delete your account and remove saved profile data. Outstanding worker instances must be completed first.
          </p>
          <button
            type="button"
            onClick={onDeleteAccount}
            disabled={submitting || deletingAccount}
            className={`mt-3 ${DELETE_ACCOUNT_BUTTON_CLASS}`}
          >
            {deletingAccount ? "Deleting account..." : "Delete account"}
          </button>
        </div>
      </div>
    </section>
  );
}
