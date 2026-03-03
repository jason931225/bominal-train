"use client";

import { useCallback, useEffect, useState } from "react";

import { clientApiBaseUrl } from "@/lib/api-base";
import { UI_BUTTON_OUTLINE_SM, UI_CARD_MD, UI_KICKER, UI_TITLE_MD } from "@/lib/ui";

type StaleTask = {
  task_id: string;
  state: string;
  created_at: string;
  updated_at: string;
  deadline_at: string;
  user_email: string;
  last_attempt_at: string | null;
  last_error_code: string | null;
  last_error_message_safe: string | null;
};

type StaleTasksResponse = {
  tasks: StaleTask[];
};

function formatDate(value: string | null) {
  if (!value) return "-";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return "-";
  return date.toLocaleString();
}

export function StaleTasksTable() {
  const [rows, setRows] = useState<StaleTask[]>([]);
  const [loading, setLoading] = useState(true);
  const [actionLoading, setActionLoading] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const res = await fetch(`${clientApiBaseUrl}/api/admin/ops/train/stale-tasks?limit=20`, {
        credentials: "include",
      });
      if (!res.ok) {
        setError("Failed to load stale tasks");
        return;
      }
      const payload = (await res.json()) as StaleTasksResponse;
      setRows(payload.tasks ?? []);
    } catch {
      setError("Connection error");
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  const requeue = async (taskId: string) => {
    const confirmed = window.confirm(`Requeue task ${taskId}?`);
    if (!confirmed) return;
    setActionLoading(taskId);
    setError(null);
    try {
      const res = await fetch(`${clientApiBaseUrl}/api/admin/ops/train/tasks/${taskId}/requeue`, {
        method: "POST",
        credentials: "include",
      });
      const payload = (await res.json().catch(() => null)) as { detail?: unknown; message?: string } | null;
      if (!res.ok) {
        const detail = typeof payload?.detail === "string" ? payload.detail : "Requeue failed";
        setError(detail);
        return;
      }
      await load();
    } catch {
      setError("Requeue failed");
    } finally {
      setActionLoading(null);
    }
  };

  return (
    <section className={UI_CARD_MD}>
      <div className="flex items-start justify-between gap-3">
        <div>
          <p className={UI_KICKER}>Ops</p>
          <h2 className={`mt-2 ${UI_TITLE_MD}`}>Stale Train Tasks</h2>
          <p className="mt-2 text-sm text-slate-500">
            Tasks stuck in a processing state past the stale window. Requeue to trigger recovery.
          </p>
        </div>
        <button type="button" className={UI_BUTTON_OUTLINE_SM} onClick={() => void load()} disabled={loading || !!actionLoading}>
          Refresh
        </button>
      </div>

      {error ? <p className="mt-4 rounded-xl bg-rose-50 px-3 py-2 text-sm text-rose-700">{error}</p> : null}

      <div className="mt-4 overflow-hidden rounded-xl border border-blossom-100">
        <table className="w-full text-sm">
          <thead className="bg-blossom-50/50 text-left text-xs uppercase tracking-wide text-slate-500">
            <tr>
              <th className="px-4 py-3">Task</th>
              <th className="px-4 py-3">State</th>
              <th className="px-4 py-3 hidden md:table-cell">Updated</th>
              <th className="px-4 py-3 hidden lg:table-cell">User</th>
              <th className="px-4 py-3">Last Error</th>
              <th className="px-4 py-3">Action</th>
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
                  No stale tasks found
                </td>
              </tr>
            ) : (
              rows.map((row) => (
                <tr key={row.task_id} className="hover:bg-blossom-50/30">
                  <td className="px-4 py-3 font-mono text-xs text-slate-700">{row.task_id}</td>
                  <td className="px-4 py-3 text-slate-700">{row.state}</td>
                  <td className="px-4 py-3 hidden md:table-cell text-xs text-slate-500">{formatDate(row.updated_at)}</td>
                  <td className="px-4 py-3 hidden lg:table-cell text-xs text-slate-500">{row.user_email}</td>
                  <td className="px-4 py-3 text-xs text-slate-600">
                    {row.last_error_message_safe ? (
                      <span title={row.last_error_code ?? undefined}>{row.last_error_message_safe}</span>
                    ) : (
                      "-"
                    )}
                  </td>
                  <td className="px-4 py-3">
                    <button
                      type="button"
                      className={UI_BUTTON_OUTLINE_SM}
                      onClick={() => void requeue(row.task_id)}
                      disabled={!!actionLoading}
                    >
                      {actionLoading === row.task_id ? "Requeuing..." : "Requeue"}
                    </button>
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

