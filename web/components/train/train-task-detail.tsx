"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { useEffect, useMemo, useState } from "react";

import { clientApiBaseUrl } from "@/lib/api-base";
import { UI_BUTTON_DANGER_SM } from "@/lib/ui";
import type { TrainArtifact, TrainTaskAttempt, TrainTaskSummary } from "@/lib/types";

const POLL_MS = 4000;
const SMALL_DANGER_BUTTON_CLASS = UI_BUTTON_DANGER_SM;
const SMALL_SUCCESS_BUTTON_CLASS =
  "inline-flex h-8 items-center justify-center rounded-full border border-emerald-200 bg-emerald-50 px-2.5 text-xs font-medium text-emerald-700 shadow-sm transition hover:bg-emerald-100 focus:outline-none focus:ring-2 focus:ring-emerald-100 disabled:cursor-not-allowed disabled:opacity-60";

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function readString(value: unknown): string | null {
  return typeof value === "string" && value.length > 0 ? value : null;
}

function readBoolean(value: unknown): boolean | null {
  return typeof value === "boolean" ? value : null;
}

function readTicketSeats(value: unknown): string[] {
  if (!Array.isArray(value)) return [];
  const seats: string[] = [];
  for (const item of value) {
    if (!isRecord(item)) continue;
    const car = readString(item.car_no);
    const seat = readString(item.seat_no);
    if (car && seat) {
      seats.push(`${car}-${seat}`);
    } else if (seat) {
      seats.push(seat);
    }
  }
  return seats;
}

function formatDateTimeKstSeconds(value: string | Date | null): string {
  if (!value) return "-";
  const date = value instanceof Date ? value : new Date(value);
  if (Number.isNaN(date.getTime())) return "-";

  const formatted = new Intl.DateTimeFormat("ko-KR", {
    timeZone: "Asia/Seoul",
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    hour12: false,
  }).format(date);
  return `${formatted} KST`;
}

export function TrainTaskDetail({ taskId }: { taskId: string }) {
  const router = useRouter();
  const [task, setTask] = useState<TrainTaskSummary | null>(null);
  const [attempts, setAttempts] = useState<TrainTaskAttempt[]>([]);
  const [artifacts, setArtifacts] = useState<TrainArtifact[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [cancellingTicket, setCancellingTicket] = useState(false);
  const [payingTicket, setPayingTicket] = useState(false);
  const [deletingTask, setDeletingTask] = useState(false);

  const isTerminal = useMemo(() => {
    if (!task) return false;
    return ["COMPLETED", "EXPIRED", "CANCELLED", "FAILED"].includes(task.state);
  }, [task]);
  const isCompleted = task?.state === "COMPLETED";
  const ticketArtifact = useMemo(() => artifacts.find((artifact) => artifact.kind === "ticket") ?? null, [artifacts]);
  const canCancelReservation = useMemo(() => {
    if (!ticketArtifact) return false;
    const status = readString(ticketArtifact.data_json_safe.status);
    const paid = readBoolean(ticketArtifact.data_json_safe.paid);
    const cancelled = readBoolean(ticketArtifact.data_json_safe.cancelled) ?? false;
    if (cancelled) return false;
    if (status === "cancelled" || status === "reservation_not_found") return false;
    if (status === "awaiting_payment" && paid !== true) return true;
    return isCompleted;
  }, [ticketArtifact, isCompleted]);
  const canPayReservation = useMemo(() => {
    if (!ticketArtifact) return false;
    const status = readString(ticketArtifact.data_json_safe.status);
    const paid = readBoolean(ticketArtifact.data_json_safe.paid);
    const cancelled = readBoolean(ticketArtifact.data_json_safe.cancelled) ?? false;
    return status === "awaiting_payment" && paid !== true && !cancelled;
  }, [ticketArtifact]);

  const loadDetail = async () => {
    try {
      const response = await fetch(`${clientApiBaseUrl}/api/train/tasks/${taskId}`, {
        credentials: "include",
        cache: "no-store",
      });
      if (!response.ok) {
        const payload = (await response.json().catch(() => null)) as { detail?: string } | null;
        setError(payload?.detail ?? "Could not load task detail.");
        return;
      }

      const payload = (await response.json()) as {
        task: TrainTaskSummary;
        attempts: TrainTaskAttempt[];
        artifacts: TrainArtifact[];
      };
      setTask(payload.task);
      setAttempts(payload.attempts);
      setArtifacts(payload.artifacts);
      setError(null);
    } catch {
      setError("Could not load task detail.");
    }
  };

  useEffect(() => {
    void loadDetail();
    const interval = window.setInterval(() => {
      void loadDetail();
    }, POLL_MS);
    return () => window.clearInterval(interval);
  }, [taskId]);

  const cancelTicket = async () => {
    if (!ticketArtifact) {
      setError("No ticket artifact found for this task.");
      return;
    }
    const confirmed = window.confirm("Cancel this reservation ticket?");
    if (!confirmed) return;

    setCancellingTicket(true);
    setError(null);
    setNotice(null);
    try {
      const response = await fetch(`${clientApiBaseUrl}/api/train/tickets/${ticketArtifact.id}/cancel`, {
        method: "POST",
        credentials: "include",
      });
      const payload = (await response.json().catch(() => null)) as { status?: string; detail?: string } | null;
      if (!response.ok) {
        setError(payload?.detail ?? "Could not cancel ticket.");
        return;
      }

      setNotice(payload?.detail ?? "Ticket cancel request completed.");
      await loadDetail();
    } catch {
      setError("Could not cancel ticket.");
    } finally {
      setCancellingTicket(false);
    }
  };

  const deleteTask = async () => {
    const confirmed = window.confirm("Hide this task from your list?");
    if (!confirmed) return;

    setDeletingTask(true);
    setError(null);
    setNotice(null);
    try {
      const response = await fetch(`${clientApiBaseUrl}/api/train/tasks/${taskId}/delete`, {
        method: "POST",
        credentials: "include",
      });
      const payload = (await response.json().catch(() => null)) as { detail?: string } | null;
      if (!response.ok) {
        setError(payload?.detail ?? "Could not delete task.");
        return;
      }

      router.push("/modules/train");
    } catch {
      setError("Could not delete task.");
    } finally {
      setDeletingTask(false);
    }
  };

  const payTicket = async () => {
    const confirmed = window.confirm("Process payment for this awaiting reservation now?");
    if (!confirmed) return;

    setPayingTicket(true);
    setError(null);
    setNotice(null);
    try {
      const response = await fetch(`${clientApiBaseUrl}/api/train/tasks/${taskId}/pay`, {
        method: "POST",
        credentials: "include",
      });
      const payload = (await response.json().catch(() => null)) as { detail?: string } | null;
      if (!response.ok) {
        setError(payload?.detail ?? "Could not process payment.");
        return;
      }

      setNotice("Payment processed.");
      await loadDetail();
    } catch {
      setError("Could not process payment.");
    } finally {
      setPayingTicket(false);
    }
  };

  return (
    <section className="space-y-6">
      <div className="rounded-3xl border border-blossom-100 bg-white p-6 shadow-petal">
        <div className="flex items-start justify-between gap-3">
          <div>
            <p className="text-xs uppercase tracking-[0.16em] text-blossom-500">Task detail</p>
            <h1 className="mt-2 text-3xl font-semibold tracking-tight text-slate-800">Train Task</h1>
            <p className="mt-1 text-sm text-slate-500">Task ID: {taskId}</p>
          </div>
          <Link href="/modules/train" className="text-sm font-medium text-blossom-600 hover:text-blossom-700">
            Back to Train
          </Link>
        </div>
      </div>

      {error ? <p className="rounded-xl bg-rose-50 px-3 py-2 text-sm text-rose-700">{error}</p> : null}
      {notice ? <p className="rounded-xl bg-emerald-50 px-3 py-2 text-sm text-emerald-700">{notice}</p> : null}

      {task ? (
        <div className="rounded-2xl border border-blossom-100 bg-white p-6 shadow-petal">
          <h2 className="text-lg font-semibold text-slate-800">Task status</h2>
          <div className="mt-3 grid gap-2 text-sm text-slate-700 md:grid-cols-2">
            <p>
              <span className="font-medium">State:</span> {task.state}
            </p>
            <p>
              <span className="font-medium">Deadline:</span> {formatDateTimeKstSeconds(task.deadline_at)}
            </p>
            <p>
              <span className="font-medium">Created:</span> {formatDateTimeKstSeconds(task.created_at)}
            </p>
            <p>
              <span className="font-medium">Last attempt:</span> {formatDateTimeKstSeconds(task.last_attempt_at)}
            </p>
          </div>
          {canPayReservation || canCancelReservation || isTerminal ? (
            <div className="mt-4 flex flex-wrap items-center gap-2">
              {canPayReservation ? (
                <button
                  type="button"
                  onClick={() => void payTicket()}
                  disabled={payingTicket || cancellingTicket || deletingTask}
                  className={SMALL_SUCCESS_BUTTON_CLASS}
                >
                  {payingTicket ? "Paying..." : "Pay"}
                </button>
              ) : null}
              {canCancelReservation ? (
                <button
                  type="button"
                  onClick={() => void cancelTicket()}
                  disabled={!ticketArtifact || cancellingTicket || deletingTask || payingTicket}
                  className={SMALL_DANGER_BUTTON_CLASS}
                >
                  {cancellingTicket ? "Cancelling..." : "Cancel"}
                </button>
              ) : null}
              {isTerminal ? (
                <button
                  type="button"
                  onClick={() => void deleteTask()}
                  disabled={deletingTask || cancellingTicket || payingTicket}
                  className={SMALL_DANGER_BUTTON_CLASS}
                >
                  {deletingTask ? "Deleting..." : "Delete"}
                </button>
              ) : null}
            </div>
          ) : null}
          {!isTerminal ? <p className="mt-3 text-xs text-slate-500">Polling every 4s while active.</p> : null}
        </div>
      ) : null}

      <div className="rounded-2xl border border-blossom-100 bg-white p-6 shadow-petal">
        <h2 className="text-lg font-semibold text-slate-800">Attempts timeline</h2>
        <div className="mt-4 overflow-x-auto">
          <table className="min-w-full text-left text-sm">
            <thead>
              <tr className="text-slate-500">
                <th className="pb-2 pr-3">Action</th>
                <th className="pb-2 pr-3">Provider</th>
                <th className="pb-2 pr-3">OK</th>
                <th className="pb-2 pr-3">Retryable</th>
                <th className="pb-2 pr-3">Duration</th>
                <th className="pb-2 pr-3">Started</th>
                <th className="pb-2">Error</th>
              </tr>
            </thead>
            <tbody>
              {attempts.map((attempt) => (
                <tr key={attempt.id} className="border-t border-blossom-100">
                  <td className="py-2 pr-3">{attempt.action}</td>
                  <td className="py-2 pr-3">{attempt.provider}</td>
                  <td className="py-2 pr-3">{attempt.ok ? "yes" : "no"}</td>
                  <td className="py-2 pr-3">{attempt.retryable ? "yes" : "no"}</td>
                  <td className="py-2 pr-3">{attempt.duration_ms}ms</td>
                  <td className="py-2 pr-3">{formatDateTimeKstSeconds(attempt.started_at)}</td>
                  <td className="py-2">{attempt.error_message_safe || "-"}</td>
                </tr>
              ))}
              {attempts.length === 0 ? (
                <tr>
                  <td className="py-2 text-slate-500" colSpan={7}>
                    No attempts yet.
                  </td>
                </tr>
              ) : null}
            </tbody>
          </table>
        </div>
      </div>

      <div className="rounded-2xl border border-blossom-100 bg-white p-6 shadow-petal">
        <h2 className="text-lg font-semibold text-slate-800">Ticket / receipt artifacts</h2>
        <ul className="mt-4 space-y-3 text-sm">
          {artifacts.length === 0 ? <li className="text-slate-500">No artifacts yet.</li> : null}
          {artifacts.map((artifact) => (
            <li key={artifact.id} className="rounded-xl border border-blossom-100 p-3">
              <p className="font-medium text-slate-700">{artifact.kind}</p>
              <p className="text-xs text-slate-500">Created: {formatDateTimeKstSeconds(artifact.created_at)}</p>
              {(() => {
                const data = artifact.data_json_safe;
                const provider = readString(data.provider);
                const reservationId = readString(data.reservation_id);
                const status = readString(data.status);
                const paid = readBoolean(data.paid);
                const paymentDeadline = readString(data.payment_deadline_at);
                const seatCount = typeof data.seat_count === "number" ? data.seat_count : null;
                const seats = readTicketSeats(data.tickets);

                if (!provider && !reservationId && !status && paid === null && !paymentDeadline && seatCount == null && seats.length === 0) {
                  return null;
                }

                return (
                  <div className="mt-3 grid gap-1.5 rounded-lg border border-blossom-100 bg-blossom-50/40 px-3 py-2 text-xs text-slate-700">
                    {provider ? (
                      <p>
                        <span className="font-medium">Provider:</span> {provider}
                      </p>
                    ) : null}
                    {reservationId ? (
                      <p>
                        <span className="font-medium">Reservation:</span> {reservationId}
                      </p>
                    ) : null}
                    {status ? (
                      <p>
                        <span className="font-medium">Status:</span> {status}
                      </p>
                    ) : null}
                    {paid !== null ? (
                      <p>
                        <span className="font-medium">Paid:</span> {paid ? "yes" : "no"}
                      </p>
                    ) : null}
                    {paymentDeadline ? (
                      <p>
                        <span className="font-medium">Pay by:</span> {formatDateTimeKstSeconds(paymentDeadline)}
                      </p>
                    ) : null}
                    {seatCount != null ? (
                      <p>
                        <span className="font-medium">Seat count:</span> {seatCount}
                      </p>
                    ) : null}
                    {seats.length > 0 ? (
                      <p>
                        <span className="font-medium">Seats:</span> {seats.join(", ")}
                      </p>
                    ) : null}
                  </div>
                );
              })()}
              <pre className="mt-2 overflow-auto rounded-lg bg-slate-50 p-2 text-xs text-slate-700">
                {JSON.stringify(artifact.data_json_safe, null, 2)}
              </pre>
            </li>
          ))}
        </ul>
      </div>
    </section>
  );
}
