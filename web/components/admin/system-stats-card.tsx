"use client";

import { useState, useEffect } from "react";
import { clientApiBaseUrl } from "@/lib/api-base";
import { UI_CARD_MD, UI_KICKER, UI_TITLE_MD } from "@/lib/ui";

type SystemStats = {
  total_users: number;
  active_users_24h: number;
  total_sessions: number;
  active_sessions: number;
  total_tasks: number;
  tasks_by_state: Record<string, number>;
  tasks_completed_24h: number;
};

export function SystemStatsCard() {
  const [stats, setStats] = useState<SystemStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    async function fetchStats() {
      try {
        const res = await fetch(`${clientApiBaseUrl}/api/admin/stats`, { credentials: "include" });
        if (res.ok) {
          const data = await res.json();
          setStats(data);
        } else {
          setError("Failed to load stats");
        }
      } catch (e) {
        setError("Connection error");
      } finally {
        setLoading(false);
      }
    }
    fetchStats();
  }, []);

  if (loading) {
    return (
      <div className={UI_CARD_MD}>
        <p className={UI_KICKER}>Overview</p>
        <h2 className={`mt-2 ${UI_TITLE_MD}`}>System Statistics</h2>
        <div className="mt-4 text-center text-slate-400 py-8">Loading...</div>
      </div>
    );
  }

  if (error || !stats) {
    return (
      <div className={UI_CARD_MD}>
        <p className={UI_KICKER}>Overview</p>
        <h2 className={`mt-2 ${UI_TITLE_MD}`}>System Statistics</h2>
        <div className="mt-4 text-center text-rose-500 py-8">{error || "No data"}</div>
      </div>
    );
  }

  const statItems = [
    {
      label: "Total Users",
      value: stats.total_users,
      sub: `${stats.active_users_24h} active (24h)`,
      color: "bg-indigo-50 text-indigo-600",
    },
    {
      label: "Sessions",
      value: stats.total_sessions,
      sub: `${stats.active_sessions} active now`,
      color: "bg-emerald-50 text-emerald-600",
    },
    {
      label: "Total Tasks",
      value: stats.total_tasks,
      sub: `${stats.tasks_completed_24h} completed (24h)`,
      color: "bg-amber-50 text-amber-600",
    },
  ];

  const taskStates = [
    { key: "QUEUED", label: "Queued", color: "bg-slate-100 text-slate-700" },
    { key: "RUNNING", label: "Running", color: "bg-blue-100 text-blue-700" },
    { key: "POLLING", label: "Polling", color: "bg-indigo-100 text-indigo-700" },
    { key: "RESERVING", label: "Reserving", color: "bg-amber-100 text-amber-800" },
    { key: "PAYING", label: "Paying", color: "bg-fuchsia-100 text-fuchsia-800" },
    { key: "PAUSED", label: "Paused", color: "bg-slate-200 text-slate-700" },
    { key: "COMPLETED", label: "Completed", color: "bg-emerald-100 text-emerald-800" },
    { key: "FAILED", label: "Failed", color: "bg-rose-100 text-rose-700" },
    { key: "CANCELLED", label: "Cancelled", color: "bg-orange-100 text-orange-800" },
    { key: "EXPIRED", label: "Expired", color: "bg-slate-100 text-slate-600" },
  ] as const;

  return (
    <section className={UI_CARD_MD}>
      <p className={UI_KICKER}>Overview</p>
      <h2 className={`mt-2 ${UI_TITLE_MD}`}>System Statistics</h2>

      {/* Main stats */}
      <div className="mt-6 grid grid-cols-1 gap-4 sm:grid-cols-3">
        {statItems.map((item) => (
          <div
            key={item.label}
            className={`rounded-xl p-4 ${item.color.split(" ")[0]}`}
          >
            <p className="text-xs font-medium uppercase tracking-wide opacity-70">
              {item.label}
            </p>
            <p className={`mt-1 text-3xl font-bold ${item.color.split(" ")[1]}`}>
              {item.value.toLocaleString()}
            </p>
            <p className="mt-1 text-xs opacity-60">{item.sub}</p>
          </div>
        ))}
      </div>

      {/* Task breakdown */}
      <div className="mt-6">
        <p className="text-xs font-medium uppercase tracking-wide text-slate-400">
          Tasks by State
        </p>
        <div className="mt-2 flex flex-wrap gap-2">
          {taskStates.map((ts) => {
            const count = stats.tasks_by_state[ts.key] ?? 0;
            return (
              <div
                key={ts.key}
                className={`inline-flex items-center gap-1.5 rounded-lg px-3 py-1.5 text-sm ${ts.color}`}
              >
                <span className="font-medium">{ts.label}</span>
                <span className="font-bold">{count}</span>
              </div>
            );
          })}
        </div>
      </div>
    </section>
  );
}
