"use client";

import { useCallback, useEffect, useState } from "react";

import { clientApiBaseUrl } from "@/lib/api-base";
import { UI_BUTTON_OUTLINE_SM, UI_CARD_MD, UI_KICKER, UI_TITLE_MD } from "@/lib/ui";

type FailureRow = {
  task_id: string;
  user_email: string;
  action: string;
  provider: string;
  error_code: string | null;
  error_message_safe: string | null;
  started_at: string;
  finished_at: string;
};

type RecentFailuresResponse = {
  failures: FailureRow[];
};

function formatDate(value: string | null) {
  if (!value) return "-";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return "-";
  return date.toLocaleString();
}

export function RecentFailuresTable() {
  const [rows, setRows] = useState<FailureRow[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const res = await fetch(`${clientApiBaseUrl}/api/admin/ops/train/recent-failures?hours=24&limit=50`, {
        credentials: "include",
      });
      if (!res.ok) {
        setError("Failed to load recent failures");
        return;
      }
      const payload = (await res.json()) as RecentFailuresResponse;
      setRows(payload.failures ?? []);
    } catch {
      setError("Connection error");
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  return (
    <section className={UI_CARD_MD}>
      <div className="flex items-start justify-between gap-3">
        <div>
          <p className={UI_KICKER}>Ops</p>
          <h2 className={`mt-2 ${UI_TITLE_MD}`}>Recent Failures</h2>
          <p className="mt-2 text-sm text-slate-500">Latest failed task attempts (last 24h).</p>
        </div>
        <button type="button" className={UI_BUTTON_OUTLINE_SM} onClick={() => void load()} disabled={loading}>
          Refresh
        </button>
      </div>

      {error ? <p className="mt-4 rounded-xl bg-rose-50 px-3 py-2 text-sm text-rose-700">{error}</p> : null}

      <div className="mt-4 overflow-hidden rounded-xl border border-blossom-100">
        <table className="w-full text-sm">
          <thead className="bg-blossom-50/50 text-left text-xs uppercase tracking-wide text-slate-500">
            <tr>
              <th className="px-4 py-3">Time</th>
              <th className="px-4 py-3">Task</th>
              <th className="px-4 py-3 hidden lg:table-cell">User</th>
              <th className="px-4 py-3">Action</th>
              <th className="px-4 py-3">Provider</th>
              <th className="px-4 py-3">Error</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-blossom-50">
            {loading ? (
              <tr>
                <td colSpan={6} className="px-4 py-8 text-center text-slate-400">
                  Loading...
                </td>
              </tr>
            ) : rows.length === 0 ? (
              <tr>
                <td colSpan={6} className="px-4 py-8 text-center text-slate-400">
                  No failures found
                </td>
              </tr>
            ) : (
              rows.map((row, idx) => (
                <tr key={`${row.task_id}:${idx}`} className="hover:bg-blossom-50/30">
                  <td className="px-4 py-3 text-xs text-slate-500">{formatDate(row.started_at)}</td>
                  <td className="px-4 py-3 font-mono text-xs text-slate-700">{row.task_id}</td>
                  <td className="px-4 py-3 hidden lg:table-cell text-xs text-slate-500">{row.user_email}</td>
                  <td className="px-4 py-3 text-slate-700">{row.action}</td>
                  <td className="px-4 py-3 text-slate-700">{row.provider}</td>
                  <td className="px-4 py-3 text-xs text-slate-600">
                    {row.error_message_safe ? (
                      <span title={row.error_code ?? undefined}>{row.error_message_safe}</span>
                    ) : (
                      "-"
                    )}
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>
    </section>
  );
}

