"use client";

import { useCallback, useEffect, useState } from "react";

import { clientApiBaseUrl } from "@/lib/api-base";
import {
  UI_BUTTON_OUTLINE,
  UI_BUTTON_OUTLINE_SM,
  UI_CARD_MD,
  UI_KICKER,
  UI_TITLE_MD,
} from "@/lib/ui";

type AdminPaymentSettingsResponse = {
  payment_enabled: boolean;
  payment_enabled_env: boolean;
  payment_enabled_override: boolean;
  wallet_only: boolean;
  updated_at: string | null;
};

async function parseApiDetail(response: Response, fallback: string): Promise<string> {
  const contentType = response.headers.get("content-type") ?? "";
  if (contentType.includes("application/json")) {
    const payload = (await response.json().catch(() => null)) as { detail?: string } | null;
    if (payload?.detail) return payload.detail;
  }
  return fallback;
}

export function PaymentKillSwitchCard() {
  const [status, setStatus] = useState<AdminPaymentSettingsResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [toggling, setToggling] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const response = await fetch(`${clientApiBaseUrl}/api/admin/payment-settings`, {
        credentials: "include",
        cache: "no-store",
      });
      if (!response.ok) {
        setError(await parseApiDetail(response, "Failed to load payment settings"));
        return;
      }
      const payload = (await response.json()) as AdminPaymentSettingsResponse;
      setStatus(payload);
    } catch {
      setError("Failed to load payment settings");
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  const onToggleEnabled = async () => {
    if (!status) return;

    const nextEnabled = !status.payment_enabled_override;
    const prompt = nextEnabled
      ? "Enable Evervault payment dispatch globally?"
      : "Disable Evervault payment dispatch globally? This will block payment dispatch to Evervault relays and providers.";
    if (!window.confirm(prompt)) {
      return;
    }

    setToggling(true);
    setError(null);
    setNotice(null);
    try {
      const response = await fetch(`${clientApiBaseUrl}/api/admin/payment-settings/enabled`, {
        method: "PATCH",
        credentials: "include",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ enabled: nextEnabled }),
      });
      if (!response.ok) {
        setError(await parseApiDetail(response, "Failed to update payment toggle"));
        return;
      }
      const payload = (await response.json()) as AdminPaymentSettingsResponse;
      setStatus(payload);
      setNotice(nextEnabled ? "Evervault payment dispatch enabled." : "Evervault payment dispatch disabled.");
    } catch {
      setError("Failed to update payment toggle");
    } finally {
      setToggling(false);
    }
  };

  return (
    <section className={UI_CARD_MD}>
      <div className="flex items-start justify-between gap-3">
        <div>
          <p className={UI_KICKER}>Maintenance</p>
          <h2 className={`mt-2 ${UI_TITLE_MD}`}>Evervault Payment Kill Switch</h2>
          <p className="mt-2 text-sm text-slate-500">
            Payment dispatch is wallet-only. Disabling this switch blocks API and worker payment calls before they can reach
            Evervault relays or provider payment endpoints.
          </p>
        </div>
        <button
          type="button"
          className={UI_BUTTON_OUTLINE_SM}
          onClick={() => void load()}
          disabled={loading || toggling}
        >
          Refresh
        </button>
      </div>

      {error ? <p className="mt-4 rounded-xl bg-rose-50 px-3 py-2 text-sm text-rose-700">{error}</p> : null}
      {notice ? <p className="mt-4 rounded-xl bg-emerald-50 px-3 py-2 text-sm text-emerald-700">{notice}</p> : null}

      {loading ? (
        <p className="mt-4 text-sm text-slate-500">Loading...</p>
      ) : status ? (
        <>
          <div className="mt-5 grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
            <div className="rounded-xl border border-blossom-100 bg-white p-3">
              <p className="text-xs uppercase tracking-wide text-slate-400">Effective Runtime</p>
              <p className="mt-1 text-sm font-semibold text-slate-800">
                {status.payment_enabled ? "Dispatch Enabled" : "Dispatch Blocked"}
              </p>
            </div>
            <div className="rounded-xl border border-blossom-100 bg-white p-3">
              <p className="text-xs uppercase tracking-wide text-slate-400">Kill Switch Override</p>
              <p className="mt-1 text-sm font-semibold text-slate-800">
                {status.payment_enabled_override ? "Allow Dispatch" : "Block Dispatch"}
              </p>
            </div>
            <div className="rounded-xl border border-blossom-100 bg-white p-3">
              <p className="text-xs uppercase tracking-wide text-slate-400">Environment Gate</p>
              <p className="mt-1 text-sm font-semibold text-slate-800">
                {status.payment_enabled_env ? "PAYMENT_ENABLED=true" : "PAYMENT_ENABLED=false"}
              </p>
            </div>
            <div className="rounded-xl border border-blossom-100 bg-white p-3">
              <p className="text-xs uppercase tracking-wide text-slate-400">Credential Source</p>
              <p className="mt-1 text-sm font-semibold text-slate-800">
                {status.wallet_only ? "Wallet Evervault-Only" : "Invalid"}
              </p>
            </div>
          </div>

          <div className="mt-4 flex flex-wrap gap-2">
            <button
              type="button"
              onClick={() => void onToggleEnabled()}
              disabled={loading || toggling || !status.payment_enabled_env}
              className={`${UI_BUTTON_OUTLINE} ${
                status.payment_enabled_override
                  ? "border-rose-200 text-rose-700 hover:bg-rose-50"
                  : "border-emerald-200 text-emerald-700 hover:bg-emerald-50"
              }`}
            >
              {toggling
                ? "Updating..."
                : status.payment_enabled_override
                  ? "Disable Dispatch"
                  : "Enable Dispatch"}
            </button>
            {!status.payment_enabled_env ? (
              <p className="self-center text-xs text-amber-700">
                Environment is fail-closed (`PAYMENT_ENABLED=false`), so dispatch remains blocked.
              </p>
            ) : null}
          </div>

          {status.updated_at ? (
            <p className="mt-4 text-xs text-slate-500">
              Last updated: {new Date(status.updated_at).toLocaleString()}
            </p>
          ) : null}
        </>
      ) : null}
    </section>
  );
}
