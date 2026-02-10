"use client";

import { useCallback, useEffect, useState } from "react";

import { clientApiBaseUrl } from "@/lib/api-base";
import { UI_BUTTON_OUTLINE_SM, UI_BUTTON_PRIMARY, UI_CARD_MD, UI_KICKER, UI_TITLE_MD } from "@/lib/ui";

type OpsStatusResponse = {
  redis: { ok: boolean; detail?: string | null };
  arq: { queued: number; in_progress: number };
  worker: { online: boolean; last_heartbeat_at?: string | null };
  train: { active_task_count: number; stale_task_count: number };
};

export function OpsStatusCard() {
  const [status, setStatus] = useState<OpsStatusResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [actionLoading, setActionLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    setNotice(null);
    try {
      const res = await fetch(`${clientApiBaseUrl}/api/admin/ops/status`, { credentials: "include" });
      if (!res.ok) {
        setError("Failed to load ops status");
        return;
      }
      const data = (await res.json()) as OpsStatusResponse;
      setStatus(data);
    } catch {
      setError("Connection error");
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  const recover = async () => {
    const confirmed = window.confirm("Recover and re-enqueue eligible train tasks now?");
    if (!confirmed) return;

    setActionLoading(true);
    setError(null);
    setNotice(null);
    try {
      const res = await fetch(`${clientApiBaseUrl}/api/admin/ops/train/recover`, {
        method: "POST",
        credentials: "include",
      });
      const payload = (await res.json().catch(() => null)) as { enqueued_count?: number; detail?: unknown } | null;
      if (!res.ok) {
        const detail = typeof payload?.detail === "string" ? payload?.detail : "Recover failed";
        setError(detail);
        return;
      }
      setNotice(`Recovered ${payload?.enqueued_count ?? 0} task(s).`);
      await load();
    } catch {
      setError("Recover failed");
    } finally {
      setActionLoading(false);
    }
  };

  return (
    <section className={UI_CARD_MD}>
      <div className="flex items-start justify-between gap-3">
        <div>
          <p className={UI_KICKER}>Ops</p>
          <h2 className={`mt-2 ${UI_TITLE_MD}`}>System Status</h2>
        </div>
        <div className="flex items-center gap-2">
          <button type="button" className={UI_BUTTON_OUTLINE_SM} onClick={() => void load()} disabled={loading || actionLoading}>
            Refresh
          </button>
          <button type="button" className={UI_BUTTON_PRIMARY} onClick={() => void recover()} disabled={loading || actionLoading}>
            {actionLoading ? "Recovering..." : "Recover Tasks"}
          </button>
        </div>
      </div>

      {error ? <p className="mt-4 rounded-xl bg-rose-50 px-3 py-2 text-sm text-rose-700">{error}</p> : null}
      {notice ? <p className="mt-4 rounded-xl bg-emerald-50 px-3 py-2 text-sm text-emerald-700">{notice}</p> : null}

      {loading ? (
        <div className="mt-6 text-sm text-slate-400">Loading...</div>
      ) : status ? (
        <div className="mt-6 grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-4">
          <div className="rounded-2xl border border-blossom-100 bg-white p-4">
            <p className="text-xs font-medium uppercase tracking-wide text-slate-400">Worker</p>
            <p className="mt-1 text-lg font-semibold text-slate-800">{status.worker.online ? "Online" : "Offline"}</p>
            <p className="mt-1 text-xs text-slate-500">
              {status.worker.last_heartbeat_at ? `Last: ${new Date(status.worker.last_heartbeat_at).toLocaleString()}` : "No heartbeat"}
            </p>
          </div>
          <div className="rounded-2xl border border-blossom-100 bg-white p-4">
            <p className="text-xs font-medium uppercase tracking-wide text-slate-400">Redis</p>
            <p className="mt-1 text-lg font-semibold text-slate-800">{status.redis.ok ? "OK" : "Degraded"}</p>
            <p className="mt-1 text-xs text-slate-500">{status.redis.ok ? "Ping OK" : status.redis.detail ?? "Ping failed"}</p>
          </div>
          <div className="rounded-2xl border border-blossom-100 bg-white p-4">
            <p className="text-xs font-medium uppercase tracking-wide text-slate-400">Queue</p>
            <p className="mt-1 text-sm text-slate-700">
              <span className="font-semibold">Queued:</span> {status.arq.queued}
            </p>
            <p className="mt-1 text-sm text-slate-700">
              <span className="font-semibold">In progress:</span> {status.arq.in_progress}
            </p>
          </div>
          <div className="rounded-2xl border border-blossom-100 bg-white p-4">
            <p className="text-xs font-medium uppercase tracking-wide text-slate-400">Train</p>
            <p className="mt-1 text-sm text-slate-700">
              <span className="font-semibold">Active:</span> {status.train.active_task_count}
            </p>
            <p className="mt-1 text-sm text-slate-700">
              <span className="font-semibold">Stale:</span> {status.train.stale_task_count}
            </p>
          </div>
        </div>
      ) : (
        <div className="mt-6 text-sm text-slate-400">No data</div>
      )}
    </section>
  );
}

