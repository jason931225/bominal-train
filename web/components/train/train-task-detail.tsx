"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { useEffect, useMemo, useState } from "react";

import { useLocale } from "@/components/locale-provider";
import { clientApiBaseUrl } from "@/lib/api-base";
import { ROUTES } from "@/lib/routes";
import { UI_BUTTON_DANGER_SM, UI_BUTTON_OUTLINE_SM } from "@/lib/ui";
import type { TrainArtifact, TrainTaskAttempt, TrainTaskSummary } from "@/lib/types";

const POLL_MS = 4000;
const SMALL_BUTTON_CLASS = UI_BUTTON_OUTLINE_SM;
const SMALL_DANGER_BUTTON_CLASS = UI_BUTTON_DANGER_SM;
const SMALL_SUCCESS_BUTTON_CLASS =
  "inline-flex h-8 items-center justify-center rounded-full border border-emerald-200 bg-emerald-50 px-2.5 text-xs font-medium text-emerald-700 shadow-sm transition hover:bg-emerald-100 focus:outline-none focus:ring-2 focus:ring-emerald-100 disabled:cursor-not-allowed disabled:opacity-60";
const SMALL_DISABLED_BUTTON_CLASS =
  "inline-flex h-8 items-center justify-center rounded-full border border-slate-200 bg-slate-100 px-2.5 text-xs font-medium text-slate-500 shadow-sm transition focus:outline-none focus:ring-2 focus:ring-slate-100";

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

function formatDateTimeKstSeconds(value: string | Date | null, locale: string): string {
  if (!value) return "-";
  const date = value instanceof Date ? value : new Date(value);
  if (Number.isNaN(date.getTime())) return "-";

  const formatted = new Intl.DateTimeFormat(locale === "ko" ? "ko-KR" : "en-US", {
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

function retryNowDisabledTitle(task: TrainTaskSummary): string {
  const reason = task.retry_now_reason ?? null;
  if (!reason) return "Retry is not available.";
  if (reason === "cooldown_active" && task.retry_now_available_at) {
    return `Retry available at ${formatDateTimeKstSeconds(task.retry_now_available_at)}.`;
  }
  if (reason === "deadline_passed") return "Task deadline has passed.";
  if (reason === "paused_use_resume") return "Task is paused. Use Resume instead.";
  if (reason === "task_running") return "Task is currently running.";
  if (reason === "terminal_state") return "Task is already finished.";
  if (reason === "not_eligible_state") return "Task is not eligible for retry.";
  return "Retry is not available.";
}

export function TrainTaskDetail({ taskId }: { taskId: string }) {
  const router = useRouter();
  const { locale, t } = useLocale();
  const [task, setTask] = useState<TrainTaskSummary | null>(null);
  const [attempts, setAttempts] = useState<TrainTaskAttempt[]>([]);
  const [artifacts, setArtifacts] = useState<TrainArtifact[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [cancellingTicket, setCancellingTicket] = useState(false);
  const [payingTicket, setPayingTicket] = useState(false);
  const [deletingTask, setDeletingTask] = useState(false);
  const [retryingNow, setRetryingNow] = useState(false);

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
        setError(payload?.detail ?? t("train.error.taskDetailLoad"));
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
      setError(t("train.error.taskDetailLoad"));
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
      setError(t("train.ticket.missingArtifact"));
      return;
    }
    const confirmed = window.confirm(t("train.confirm.cancelTicket"));
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
        setError(payload?.detail ?? t("train.error.ticketCancel"));
        return;
      }

      setNotice(payload?.detail ?? t("train.notice.ticketCancelDone"));
      await loadDetail();
    } catch {
      setError(t("train.error.ticketCancel"));
    } finally {
      setCancellingTicket(false);
    }
  };

  const deleteTask = async () => {
    const confirmed = window.confirm(t("train.confirm.hideTask"));
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
        setError(payload?.detail ?? t("train.task.actionFailed"));
        return;
      }

      router.push(ROUTES.modules.train);
    } catch {
      setError(t("train.task.actionFailed"));
    } finally {
      setDeletingTask(false);
    }
  };

  const payTicket = async () => {
    const confirmed = window.confirm(t("train.confirm.payNow"));
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
        setError(payload?.detail ?? t("train.error.paymentProcess"));
        return;
      }

      setNotice(t("train.notice.paymentProcessed"));
      await loadDetail();
    } catch {
      setError(t("train.error.paymentProcess"));
    } finally {
      setPayingTicket(false);
    }
  };

  const retryNow = async () => {
    if (!task) return;

    setRetryingNow(true);
    setError(null);
    setNotice(null);
    try {
      const response = await fetch(`${clientApiBaseUrl}/api/train/tasks/${taskId}/retry`, {
        method: "POST",
        credentials: "include",
      });
      const payload = (await response.json().catch(() => null)) as { detail?: string } | null;
      if (!response.ok) {
        setError(payload?.detail ?? "Could not retry task.");
        return;
      }

      setNotice("Retry requested.");
      await loadDetail();
    } catch {
      setError("Could not retry task.");
    } finally {
      setRetryingNow(false);
    }
  };

  return (
    <section className="space-y-6">
      <div className="rounded-3xl border border-blossom-100 bg-white p-6 shadow-petal">
        <div className="flex items-start justify-between gap-3">
          <div>
            <p className="text-xs uppercase tracking-[0.16em] text-blossom-500">{t("train.taskDetailKicker")}</p>
            <h1 className="mt-2 text-3xl font-semibold tracking-tight text-slate-800">{t("train.trainTask")}</h1>
            <p className="mt-1 text-sm text-slate-500">
              {t("train.label.taskId")} {taskId}
            </p>
          </div>
          <Link href={ROUTES.modules.train} className="text-sm font-medium text-blossom-600 hover:text-blossom-700">
            {t("train.backToTrain")}
          </Link>
        </div>
      </div>

      {error ? <p className="rounded-xl bg-rose-50 px-3 py-2 text-sm text-rose-700">{error}</p> : null}
      {notice ? <p className="rounded-xl bg-emerald-50 px-3 py-2 text-sm text-emerald-700">{notice}</p> : null}

      {task ? (
        <div className="rounded-2xl border border-blossom-100 bg-white p-6 shadow-petal">
          <h2 className="text-lg font-semibold text-slate-800">{t("train.taskStatus")}</h2>
          <div className="mt-3 grid gap-2 text-sm text-slate-700 md:grid-cols-2">
            <p>
              <span className="font-medium">{t("train.state")}</span> {task.state}
            </p>
            <p>
              <span className="font-medium">{t("train.deadline")}</span> {formatDateTimeKstSeconds(task.deadline_at, locale)}
            </p>
            <p>
              <span className="font-medium">{t("train.created")}</span> {formatDateTimeKstSeconds(task.created_at, locale)}
            </p>
            <p>
              <span className="font-medium">{t("train.lastAttempt")}</span>{" "}
              {formatDateTimeKstSeconds(task.last_attempt_at, locale)}
            </p>
            {task.state === "POLLING" ? (
              <p>
                <span className="font-medium">Next check:</span>{" "}
                {task.next_run_at ? formatDateTimeKstSeconds(task.next_run_at) : "-"}
              </p>
            ) : null}
          </div>
          {!isTerminal && (task.state === "QUEUED" || task.state === "POLLING") ? (
            <div className="mt-4 flex flex-wrap items-center gap-2">
              <button
                type="button"
                onClick={() => void retryNow()}
                disabled={
                  retryingNow ||
                  cancellingTicket ||
                  payingTicket ||
                  deletingTask ||
                  !task.retry_now_allowed
                }
                title={task.retry_now_allowed ? "Retry now" : retryNowDisabledTitle(task)}
                className={
                  task.retry_now_allowed && !retryingNow ? SMALL_BUTTON_CLASS : SMALL_DISABLED_BUTTON_CLASS
                }
              >
                {retryingNow ? "Retrying..." : "Retry now"}
              </button>
            </div>
          ) : null}
          {canPayReservation || canCancelReservation || isTerminal ? (
            <div className="mt-4 flex flex-wrap items-center gap-2">
              {canPayReservation ? (
                <button
                  type="button"
                  onClick={() => void payTicket()}
                  disabled={payingTicket || cancellingTicket || deletingTask}
                  className={SMALL_SUCCESS_BUTTON_CLASS}
                >
                  {payingTicket ? t("train.action.paying") : t("train.action.pay")}
                </button>
              ) : null}
              {canCancelReservation ? (
                <button
                  type="button"
                  onClick={() => void cancelTicket()}
                  disabled={!ticketArtifact || cancellingTicket || deletingTask || payingTicket}
                  className={SMALL_DANGER_BUTTON_CLASS}
                >
                  {cancellingTicket ? t("train.action.cancelling") : t("train.action.cancel")}
                </button>
              ) : null}
              {isTerminal ? (
                <button
                  type="button"
                  onClick={() => void deleteTask()}
                  disabled={deletingTask || cancellingTicket || payingTicket}
                  className={SMALL_DANGER_BUTTON_CLASS}
                >
                  {deletingTask ? t("common.deleting") : t("common.delete")}
                </button>
              ) : null}
            </div>
          ) : null}
          {!isTerminal ? <p className="mt-3 text-xs text-slate-500">{t("train.pollingHint")}</p> : null}
        </div>
      ) : null}

      <div className="rounded-2xl border border-blossom-100 bg-white p-6 shadow-petal">
        <h2 className="text-lg font-semibold text-slate-800">{t("train.attemptsTimeline")}</h2>
        <div className="mt-4 overflow-x-auto">
          <table className="min-w-full text-left text-sm">
            <thead>
              <tr className="text-slate-500">
                <th className="pb-2 pr-3">{t("train.attemptTable.action")}</th>
                <th className="pb-2 pr-3">{t("train.attemptTable.provider")}</th>
                <th className="pb-2 pr-3">{t("train.attemptTable.ok")}</th>
                <th className="pb-2 pr-3">{t("train.attemptTable.retryable")}</th>
                <th className="pb-2 pr-3">{t("train.attemptTable.duration")}</th>
                <th className="pb-2 pr-3">{t("train.attemptTable.started")}</th>
                <th className="pb-2">{t("train.attemptTable.error")}</th>
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
                  <td className="py-2 pr-3">{formatDateTimeKstSeconds(attempt.started_at, locale)}</td>
                  <td className="py-2">{attempt.error_message_safe || "-"}</td>
                </tr>
              ))}
              {attempts.length === 0 ? (
                <tr>
                  <td className="py-2 text-slate-500" colSpan={7}>
                    {t("train.empty.attempts")}
                  </td>
                </tr>
              ) : null}
            </tbody>
          </table>
        </div>
      </div>

      <div className="rounded-2xl border border-blossom-100 bg-white p-6 shadow-petal">
        <h2 className="text-lg font-semibold text-slate-800">{t("train.ticketArtifacts")}</h2>
        <ul className="mt-4 space-y-3 text-sm">
          {artifacts.length === 0 ? <li className="text-slate-500">{t("train.empty.artifacts")}</li> : null}
          {artifacts.map((artifact) => (
            <li key={artifact.id} className="rounded-xl border border-blossom-100 p-3">
              <p className="font-medium text-slate-700">{artifact.kind}</p>
              <p className="text-xs text-slate-500">
                {t("common.created")}: {formatDateTimeKstSeconds(artifact.created_at, locale)}
              </p>
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
                        <span className="font-medium">{t("train.artifact.provider")}</span> {provider}
                      </p>
                    ) : null}
                    {reservationId ? (
                      <p>
                        <span className="font-medium">{t("train.artifact.reservation")}</span> {reservationId}
                      </p>
                    ) : null}
                    {status ? (
                      <p>
                        <span className="font-medium">{t("train.artifact.status")}</span> {status}
                      </p>
                    ) : null}
                    {paid !== null ? (
                      <p>
                        <span className="font-medium">{t("train.artifact.paid")}</span> {paid ? t("train.artifact.yes") : t("train.artifact.no")}
                      </p>
                    ) : null}
                    {paymentDeadline ? (
                      <p>
                        <span className="font-medium">{t("train.payBy")}</span> {formatDateTimeKstSeconds(paymentDeadline, locale)}
                      </p>
                    ) : null}
                    {seatCount != null ? (
                      <p>
                        <span className="font-medium">{t("train.artifact.seatCount")}</span> {seatCount}
                      </p>
                    ) : null}
                    {seats.length > 0 ? (
                      <p>
                        <span className="font-medium">{t("train.artifact.seats")}</span> {seats.join(", ")}
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
