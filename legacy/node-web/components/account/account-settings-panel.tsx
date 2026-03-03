"use client";

import { FormEvent, useEffect, useMemo, useState } from "react";
import { useRouter } from "next/navigation";
import { createPortal } from "react-dom";

import { useLocale } from "@/components/locale-provider";
import { clientApiBaseUrl } from "@/lib/api-base";
import {
  listPasskeysFromSession,
  registerPasskeyFromSession,
  removePasskeyFromSession,
  verifyPasskeyStepUpFromSession,
} from "@/lib/passkey";
import { ROUTES } from "@/lib/routes";
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
  email_change_code: string;
};

type PrefillEmailChange = {
  email: string;
  code: string;
};

type PasskeyListItem = {
  id: string;
  created_at: string;
  last_used_at: string | null;
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

function normalizeEmail(value: string): string {
  return normalize(value).toLowerCase();
}

type PasswordStrength = "weak" | "good" | "excellent";

const PASSWORD_STRENGTH_BAR_CLASS: Record<PasswordStrength, string> = {
  weak: "w-1/3 bg-rose-400",
  good: "w-2/3 bg-amber-400",
  excellent: "w-full bg-emerald-500",
};

function passwordStrengthLabel(password: string): PasswordStrength {
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

type AccountOptionalPatchKey =
  | "display_name"
  | "phone_number"
  | "billing_address_line1"
  | "billing_address_line2"
  | "billing_city"
  | "billing_state_province"
  | "billing_country"
  | "billing_postal_code";

type AccountPatchKey = AccountOptionalPatchKey | "email" | "birthday" | "new_password";
type AccountPatchPayload = Partial<Record<AccountPatchKey, string | null>>;

const OPTIONAL_PATCH_KEYS: AccountOptionalPatchKey[] = [
  "display_name",
  "phone_number",
  "billing_address_line1",
  "billing_address_line2",
  "billing_city",
  "billing_state_province",
  "billing_country",
  "billing_postal_code",
];

function setOptionalIfChanged(
  payload: AccountPatchPayload,
  key: AccountOptionalPatchKey,
  nextRaw: string,
  baselineRaw: string,
): void {
  const next = normalize(nextRaw);
  const baseline = normalize(baselineRaw);
  if (next !== baseline) {
    payload[key] = next || null;
  }
}

function buildAccountPatch(form: AccountFormState, baseline: AccountFormState): AccountPatchPayload {
  const payload: AccountPatchPayload = {};

  const nextEmail = normalizeEmail(form.email);
  if (nextEmail !== normalizeEmail(baseline.email)) {
    payload.email = nextEmail;
  }

  for (const key of OPTIONAL_PATCH_KEYS) {
    setOptionalIfChanged(payload, key, form[key], baseline[key]);
  }

  if (form.birthday !== baseline.birthday) {
    payload.birthday = form.birthday || null;
  }

  if (form.new_password) {
    payload.new_password = form.new_password;
  }

  return payload;
}

function isSensitiveAccountPatch(payload: AccountPatchPayload): boolean {
  return (
    Object.prototype.hasOwnProperty.call(payload, "email") ||
    Object.prototype.hasOwnProperty.call(payload, "new_password")
  );
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
    email_change_code: "",
  };
}

type SensitiveActionContext =
  | { type: "account_update"; payload: AccountPatchPayload }
  | { type: "add_passkey" };

async function parseApiErrorMessage(response: Response, fallback: string): Promise<string> {
  const contentType = response.headers.get("content-type") ?? "";
  if (contentType.includes("application/json")) {
    const payload = (await response.json().catch(() => null)) as { detail?: string } | null;
    return payload?.detail ?? fallback;
  }
  const text = await response.text().catch(() => "");
  return text.trim() || fallback;
}

export function AccountSettingsPanel({
  initialUser,
  prefillEmailChange,
}: {
  initialUser: BominalUser;
  prefillEmailChange?: PrefillEmailChange | null;
}) {
  const router = useRouter();
  const { t } = useLocale();
  const [form, setForm] = useState<AccountFormState>(() => buildInitialForm(initialUser));
  const [baseline, setBaseline] = useState<AccountFormState>(() => buildInitialForm(initialUser));
  const [submitting, setSubmitting] = useState(false);
  const [deletingAccount, setDeletingAccount] = useState(false);
  const [confirmingEmailChange, setConfirmingEmailChange] = useState(false);
  const [passkeyBusy, setPasskeyBusy] = useState(false);
  const [passkeyLoading, setPasskeyLoading] = useState(false);
  const [passkeys, setPasskeys] = useState<PasskeyListItem[]>([]);
  const [portalReady, setPortalReady] = useState(false);
  const [passwordPromptOpen, setPasswordPromptOpen] = useState(false);
  const [passwordPromptValue, setPasswordPromptValue] = useState("");
  const [passwordPromptBusy, setPasswordPromptBusy] = useState(false);
  const [pendingSensitiveAction, setPendingSensitiveAction] = useState<SensitiveActionContext | null>(null);
  const [pendingEmailChangeTo, setPendingEmailChangeTo] = useState<string | null>(prefillEmailChange?.email ?? null);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  useEffect(() => {
    if (prefillEmailChange?.code) {
      setForm((current) => ({ ...current, email_change_code: prefillEmailChange.code }));
    }
    if (prefillEmailChange?.email) {
      setPendingEmailChangeTo(prefillEmailChange.email);
    }
  }, [prefillEmailChange?.code, prefillEmailChange?.email]);

  useEffect(() => {
    setPortalReady(true);
  }, []);

  useEffect(() => {
    let active = true;
    const loadPasskeys = async () => {
      setPasskeyLoading(true);
      try {
        const response = await listPasskeysFromSession(clientApiBaseUrl);
        if (active) {
          setPasskeys(response.credentials);
        }
      } catch {
        if (active) {
          setPasskeys([]);
        }
      } finally {
        if (active) {
          setPasskeyLoading(false);
        }
      }
    };
    void loadPasskeys();
    return () => {
      active = false;
    };
  }, []);

  const passwordStrength = useMemo(() => {
    if (!form.new_password) return null;
    return passwordStrengthLabel(form.new_password);
  }, [form.new_password]);

  const patch = useMemo(() => {
    const payload = buildAccountPatch(form, baseline);
    const passwordChanged = Object.prototype.hasOwnProperty.call(payload, "new_password");
    const sensitiveChanged = isSensitiveAccountPatch(payload);
    const hasChanges = Object.keys(payload).length > 0;
    return { payload, passwordChanged, sensitiveChanged, hasChanges };
  }, [form, baseline]);

  const openPasswordPrompt = (action: SensitiveActionContext) => {
    setPendingSensitiveAction(action);
    setPasswordPromptValue("");
    setPasswordPromptOpen(true);
  };

  const closePasswordPrompt = () => {
    setPasswordPromptOpen(false);
    setPasswordPromptValue("");
    setPasswordPromptBusy(false);
    setPendingSensitiveAction(null);
  };

  const submitAccountPatch = async (
    payload: AccountPatchPayload,
    auth?: { currentPassword?: string; stepUpToken?: string },
  ): Promise<boolean> => {
    setSubmitting(true);
    try {
      const bodyPayload = {
        ...payload,
        ...(auth?.currentPassword ? { current_password: auth.currentPassword } : {}),
        ...(auth?.stepUpToken ? { passkey_step_up_token: auth.stepUpToken } : {}),
      };
      const response = await fetch(`${clientApiBaseUrl}/api/auth/account`, {
        method: "PATCH",
        credentials: "include",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(bodyPayload),
      });
      if (!response.ok) {
        setError(await parseApiErrorMessage(response, t("settings.updateFailed")));
        return false;
      }

      const body = (await response.json()) as {
        user: BominalUser;
        notice?: string;
        pending_email_change_to?: string | null;
      };
      const refreshed = buildInitialForm(body.user);
      setBaseline(refreshed);
      setForm(refreshed);
      setPendingEmailChangeTo(body.pending_email_change_to ?? null);
      setNotice(body.notice ?? t("settings.updated"));
      router.refresh();
      return true;
    } catch {
      setError(t("settings.updateFailed"));
      return false;
    } finally {
      setSubmitting(false);
    }
  };

  const attemptPasskeyStepUp = async (): Promise<string | null> => {
    const result = await verifyPasskeyStepUpFromSession(clientApiBaseUrl);
    if (result.ok && result.stepUpToken) {
      return result.stepUpToken;
    }
    return null;
  };

  const performAddPasskey = async (): Promise<boolean> => {
    setPasskeyBusy(true);
    try {
      const result = await registerPasskeyFromSession(clientApiBaseUrl);
      if (!result.ok) {
        setError(result.error ?? t("settings.passkeyAddFailed"));
        return false;
      }
      const refreshed = await listPasskeysFromSession(clientApiBaseUrl);
      setPasskeys(refreshed.credentials);
      setNotice(t("settings.passkeyAdded"));
      return true;
    } catch {
      setError(t("settings.passkeyAddFailed"));
      return false;
    } finally {
      setPasskeyBusy(false);
    }
  };

  const onSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setError(null);
    setNotice(null);

    if (!patch.hasChanges) {
      setNotice(t("settings.noChanges"));
      return;
    }

    if (patch.passwordChanged && form.new_password !== form.new_password_confirm) {
      setError(t("settings.passwordConfirmMismatch"));
      return;
    }

    if (!patch.sensitiveChanged) {
      await submitAccountPatch(patch.payload);
      return;
    }

    const stepUpToken = await attemptPasskeyStepUp();
    if (stepUpToken) {
      await submitAccountPatch(patch.payload, { stepUpToken });
      return;
    }

    openPasswordPrompt({ type: "account_update", payload: patch.payload });
  };

  const onDeleteAccount = async () => {
    setError(null);
    setNotice(null);
    const confirmed = window.confirm(t("settings.deleteConfirm"));
    if (!confirmed) return;

    setDeletingAccount(true);
    try {
      const response = await fetch(`${clientApiBaseUrl}/api/auth/account`, {
        method: "DELETE",
        credentials: "include",
      });
      if (!response.ok) {
        setError(await parseApiErrorMessage(response, t("settings.deleteFailed")));
        return;
      }
      router.push(ROUTES.login);
    } catch {
      setError(t("settings.deleteFailed"));
    } finally {
      setDeletingAccount(false);
    }
  };

  const onConfirmEmailChange = async () => {
    setError(null);
    setNotice(null);
    const targetEmail = (pendingEmailChangeTo || "").trim().toLowerCase();
    const code = form.email_change_code.trim();
    if (!targetEmail || !code) {
      setError(t("settings.emailChangeCodeRequired"));
      return;
    }

    setConfirmingEmailChange(true);
    try {
      const response = await fetch(`${clientApiBaseUrl}/api/auth/account/email-change/confirm`, {
        method: "POST",
        credentials: "include",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ email: targetEmail, code }),
      });
      if (!response.ok) {
        setError(await parseApiErrorMessage(response, t("settings.emailChangeConfirmFailed")));
        return;
      }
      const body = (await response.json()) as { user: BominalUser; notice?: string };
      const refreshed = buildInitialForm(body.user);
      refreshed.email_change_code = "";
      setBaseline(refreshed);
      setForm(refreshed);
      setPendingEmailChangeTo(null);
      setNotice(body.notice ?? t("settings.emailChangeConfirmed"));
      router.refresh();
    } catch {
      setError(t("settings.emailChangeConfirmFailed"));
    } finally {
      setConfirmingEmailChange(false);
    }
  };

  const onAddPasskey = async () => {
    setError(null);
    setNotice(null);
    openPasswordPrompt({ type: "add_passkey" });
  };

  const onConfirmPasswordPrompt = async () => {
    setError(null);
    if (!pendingSensitiveAction) return;

    const currentPassword = passwordPromptValue.trim();
    if (!currentPassword) {
      setError(t("settings.currentPasswordRequired"));
      return;
    }

    setPasswordPromptBusy(true);
    try {
      if (pendingSensitiveAction.type === "account_update") {
        const ok = await submitAccountPatch(pendingSensitiveAction.payload, { currentPassword });
        if (ok) {
          closePasswordPrompt();
        }
        return;
      }

      const verifyResponse = await fetch(`${clientApiBaseUrl}/api/auth/account/verify-password`, {
        method: "POST",
        credentials: "include",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ current_password: currentPassword }),
      });
      if (!verifyResponse.ok) {
        setError(await parseApiErrorMessage(verifyResponse, t("settings.currentPasswordRequiredSensitive")));
        return;
      }
      const added = await performAddPasskey();
      if (added) {
        closePasswordPrompt();
      }
    } catch {
      setError(t("settings.currentPasswordRequiredSensitive"));
    } finally {
      setPasswordPromptBusy(false);
    }
  };

  const onRemovePasskey = async (passkeyId: string) => {
    setError(null);
    setNotice(null);
    setPasskeyBusy(true);
    try {
      await removePasskeyFromSession(clientApiBaseUrl, passkeyId);
      setPasskeys((current) => current.filter((item) => item.id !== passkeyId));
      setNotice(t("settings.passkeyRemoved"));
    } catch {
      setError(t("settings.passkeyRemoveFailed"));
    } finally {
      setPasskeyBusy(false);
    }
  };

  const passwordPromptModal =
    passwordPromptOpen && portalReady
      ? createPortal(
          <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-900/45 p-4">
            <div className="w-full max-w-md rounded-2xl bg-white p-5 shadow-xl">
              <p className="text-sm font-semibold text-slate-900">{t("settings.sensitiveAuthTitle")}</p>
              <p className="mt-1 text-xs text-slate-600">{t("settings.sensitiveAuthBody")}</p>
              <label className="mt-4 block text-sm text-slate-700">
                {t("settings.currentPassword")}
                <input
                  type="password"
                  value={passwordPromptValue}
                  onChange={(event) => setPasswordPromptValue(event.target.value)}
                  className={`mt-1 ${UI_FIELD}`}
                  placeholder={t("settings.currentPasswordRequired")}
                  autoFocus
                />
              </label>
              <div className="mt-4 flex justify-end gap-2">
                <button
                  type="button"
                  onClick={closePasswordPrompt}
                  disabled={passwordPromptBusy}
                  className="inline-flex h-10 items-center justify-center rounded-full border border-slate-200 bg-white px-4 text-sm font-medium text-slate-700 transition hover:bg-slate-50 disabled:cursor-not-allowed disabled:opacity-60"
                >
                  {t("common.cancel")}
                </button>
                <button
                  type="button"
                  onClick={onConfirmPasswordPrompt}
                  disabled={passwordPromptBusy}
                  className={`inline-flex h-10 items-center justify-center ${UI_BUTTON_PRIMARY}`}
                >
                  {passwordPromptBusy ? t("common.saving") : t("settings.sensitiveAuthConfirm")}
                </button>
              </div>
            </div>
          </div>,
          document.body,
        )
      : null;

  return (
    <section>
      <div className={UI_CARD_MD}>
        <p className={UI_KICKER}>{t("settings.kicker")}</p>
        <h1 className={`mt-1 ${UI_TITLE_MD}`}>{t("settings.accountTitle")}</h1>
        <p className="mt-2 text-sm text-slate-600">{t("settings.accountBody")}</p>

        {error ? <p className="mt-4 rounded-xl bg-rose-50 px-3 py-2 text-sm text-rose-700">{error}</p> : null}
        {notice ? <p className="mt-4 rounded-xl bg-emerald-50 px-3 py-2 text-sm text-emerald-700">{notice}</p> : null}

        <form onSubmit={onSubmit} className="mt-5 grid gap-3 md:grid-cols-2">
          <p className="mt-2 text-xs uppercase tracking-[0.16em] text-blossom-500 md:col-span-2">
            {t("settings.generalSection")}
          </p>

          <label className="text-sm text-slate-700">
            {t("settings.email")}
            <input
              type="email"
              value={form.email}
              onChange={(event) => setForm((current) => ({ ...current, email: event.target.value }))}
              className={`mt-1 ${UI_FIELD}`}
              required
            />
          </label>

          <label className="text-sm text-slate-700">
            {t("settings.displayName")}
            <input
              type="text"
              value={form.display_name}
              onChange={(event) => setForm((current) => ({ ...current, display_name: event.target.value }))}
              className={`mt-1 ${UI_FIELD}`}
              placeholder={t("common.optional")}
            />
          </label>

          <label className="text-sm text-slate-700">
            {t("settings.phone")}
            <input
              type="text"
              value={form.phone_number}
              onChange={(event) => setForm((current) => ({ ...current, phone_number: event.target.value }))}
              className={`mt-1 ${UI_FIELD}`}
              placeholder={t("common.optional")}
            />
          </label>

          <label className="text-sm text-slate-700">
            {t("settings.birthday")}
            <input
              type="date"
              value={form.birthday}
              onChange={(event) => setForm((current) => ({ ...current, birthday: event.target.value }))}
              className={`mt-1 ${UI_FIELD}`}
            />
          </label>

          <p className="mt-2 text-xs uppercase tracking-[0.16em] text-blossom-500 md:col-span-2">
            {t("settings.billingAddressSection")}
          </p>

          <label className="text-sm text-slate-700">
            {t("settings.billingAddress1")}
            <input
              type="text"
              value={form.billing_address_line1}
              onChange={(event) => setForm((current) => ({ ...current, billing_address_line1: event.target.value }))}
              className={`mt-1 ${UI_FIELD}`}
              placeholder={t("common.optional")}
            />
          </label>

          <label className="text-sm text-slate-700">
            {t("settings.billingAddress2")}
            <input
              type="text"
              value={form.billing_address_line2}
              onChange={(event) => setForm((current) => ({ ...current, billing_address_line2: event.target.value }))}
              className={`mt-1 ${UI_FIELD}`}
              placeholder={t("common.optional")}
            />
          </label>

          <label className="text-sm text-slate-700">
            {t("settings.billingCity")}
            <input
              type="text"
              value={form.billing_city}
              onChange={(event) => setForm((current) => ({ ...current, billing_city: event.target.value }))}
              className={`mt-1 ${UI_FIELD}`}
              placeholder={t("common.optional")}
            />
          </label>

          <label className="text-sm text-slate-700">
            {t("settings.billingState")}
            <input
              type="text"
              value={form.billing_state_province}
              onChange={(event) =>
                setForm((current) => ({ ...current, billing_state_province: event.target.value }))
              }
              className={`mt-1 ${UI_FIELD}`}
              placeholder={t("common.optional")}
            />
          </label>

          <label className="text-sm text-slate-700">
            {t("settings.billingCountry")}
            <input
              type="text"
              value={form.billing_country}
              onChange={(event) => setForm((current) => ({ ...current, billing_country: event.target.value }))}
              className={`mt-1 ${UI_FIELD}`}
              placeholder={t("common.optional")}
            />
          </label>

          <label className="text-sm text-slate-700">
            {t("settings.billingPostal")}
            <input
              type="text"
              value={form.billing_postal_code}
              onChange={(event) => setForm((current) => ({ ...current, billing_postal_code: event.target.value }))}
              className={`mt-1 ${UI_FIELD}`}
              placeholder={t("common.optional")}
            />
          </label>

          <p className="mt-2 text-xs uppercase tracking-[0.16em] text-blossom-500 md:col-span-2">
            {t("settings.securitySection")}
          </p>

          <label className="text-sm text-slate-700">
            {t("settings.newPassword")}
            <input
              type="password"
              value={form.new_password}
              onChange={(event) => setForm((current) => ({ ...current, new_password: event.target.value }))}
              className={`mt-1 ${UI_FIELD}`}
              placeholder={t("common.optional")}
            />
            {passwordStrength ? (
              <span className="mt-2 block text-xs text-slate-500">
                <span className="mb-1 block">
                  {t("settings.passwordStrength.label")}: {t(`settings.passwordStrength.${passwordStrength}`)}
                </span>
                <span className="block h-1.5 w-full overflow-hidden rounded-full bg-slate-100">
                  <span
                    className={`block h-full rounded-full transition-all ${PASSWORD_STRENGTH_BAR_CLASS[passwordStrength]}`}
                  />
                </span>
              </span>
            ) : null}
          </label>

          <label className="text-sm text-slate-700">
            {t("settings.confirmNewPassword")}
            <input
              type="password"
              value={form.new_password_confirm}
              onChange={(event) =>
                setForm((current) => ({ ...current, new_password_confirm: event.target.value }))
              }
              className={`mt-1 ${UI_FIELD}`}
              placeholder={t("settings.passwordConfirmRequired")}
            />
          </label>

          {pendingEmailChangeTo ? (
            <div className="rounded-2xl border border-amber-200 bg-amber-50 p-3 md:col-span-2">
              <p className="text-sm font-medium text-amber-800">{t("settings.emailChangePendingTitle")}</p>
              <p className="mt-1 text-xs text-amber-700">
                {t("settings.emailChangePendingBody", { email: pendingEmailChangeTo })}
              </p>
              <label className="mt-3 block text-sm text-slate-700">
                {t("settings.emailChangeCode")}
                <input
                  type="text"
                  value={form.email_change_code}
                  onChange={(event) => setForm((current) => ({ ...current, email_change_code: event.target.value }))}
                  className={`mt-1 ${UI_FIELD}`}
                  placeholder={t("settings.emailChangeCodePlaceholder")}
                />
              </label>
              <button
                type="button"
                onClick={onConfirmEmailChange}
                disabled={submitting || deletingAccount || confirmingEmailChange}
                className="mt-3 inline-flex h-10 items-center justify-center rounded-full border border-amber-300 bg-white px-4 text-sm font-medium text-amber-800 transition hover:bg-amber-100 disabled:cursor-not-allowed disabled:opacity-60"
              >
                {confirmingEmailChange ? t("common.saving") : t("settings.confirmEmailChange")}
              </button>
            </div>
          ) : null}

          <div className="rounded-2xl border border-blossom-100 bg-white p-3 md:col-span-2">
            <p className="text-sm font-medium text-slate-900">{t("settings.passkeyTitle")}</p>
            <p className="mt-1 text-xs text-slate-600">{t("settings.passkeyBody")}</p>
            <button
              type="button"
              onClick={onAddPasskey}
              disabled={submitting || deletingAccount || passkeyBusy}
              className="mt-3 inline-flex h-10 items-center justify-center rounded-full border border-blossom-200 bg-blossom-50 px-4 text-sm font-medium text-blossom-700 transition hover:bg-blossom-100 disabled:cursor-not-allowed disabled:opacity-60"
            >
              {passkeyBusy ? t("common.saving") : t("settings.addPasskey")}
            </button>
            {passkeyLoading ? (
              <p className="mt-2 text-xs text-slate-500">{t("common.loading")}</p>
            ) : passkeys.length === 0 ? (
              <p className="mt-2 text-xs text-slate-500">{t("settings.passkeyEmpty")}</p>
            ) : (
              <ul className="mt-3 space-y-2">
                {passkeys.map((item) => (
                  <li
                    key={item.id}
                    className="flex flex-wrap items-center justify-between gap-2 rounded-xl border border-slate-200 bg-slate-50 px-3 py-2 text-xs text-slate-700"
                  >
                    <div>
                      <p>{t("settings.passkeyCreatedAt", { date: new Date(item.created_at).toLocaleString() })}</p>
                      <p>
                        {item.last_used_at
                          ? t("settings.passkeyLastUsedAt", { date: new Date(item.last_used_at).toLocaleString() })
                          : t("settings.passkeyNeverUsed")}
                      </p>
                    </div>
                    <button
                      type="button"
                      onClick={() => onRemovePasskey(item.id)}
                      disabled={submitting || deletingAccount || passkeyBusy}
                      className="inline-flex h-8 items-center justify-center rounded-full border border-rose-200 bg-white px-3 text-xs font-medium text-rose-700 transition hover:bg-rose-50 disabled:cursor-not-allowed disabled:opacity-60"
                    >
                      {t("common.delete")}
                    </button>
                  </li>
                ))}
              </ul>
            )}
          </div>

          <div className="md:col-span-2">
            <button type="submit" disabled={submitting || deletingAccount} className={UI_BUTTON_PRIMARY}>
              {submitting ? t("common.saving") : t("settings.saveAccountSettings")}
            </button>
          </div>
        </form>

        <div className="mt-8 border-t border-rose-100 pt-5">
          <p className="text-xs uppercase tracking-[0.16em] text-rose-500">{t("settings.dangerZone")}</p>
          <p className="mt-2 text-sm text-slate-600">
            {t("settings.dangerBody")}
          </p>
          <button
            type="button"
            onClick={onDeleteAccount}
            disabled={submitting || deletingAccount}
            className={`mt-3 ${DELETE_ACCOUNT_BUTTON_CLASS}`}
          >
            {deletingAccount ? t("settings.deleting") : t("settings.deleteButton")}
          </button>
        </div>
      </div>

      {passwordPromptModal}
    </section>
  );
}
