"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { useLocale } from "@/components/locale-provider";
import { clientApiBaseUrl } from "@/lib/api-base";
import { ROUTES } from "@/lib/routes";
import { fetchTrainTaskDetailViaGraphql } from "@/lib/train/graphql";
import { subscribeTrainTaskEvents } from "@/lib/train/task-events";
import { UI_BUTTON_DANGER_SM, UI_BUTTON_OUTLINE_SM } from "@/lib/ui";
import type { TrainArtifact, TrainTaskAttempt, TrainTaskSummary } from "@/lib/types";

const ATTEMPTS_PAGE_SIZE_OPTIONS = [10, 20, 50, 100, "all"] as const;
type AttemptsPageSize = (typeof ATTEMPTS_PAGE_SIZE_OPTIONS)[number];
const ATTEMPT_SORT_OPTIONS = ["newest", "oldest"] as const;
type AttemptSortOrder = (typeof ATTEMPT_SORT_OPTIONS)[number];
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

function retryNowDisabledTitle(task: TrainTaskSummary, locale: string): string {
  const reason = task.retry_now_reason ?? null;
  if (!reason) return "Retry is not available.";
  if (reason === "cooldown_active" && task.retry_now_available_at) {
    return `Retry available at ${formatDateTimeKstSeconds(task.retry_now_available_at, locale)}.`;
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
  const detailLoadInFlightRef = useRef(false);
  const queuedDetailReloadRef = useRef(false);
  const [task, setTask] = useState<TrainTaskSummary | null>(null);
  const [attempts, setAttempts] = useState<TrainTaskAttempt[]>([]);
  const [attemptsPageSize, setAttemptsPageSize] = useState<AttemptsPageSize>(10);
  const [attemptSortOrder, setAttemptSortOrder] = useState<AttemptSortOrder>("newest");
  const [attemptsPage, setAttemptsPage] = useState(1);
  const [artifacts, setArtifacts] = useState<TrainArtifact[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [refreshingTask, setRefreshingTask] = useState(false);
  const [cancellingTicket, setCancellingTicket] = useState(false);
  const [payingTicket, setPayingTicket] = useState(false);
  const [pausingTask, setPausingTask] = useState(false);
  const [resumingTask, setResumingTask] = useState(false);
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
    if (task?.state === "EXPIRED" || task?.state === "CANCELLED" || task?.state === "FAILED") return false;
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
    if (task?.state === "EXPIRED" || task?.state === "CANCELLED" || task?.state === "FAILED") return false;
    const status = readString(ticketArtifact.data_json_safe.status);
    const paid = readBoolean(ticketArtifact.data_json_safe.paid);
    const cancelled = readBoolean(ticketArtifact.data_json_safe.cancelled) ?? false;
    return status === "awaiting_payment" && paid !== true && !cancelled;
  }, [task?.state, ticketArtifact]);
  const canPauseTask = Boolean(task) && !isTerminal && task?.state !== "PAUSED";
  const canResumeTask = Boolean(task) && !isTerminal && task?.state === "PAUSED";
  const canCancelTask = Boolean(task) && (!isTerminal || canCancelReservation);
  const canRetryNow = task ? task.state === "QUEUED" || task.state === "POLLING" || isTerminal : false;
  const sortedAttempts = useMemo(() => {
    return [...attempts].sort((left, right) => {
      const leftStarted = Date.parse(left.started_at);
      const rightStarted = Date.parse(right.started_at);
      const leftFinished = Date.parse(left.finished_at);
      const rightFinished = Date.parse(right.finished_at);

      const leftTime = Number.isNaN(leftStarted) ? (Number.isNaN(leftFinished) ? 0 : leftFinished) : leftStarted;
      const rightTime = Number.isNaN(rightStarted) ? (Number.isNaN(rightFinished) ? 0 : rightFinished) : rightStarted;

      if (leftTime === rightTime) {
        const stableTieBreak = left.id.localeCompare(right.id);
        return attemptSortOrder === "newest" ? -stableTieBreak : stableTieBreak;
      }
      return attemptSortOrder === "newest" ? rightTime - leftTime : leftTime - rightTime;
    });
  }, [attemptSortOrder, attempts]);
  const totalAttemptPages = useMemo(() => {
    if (attemptsPageSize === "all") return 1;
    return Math.max(1, Math.ceil(sortedAttempts.length / attemptsPageSize));
  }, [attemptsPageSize, sortedAttempts.length]);
  const pagedAttempts = useMemo(() => {
    if (attemptsPageSize === "all") return sortedAttempts;
    const start = (attemptsPage - 1) * attemptsPageSize;
    return sortedAttempts.slice(start, start + attemptsPageSize);
  }, [attemptsPage, attemptsPageSize, sortedAttempts]);

  const loadDetail = useCallback(async () => {
    if (detailLoadInFlightRef.current) {
      queuedDetailReloadRef.current = true;
      return;
    }
    detailLoadInFlightRef.current = true;

    try {
      do {
        queuedDetailReloadRef.current = false;
        try {
          const graphqlPayload = await fetchTrainTaskDetailViaGraphql(taskId);
          if (graphqlPayload) {
            setTask(graphqlPayload.task);
            setAttempts(graphqlPayload.attempts);
            setArtifacts(graphqlPayload.artifacts);
            setError(null);
            continue;
          }

          const response = await fetch(`${clientApiBaseUrl}/api/train/tasks/${taskId}`, {
            credentials: "include",
            cache: "no-store",
          });
          if (!response.ok) {
            const payload = (await response.json().catch(() => null)) as { detail?: string } | null;
            setError(payload?.detail ?? t("train.error.taskDetailLoad"));
          } else {
            const payload = (await response.json()) as {
              task: TrainTaskSummary;
              attempts: TrainTaskAttempt[];
              artifacts: TrainArtifact[];
            };
            setTask(payload.task);
            setAttempts(payload.attempts);
            setArtifacts(payload.artifacts);
            setError(null);
          }
        } catch {
          setError(t("train.error.taskDetailLoad"));
        }
      } while (queuedDetailReloadRef.current);
    } finally {
      detailLoadInFlightRef.current = false;
      queuedDetailReloadRef.current = false;
    }
  }, [taskId, t]);

  const refreshTask = async () => {
    setRefreshingTask(true);
    setError(null);
    setNotice(null);
    try {
      const response = await fetch(`${clientApiBaseUrl}/api/train/tasks/${taskId}/refresh`, {
        method: "POST",
        credentials: "include",
      });
      if (!response.ok) {
        const payload = (await response.json().catch(() => null)) as { detail?: string } | null;
        setError(payload?.detail ?? t("train.error.taskRefresh"));
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
      setNotice(t("train.notice.taskRefreshed"));
    } catch {
      setError(t("train.error.taskRefresh"));
    } finally {
      setRefreshingTask(false);
    }
  };

  useEffect(() => {
    void loadDetail();
    const unsubscribeTaskEvents = subscribeTrainTaskEvents((payload, event) => {
      if (event.type !== "task_state") return;
      if (payload.task_id === taskId) {
        void loadDetail();
      }
    });
    return unsubscribeTaskEvents;
  }, [loadDetail, taskId]);

  useEffect(() => {
    setAttemptsPage((current) => Math.min(Math.max(1, current), totalAttemptPages));
  }, [totalAttemptPages]);

  const updateTaskState = async (action: "pause" | "resume") => {
    if (action === "pause") setPausingTask(true);
    if (action === "resume") setResumingTask(true);
    setError(null);
    setNotice(null);
    try {
      const response = await fetch(`${clientApiBaseUrl}/api/train/tasks/${taskId}/${action}`, {
        method: "POST",
        credentials: "include",
      });
      const payload = (await response.json().catch(() => null)) as { detail?: string } | null;
      if (!response.ok) {
        setError(payload?.detail ?? t("train.task.actionFailed"));
        return;
      }
      await loadDetail();
    } catch {
      setError(t("train.task.actionFailed"));
    } finally {
      setPausingTask(false);
      setResumingTask(false);
    }
  };

  const cancelTask = async () => {
    const confirmed = window.confirm(t("train.confirm.cancelTask"));
    if (!confirmed) return;

    setCancellingTicket(true);
    setError(null);
    setNotice(null);
    try {
      const response = await fetch(`${clientApiBaseUrl}/api/train/tasks/${taskId}/cancel`, {
        method: "POST",
        credentials: "include",
      });
      const payload = (await response.json().catch(() => null)) as { detail?: string } | null;
      if (!response.ok) {
        setError(payload?.detail ?? t("train.task.actionFailed"));
        return;
      }

      setNotice(payload?.detail ?? t("train.notice.ticketCancelDone"));
      await loadDetail();
    } catch {
      setError(t("train.task.actionFailed"));
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
          <div className="flex items-center justify-between gap-3">
            <h2 className="text-lg font-semibold text-slate-800">{t("train.taskStatus")}</h2>
            <button
              type="button"
              onClick={() => void refreshTask()}
              aria-label={t("train.action.syncRefresh")}
              title={t("train.action.syncRefresh")}
              disabled={refreshingTask || cancellingTicket || payingTicket || pausingTask || resumingTask || deletingTask}
              className={
                refreshingTask
                  ? `${SMALL_DISABLED_BUTTON_CLASS} w-8 px-0`
                  : `${SMALL_BUTTON_CLASS} w-8 px-0`
              }
            >
              <svg
                aria-hidden="true"
                className={`h-4 w-4 ${refreshingTask ? "animate-spin" : ""}`}
                viewBox="0 0 20 20"
                fill="currentColor"
              >
                <path
                  fillRule="evenodd"
                  d="M15.312 6.094a7 7 0 10.908 7.61 1 1 0 111.836.793A9 9 0 1120 10a1 1 0 11-2 0 7 7 0 00-2.688-5.51V7a1 1 0 11-2 0V3a1 1 0 011-1h4a1 1 0 010 2h-2.688z"
                  clipRule="evenodd"
                />
              </svg>
            </button>
          </div>
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
                {task.next_run_at ? formatDateTimeKstSeconds(task.next_run_at, locale) : "-"}
              </p>
            ) : null}
          </div>
          {canRetryNow ? (
            <div className="mt-4 flex flex-wrap items-center gap-2">
              <button
                type="button"
                onClick={() => void retryNow()}
                disabled={
                  retryingNow ||
                  cancellingTicket ||
                  payingTicket ||
                  pausingTask ||
                  resumingTask ||
                  refreshingTask ||
                  deletingTask ||
                  !task.retry_now_allowed
                }
                title={task.retry_now_allowed ? "Retry now" : retryNowDisabledTitle(task, locale)}
                className={
                  task.retry_now_allowed && !retryingNow ? SMALL_BUTTON_CLASS : SMALL_DISABLED_BUTTON_CLASS
                }
              >
                {retryingNow ? "Retrying..." : "Retry now"}
              </button>
            </div>
          ) : null}
          {canPayReservation || canPauseTask || canResumeTask || canCancelTask || isTerminal ? (
            <div className="mt-4 flex flex-wrap items-center gap-2">
              {canPayReservation ? (
                <button
                  type="button"
                  onClick={() => void payTicket()}
                  disabled={payingTicket || cancellingTicket || pausingTask || resumingTask || refreshingTask || deletingTask}
                  className={SMALL_SUCCESS_BUTTON_CLASS}
                >
                  {payingTicket ? t("train.action.paying") : t("train.action.pay")}
                </button>
              ) : null}
              {canPauseTask ? (
                <button
                  type="button"
                  onClick={() => void updateTaskState("pause")}
                  disabled={pausingTask || resumingTask || cancellingTicket || payingTicket || refreshingTask || deletingTask}
                  className={pausingTask ? SMALL_DISABLED_BUTTON_CLASS : SMALL_BUTTON_CLASS}
                >
                  {pausingTask ? t("common.loading") : t("train.action.pause")}
                </button>
              ) : null}
              {canResumeTask ? (
                <button
                  type="button"
                  onClick={() => void updateTaskState("resume")}
                  disabled={resumingTask || pausingTask || cancellingTicket || payingTicket || refreshingTask || deletingTask}
                  className={resumingTask ? SMALL_DISABLED_BUTTON_CLASS : SMALL_BUTTON_CLASS}
                >
                  {resumingTask ? t("common.loading") : t("train.action.resume")}
                </button>
              ) : null}
              {canCancelTask ? (
                <button
                  type="button"
                  onClick={() => void cancelTask()}
                  disabled={cancellingTicket || pausingTask || resumingTask || deletingTask || payingTicket || refreshingTask}
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

      <div className="rounded-2xl border border-blossom-100 bg-white p-6 shadow-petal">
        <h2 className="text-lg font-semibold text-slate-800">{t("train.attemptsTimeline")}</h2>
        <div className="mt-4 flex flex-wrap items-center justify-between gap-3 text-sm">
          <div className="flex flex-wrap items-center gap-3">
            <label className="inline-flex items-center gap-2 text-slate-700">
              <span>{t("train.attemptPagination.actionsPerPage")}</span>
              <select
                value={String(attemptsPageSize)}
                onChange={(event) => {
                  const next = event.target.value === "all" ? "all" : Number.parseInt(event.target.value, 10);
                  if (
                    next === "all" ||
                    next === 10 ||
                    next === 20 ||
                    next === 50 ||
                    next === 100
                  ) {
                    setAttemptsPageSize(next);
                    setAttemptsPage(1);
                  }
                }}
                className="rounded-lg border border-blossom-200 bg-white px-2 py-1 text-sm text-slate-700"
              >
                {ATTEMPTS_PAGE_SIZE_OPTIONS.map((option) => (
                  <option key={String(option)} value={String(option)}>
                    {option === "all" ? t("train.attemptPagination.all") : option}
                  </option>
                ))}
              </select>
            </label>
            <label className="inline-flex items-center gap-2 text-slate-700">
              <span>{t("train.attemptPagination.sort")}</span>
              <select
                value={attemptSortOrder}
                onChange={(event) => {
                  const next = event.target.value === "oldest" ? "oldest" : "newest";
                  setAttemptSortOrder(next);
                  setAttemptsPage(1);
                }}
                className="rounded-lg border border-blossom-200 bg-white px-2 py-1 text-sm text-slate-700"
              >
                {ATTEMPT_SORT_OPTIONS.map((option) => (
                  <option key={option} value={option}>
                    {option === "newest"
                      ? t("train.attemptPagination.newestFirst")
                      : t("train.attemptPagination.oldestFirst")}
                  </option>
                ))}
              </select>
            </label>
          </div>
          <div className="inline-flex items-center gap-2">
            <span className="text-xs text-slate-500">
              {t("train.attemptPagination.page")} {attemptsPage} / {totalAttemptPages}
            </span>
            <button
              type="button"
              onClick={() => setAttemptsPage((current) => Math.max(1, current - 1))}
              disabled={attemptsPage <= 1}
              className={attemptsPage <= 1 ? SMALL_DISABLED_BUTTON_CLASS : SMALL_BUTTON_CLASS}
            >
              {t("train.attemptPagination.prev")}
            </button>
            <button
              type="button"
              onClick={() => setAttemptsPage((current) => Math.min(totalAttemptPages, current + 1))}
              disabled={attemptsPage >= totalAttemptPages}
              className={attemptsPage >= totalAttemptPages ? SMALL_DISABLED_BUTTON_CLASS : SMALL_BUTTON_CLASS}
            >
              {t("train.attemptPagination.next")}
            </button>
          </div>
        </div>
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
                <th className="pb-2 pr-3">{t("train.attemptTable.finished")}</th>
                <th className="pb-2 pr-3">{t("train.attemptTable.errorCode")}</th>
                <th className="pb-2">{t("train.attemptTable.error")}</th>
              </tr>
            </thead>
            <tbody>
              {pagedAttempts.map((attempt) => (
                <tr key={attempt.id} className="border-t border-blossom-100">
                  <td className="py-2 pr-3">{attempt.action}</td>
                  <td className="py-2 pr-3">{attempt.provider}</td>
                  <td className="py-2 pr-3">{attempt.ok ? "yes" : "no"}</td>
                  <td className="py-2 pr-3">{attempt.retryable ? "yes" : "no"}</td>
                  <td className="py-2 pr-3">{attempt.duration_ms}ms</td>
                  <td className="py-2 pr-3">{formatDateTimeKstSeconds(attempt.started_at, locale)}</td>
                  <td className="py-2 pr-3">{formatDateTimeKstSeconds(attempt.finished_at, locale)}</td>
                  <td className="py-2 pr-3">{attempt.error_code || "-"}</td>
                  <td className="py-2">
                    <p>{attempt.error_message_safe || "-"}</p>
                    {attempt.meta_json_safe ? (
                      <details className="mt-1">
                        <summary className="cursor-pointer text-xs text-slate-500">{t("train.attemptTable.metadata")}</summary>
                        <pre className="mt-1 overflow-auto rounded bg-slate-50 p-2 text-xs text-slate-700">
                          {JSON.stringify(attempt.meta_json_safe, null, 2)}
                        </pre>
                      </details>
                    ) : null}
                  </td>
                </tr>
              ))}
              {pagedAttempts.length === 0 ? (
                <tr>
                  <td className="py-2 text-slate-500" colSpan={9}>
                    {t("train.empty.attempts")}
                  </td>
                </tr>
              ) : null}
            </tbody>
          </table>
        </div>
      </div>
    </section>
  );
}
