"use client";

import Link from "next/link";
import { FormEvent, useEffect, useMemo, useRef, useState } from "react";

import { clientApiBaseUrl } from "@/lib/api-base";
import { formatDateTimeKst, kstDateInputValue } from "@/lib/kst";
import { UI_BUTTON_OUTLINE_SM, UI_BUTTON_PRIMARY, UI_BUTTON_DANGER_SM, UI_FIELD } from "@/lib/ui";
import type {
  TrainArtifact,
  TrainCredentialStatusResponse,
  TrainSchedule,
  TrainSeatClass,
  TrainStation,
  TrainTaskSummary,
  WalletPaymentCardStatus,
} from "@/lib/types";

type SearchFormState = {
  dep: string;
  arr: string;
  date: string;
  start: string;
  end: string;
  providers: { SRT: boolean; KTX: boolean };
};

type CreateTaskState = {
  seatClass: TrainSeatClass;
  adults: number;
  children: number;
  autoPay: boolean;
  notify: boolean;
};

type CredentialFormState = {
  username: string;
  password: string;
};

type CredentialProvider = "KTX" | "SRT";

const POLL_MS = 12000;
const CREDENTIAL_STATUS_TIMEOUT_MS = 10000;
const DEFAULT_DEP_STATION = "수서";
const DEFAULT_ARR_STATION = "마산";
const TASK_LIST_ERROR_MESSAGE = "Could not load task lists.";
const SESSION_EXPIRED_MESSAGE = "Session expired. Please log in again.";

/**
 * Normalize Korean phone numbers to 11-digit format (e.g., 01012345678).
 * Handles: 010-1234-5678, 010 1234 5678, +82-10-1234-5678, +8210-1234-5678, etc.
 * Returns original input if not a recognizable phone pattern.
 */
function normalizePhoneNumber(input: string): string {
  const trimmed = input.trim();
  // Remove all non-digit characters except leading +
  let digits = trimmed.replace(/[^\d+]/g, "");
  
  // Handle Korean international format (+82)
  if (digits.startsWith("+82")) {
    digits = "0" + digits.slice(3);
  } else if (digits.startsWith("82") && digits.length >= 11) {
    digits = "0" + digits.slice(2);
  }
  
  // Remove any remaining + signs
  digits = digits.replace(/\+/g, "");
  
  // Check if it looks like a Korean mobile number (starts with 01)
  if (/^01[0-9]/.test(digits) && digits.length >= 10 && digits.length <= 11) {
    return digits;
  }
  
  // Not a phone number pattern, return original trimmed input
  return trimmed;
}

const ACTIVE_TASK_STATES = new Set([
  "QUEUED",
  "RUNNING",
  "POLLING",
  "RESERVING",
  "PAYING",
  "PAUSED",
]);
const FIELD_BASE_CLASS = `mt-1 ${UI_FIELD}`;
const PRIMARY_BUTTON_CLASS = UI_BUTTON_PRIMARY;
const SMALL_BUTTON_CLASS = UI_BUTTON_OUTLINE_SM;
const SMALL_DANGER_BUTTON_CLASS = UI_BUTTON_DANGER_SM;
const SMALL_SUCCESS_BUTTON_CLASS =
  "inline-flex h-8 items-center justify-center rounded-full border border-emerald-200 bg-emerald-50 px-2.5 text-xs font-medium text-emerald-700 shadow-sm transition hover:bg-emerald-100 focus:outline-none focus:ring-2 focus:ring-emerald-100 disabled:cursor-not-allowed disabled:opacity-60";
const SMALL_DISABLED_BUTTON_CLASS =
  "inline-flex h-8 items-center justify-center rounded-full border border-slate-200 bg-slate-100 px-2.5 text-xs font-medium text-slate-500 shadow-sm transition focus:outline-none focus:ring-2 focus:ring-slate-100";
const SEAT_CLASS_LABELS: Record<TrainSeatClass, string> = {
  general_preferred: "General Preferred",
  general: "General",
  special_preferred: "Special Preferred",
  special: "Special",
};

const EMPTY_CREDENTIAL_STATUS: TrainCredentialStatusResponse = {
  ktx: { configured: false, verified: false, username: null, verified_at: null, detail: "Credentials are missing" },
  srt: { configured: false, verified: false, username: null, verified_at: null, detail: "Credentials are missing" },
};

async function fetchAllTasks(options?: { refreshCompleted?: boolean }) {
  const query = new URLSearchParams({ status: "all" });
  if (options?.refreshCompleted) {
    query.set("refresh_completed", "true");
  }

  const response = await fetch(`${clientApiBaseUrl}/api/train/tasks?${query.toString()}`, {
    credentials: "include",
    cache: "no-store",
  });

  if (!response.ok) {
    if (response.status === 401) {
      throw new Error(SESSION_EXPIRED_MESSAGE);
    }
    throw new Error(TASK_LIST_ERROR_MESSAGE);
  }

  const payload = (await response.json()) as { tasks: TrainTaskSummary[] };
  return payload.tasks;
}

async function parseApiErrorMessage(response: Response, fallback: string): Promise<string> {
  const contentType = response.headers.get("content-type") ?? "";
  if (contentType.includes("application/json")) {
    const payload = (await response.json().catch(() => null)) as { detail?: string } | null;
    if (payload?.detail) {
      return payload.detail;
    }
    return fallback;
  }

  const text = await response.text().catch(() => "");
  const trimmed = text.trim();
  if (!trimmed) {
    return fallback;
  }
  return trimmed.length > 160 ? `${trimmed.slice(0, 160)}...` : trimmed;
}

function formatTransitDuration(departureAt: string, arrivalAt: string): string {
  const departure = new Date(departureAt);
  const arrival = new Date(arrivalAt);
  if (Number.isNaN(departure.getTime()) || Number.isNaN(arrival.getTime())) {
    return "-";
  }

  const totalMinutes = Math.max(0, Math.round((arrival.getTime() - departure.getTime()) / 60000));
  const hours = Math.floor(totalMinutes / 60);
  const minutes = totalMinutes % 60;

  if (hours === 0) return `${minutes}m`;
  if (minutes === 0) return `${hours}h`;
  return `${hours}h ${minutes}m`;
}

function formatTimeKst(value: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return "-";
  }
  const time = new Intl.DateTimeFormat("ko-KR", {
    timeZone: "Asia/Seoul",
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  }).format(date);
  return time;
}

function formatTicketStatus(value: string | null | undefined): string | null {
  if (!value) return null;
  const words = value.split("_").filter(Boolean);
  if (words.length === 0) return null;
  return words.map((word) => word[0].toUpperCase() + word.slice(1)).join(" ");
}

function getTaskTicketBadge(task: TrainTaskSummary): { label: string; className: string } | null {
  const status = task.ticket_status ?? null;
  const paid = task.ticket_paid === true;

  if (!status && !paid) return null;
  if (status === "cancelled") {
    return { label: "Cancelled", className: "bg-slate-100 text-slate-700" };
  }
  if (status === "reservation_not_found") {
    return { label: "Reservation Not Found", className: "bg-rose-100 text-rose-700" };
  }
  if (status === "awaiting_payment" && !paid) {
    return { label: "Awaiting Payment", className: "bg-amber-100 text-amber-700" };
  }
  if (paid) {
    return { label: "Confirmed", className: "bg-emerald-100 text-emerald-700" };
  }
  return {
    label: formatTicketStatus(status) ?? status ?? "Unknown",
    className: "bg-slate-100 text-slate-700",
  };
}

function isAwaitingPaymentTask(task: TrainTaskSummary): boolean {
  return task.ticket_status === "awaiting_payment" && task.ticket_paid !== true;
}

function shouldShowCompletedCancel(task: TrainTaskSummary): boolean {
  if (task.state !== "COMPLETED") return false;
  if (task.ticket_status === "cancelled") return false;
  if (task.ticket_status === "reservation_not_found" && task.ticket_paid !== true) return false;
  return true;
}

function formatScheduleTitleDate(value: string): string {
  if (!value) return "MM/DD/YYYY";
  const [year, month, day] = value.split("-");
  if (!year || !month || !day) return "MM/DD/YYYY";
  return `${month.padStart(2, "0")}/${day.padStart(2, "0")}/${year}`;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function readInteger(value: unknown): number | null {
  return typeof value === "number" && Number.isFinite(value) ? Math.trunc(value) : null;
}

function taskInfoFromSpec(task: TrainTaskSummary): {
  scheduleLabel: string;
  dep: string;
  arr: string;
  passengerLabel: string;
} {
  const fallback = {
    scheduleLabel: "-",
    dep: "-",
    arr: "-",
    passengerLabel: "-",
  };

  if (!isRecord(task.spec_json)) {
    return fallback;
  }

  const dep = typeof task.spec_json.dep === "string" && task.spec_json.dep.length > 0 ? task.spec_json.dep : "-";
  const arr = typeof task.spec_json.arr === "string" && task.spec_json.arr.length > 0 ? task.spec_json.arr : "-";
  const dateString = typeof task.spec_json.date === "string" ? task.spec_json.date : "";
  const dateLabel = formatScheduleTitleDate(dateString);

  const rankedRaw = Array.isArray(task.spec_json.selected_trains_ranked) ? task.spec_json.selected_trains_ranked : [];
  const ranked = rankedRaw
    .filter(isRecord)
    .map((row) => ({
      rank: readInteger(row.rank) ?? 999,
      departureAt: typeof row.departure_at === "string" ? row.departure_at : "",
    }))
    .filter((row) => row.departureAt.length > 0)
    .sort((a, b) => a.rank - b.rank);

  let scheduleLabel = "-";
  if (ranked.length > 0) {
    const firstTime = formatTimeKst(ranked[0].departureAt);
    scheduleLabel = `${dateLabel} ${firstTime}`;
    if (ranked.length > 1) {
      scheduleLabel += ` (+${ranked.length - 1})`;
    }
  } else if (dateString) {
    scheduleLabel = dateLabel;
  }

  const passengersRaw = isRecord(task.spec_json.passengers) ? task.spec_json.passengers : {};
  const adults = Math.max(0, readInteger(passengersRaw.adults) ?? 0);
  const children = Math.max(0, readInteger(passengersRaw.children) ?? 0);
  const adultLabel = `${adults} adult${adults === 1 ? "" : "s"}`;
  const childLabel = `${children} child${children === 1 ? "" : "ren"}`;

  return {
    scheduleLabel,
    dep,
    arr,
    passengerLabel: `${adultLabel}, ${childLabel}`,
  };
}

export function TrainDashboard() {
  const [searchForm, setSearchForm] = useState<SearchFormState>({
    dep: DEFAULT_DEP_STATION,
    arr: DEFAULT_ARR_STATION,
    date: kstDateInputValue(new Date()),
    start: "06:00",
    end: "23:59",
    providers: { SRT: true, KTX: true },
  });
  const [createForm, setCreateForm] = useState<CreateTaskState>({
    seatClass: "general_preferred",
    adults: 1,
    children: 0,
    autoPay: false,
    notify: false,
  });
  const [searching, setSearching] = useState(false);
  const [hasSearched, setHasSearched] = useState(false);
  const [schedules, setSchedules] = useState<TrainSchedule[]>([]);
  const [selectedScheduleIds, setSelectedScheduleIds] = useState<string[]>([]);
  const [creatingTask, setCreatingTask] = useState(false);
  const [cancellingTaskId, setCancellingTaskId] = useState<string | null>(null);
  const [payingTaskId, setPayingTaskId] = useState<string | null>(null);
  const [signingOutProvider, setSigningOutProvider] = useState<CredentialProvider | null>(null);
  const [activeTasks, setActiveTasks] = useState<TrainTaskSummary[]>([]);
  const [completedTasks, setCompletedTasks] = useState<TrainTaskSummary[]>([]);
  const [stations, setStations] = useState<TrainStation[]>([]);
  const [stationsLoading, setStationsLoading] = useState(false);
  const [credentialStatus, setCredentialStatus] = useState<TrainCredentialStatusResponse | null>(null);
  const [paymentCardStatus, setPaymentCardStatus] = useState<WalletPaymentCardStatus | null>(null);
  const [credentialLoading, setCredentialLoading] = useState(false);
  const [credentialSubmitting, setCredentialSubmitting] = useState(false);
  const [credentialProvider, setCredentialProvider] = useState<CredentialProvider | null>(null);
  const [credentialPanelOpen, setCredentialPanelOpen] = useState(false);
  const [omittedProviders, setOmittedProviders] = useState<Set<CredentialProvider>>(new Set());
  const [credentialForm, setCredentialForm] = useState<CredentialFormState>({ username: "", password: "" });
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const tasksLoadInFlight = useRef(false);

  const scheduleById = useMemo(() => {
    const map = new Map<string, TrainSchedule>();
    for (const schedule of schedules) {
      map.set(schedule.schedule_id, schedule);
    }
    return map;
  }, [schedules]);

  const selectedSchedules = useMemo(
    () => selectedScheduleIds.map((id) => scheduleById.get(id)).filter(Boolean) as TrainSchedule[],
    [selectedScheduleIds, scheduleById]
  );
  const selectedDateLabel = useMemo(() => formatScheduleTitleDate(searchForm.date), [searchForm.date]);

  const ktxVerified = Boolean(credentialStatus?.ktx.verified);
  const srtVerified = Boolean(credentialStatus?.srt.verified);
  const selectedProviderCount = Number(searchForm.providers.SRT) + Number(searchForm.providers.KTX);
  const hasSearchResults = schedules.length > 0;
  const showRanking = hasSearched && hasSearchResults;
  const selectedProviders = new Set(selectedSchedules.map((schedule) => schedule.provider));
  const selectedProviderList = Array.from(selectedProviders).sort();
  const suggestedCredentialProvider =
    credentialStatus == null
      ? "KTX"
      : !credentialStatus.ktx.verified && !omittedProviders.has("KTX")
        ? "KTX"
        : !credentialStatus.srt.verified && !omittedProviders.has("SRT")
          ? "SRT"
          : null;
  const activeCredentialProvider = credentialProvider ?? suggestedCredentialProvider;
  const hasAnyConnectedProvider = ktxVerified || srtVerified;
  const searchUnlocked = credentialStatus != null && hasAnyConnectedProvider;
  const searchDisabled = searching || selectedProviderCount === 0 || !searchUnlocked;
  const createDisabled = !showRanking || selectedSchedules.length === 0 || creatingTask;
  const autoPayAvailable = Boolean(paymentCardStatus?.configured);

  const reloadTasks = async (options?: { refreshCompleted?: boolean }) => {
    if (tasksLoadInFlight.current) return;
    tasksLoadInFlight.current = true;
    try {
      const allTasks = await fetchAllTasks(options);
      const active = allTasks.filter((task) => ACTIVE_TASK_STATES.has(task.state));
      const completed = allTasks.filter((task) => !ACTIVE_TASK_STATES.has(task.state));
      setActiveTasks(active);
      setCompletedTasks(completed);
      setErrorMessage((current) => {
        if (current === TASK_LIST_ERROR_MESSAGE || current === SESSION_EXPIRED_MESSAGE) {
          return null;
        }
        return current;
      });
    } catch (error) {
      const message = error instanceof Error ? error.message : TASK_LIST_ERROR_MESSAGE;
      setErrorMessage((current) => {
        if (current && current !== TASK_LIST_ERROR_MESSAGE && current !== SESSION_EXPIRED_MESSAGE) {
          return current;
        }
        return message;
      });
    } finally {
      tasksLoadInFlight.current = false;
    }
  };

  const loadCredentialStatus = async () => {
    setCredentialLoading(true);
    const abortController = new AbortController();
    const timeoutHandle = window.setTimeout(() => abortController.abort(), CREDENTIAL_STATUS_TIMEOUT_MS);
    try {
      const response = await fetch(`${clientApiBaseUrl}/api/train/credentials/status`, {
        credentials: "include",
        cache: "no-store",
        signal: abortController.signal,
      });
      if (!response.ok) {
        throw new Error("failed");
      }

      const payload = (await response.json()) as TrainCredentialStatusResponse;
      setCredentialStatus(payload);
      setOmittedProviders((current) => {
        const next = new Set(current);
        if (payload.ktx.verified) next.delete("KTX");
        if (payload.srt.verified) next.delete("SRT");
        return next;
      });

      setCredentialProvider((currentProvider) => {
        if (!currentProvider) return null;
        const currentStatus = currentProvider === "KTX" ? payload.ktx : payload.srt;
        return currentStatus.verified ? null : currentProvider;
      });
      if (payload.ktx.verified && payload.srt.verified) {
        setCredentialPanelOpen(false);
      }
    } catch (error) {
      setCredentialStatus((current) => current ?? EMPTY_CREDENTIAL_STATUS);
      if (error instanceof DOMException && error.name === "AbortError") {
        setErrorMessage("Credential check exceeded 10 seconds. You can continue by connecting providers manually.");
      } else {
        setErrorMessage("Could not load provider credential status.");
      }
    } finally {
      window.clearTimeout(timeoutHandle);
      setCredentialStatus((current) => current ?? EMPTY_CREDENTIAL_STATUS);
      setCredentialLoading(false);
    }
  };

  const loadPaymentCardStatus = async () => {
    try {
      const response = await fetch(`${clientApiBaseUrl}/api/wallet/payment-card`, {
        credentials: "include",
        cache: "no-store",
      });
      if (!response.ok) {
        throw new Error("failed");
      }
      const payload = (await response.json()) as WalletPaymentCardStatus;
      setPaymentCardStatus(payload);
    } catch {
      setErrorMessage((current) => current ?? "Could not load wallet status.");
    }
  };

  useEffect(() => {
    void loadCredentialStatus();
  }, []);

  useEffect(() => {
    void loadPaymentCardStatus();
  }, []);

  useEffect(() => {
    setCreateForm((current) => {
      if (!autoPayAvailable) {
        return current.autoPay ? { ...current, autoPay: false } : current;
      }
      return current.autoPay ? current : { ...current, autoPay: true };
    });
  }, [autoPayAvailable]);

  useEffect(() => {
    if (!activeCredentialProvider || !credentialStatus) return;
    const statusInfo = activeCredentialProvider === "KTX" ? credentialStatus.ktx : credentialStatus.srt;
    setCredentialForm({
      username: statusInfo.username || "",
      password: "",
    });
  }, [activeCredentialProvider, credentialStatus]);

  useEffect(() => {
    if (!credentialStatus) return;

    setSearchForm((current) => ({
      ...current,
      providers: {
        SRT: srtVerified,
        KTX: ktxVerified,
      },
    }));
  }, [credentialStatus, ktxVerified, srtVerified]);

  useEffect(() => {
    if (!hasAnyConnectedProvider) {
      setCredentialPanelOpen(true);
      return;
    }
    if (ktxVerified && srtVerified) {
      setCredentialPanelOpen(false);
    }
  }, [hasAnyConnectedProvider, ktxVerified, srtVerified]);

  useEffect(() => {
    const tick = async () => {
      if (document.visibilityState === "hidden") {
        return;
      }
      await reloadTasks();
    };

    void reloadTasks({ refreshCompleted: true });
    const interval = window.setInterval(() => {
      void tick();
    }, POLL_MS);

    const onVisibilityChange = () => {
      if (document.visibilityState === "visible") {
        void tick();
      }
    };

    document.addEventListener("visibilitychange", onVisibilityChange);
    return () => {
      window.clearInterval(interval);
      document.removeEventListener("visibilitychange", onVisibilityChange);
    };
  }, []);

  useEffect(() => {
    let alive = true;
    const loadStations = async () => {
      setStationsLoading(true);
      try {
        const response = await fetch(`${clientApiBaseUrl}/api/train/stations`, {
          credentials: "include",
          cache: "no-store",
        });
        if (!response.ok) {
          return;
        }
        const payload = (await response.json()) as { stations: TrainStation[] };
        if (!alive) {
          return;
        }
        setStations(payload.stations);
        if (payload.stations.length > 0) {
          const names = new Set(payload.stations.map((station) => station.name));
          setSearchForm((current) => ({
            ...current,
            dep: names.has(current.dep)
              ? current.dep
              : names.has(DEFAULT_DEP_STATION)
                ? DEFAULT_DEP_STATION
                : payload.stations[0].name,
            arr: names.has(current.arr)
              ? current.arr
              : names.has(DEFAULT_ARR_STATION)
                ? DEFAULT_ARR_STATION
                : payload.stations[Math.min(1, payload.stations.length - 1)].name,
          }));
        }
      } finally {
        if (alive) {
          setStationsLoading(false);
        }
      }
    };
    void loadStations();
    return () => {
      alive = false;
    };
  }, []);

  const onSearch = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setSearching(true);
    setErrorMessage(null);
    setNotice(null);

    if (!searchUnlocked) {
      setErrorMessage("Connect to a provider to unlock search.");
      setSearching(false);
      return;
    }

    const providers: Array<"SRT" | "KTX"> = [];
    if (searchForm.providers.SRT) providers.push("SRT");
    if (searchForm.providers.KTX) providers.push("KTX");

    if (providers.length === 0) {
      setErrorMessage("Select at least one provider.");
      setSearching(false);
      return;
    }

    try {
      const response = await fetch(`${clientApiBaseUrl}/api/train/search`, {
        method: "POST",
        credentials: "include",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          providers,
          dep: searchForm.dep,
          arr: searchForm.arr,
          date: searchForm.date,
          time_window: {
            start: searchForm.start,
            end: searchForm.end,
          },
        }),
      });

      if (!response.ok) {
        const payload = (await response.json().catch(() => null)) as { detail?: string } | null;
        setErrorMessage(payload?.detail ?? "Search failed.");
        setSchedules([]);
        return;
      }

      const payload = (await response.json()) as { schedules: TrainSchedule[] };
      setHasSearched(true);
      setSchedules(payload.schedules);
      setSelectedScheduleIds([]);
      if (payload.schedules.length === 0) {
        setNotice("No schedules in this window.");
      }
    } catch {
      setErrorMessage("Could not reach API for search.");
    } finally {
      setSearching(false);
    }
  };

  const onSubmitCredentials = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!activeCredentialProvider) return;
    const provider = activeCredentialProvider;

    setCredentialSubmitting(true);
    setErrorMessage(null);
    setNotice(null);

    try {
      const response = await fetch(`${clientApiBaseUrl}/api/train/credentials/${provider.toLowerCase()}`, {
        method: "POST",
        credentials: "include",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          username: normalizePhoneNumber(credentialForm.username),
          password: credentialForm.password,
        }),
      });

      const payload = (await response.json().catch(() => null)) as { detail?: string } | null;
      if (!response.ok) {
        setErrorMessage(payload?.detail ?? `${provider} login failed.`);
        return;
      }

      setCredentialForm((current) => ({ ...current, password: "" }));
      setOmittedProviders((current) => {
        const next = new Set(current);
        next.delete(provider);
        return next;
      });
      setNotice(`${provider} credentials verified.`);
      setCredentialProvider(null);
      await loadCredentialStatus();
    } catch {
      setErrorMessage(`Could not verify ${provider} credentials.`);
    } finally {
      setCredentialSubmitting(false);
    }
  };

  const continueWithoutProvider = (provider: CredentialProvider) => {
    setOmittedProviders((current) => {
      const next = new Set(current);
      next.add(provider);
      return next;
    });
    setCredentialProvider(null);
    setCredentialForm({ username: "", password: "" });
    setNotice(`Continuing without ${provider}. ${provider} search is disabled.`);
  };

  const signOutProvider = async (provider: CredentialProvider) => {
    const confirmed = window.confirm(`Sign out ${provider} credentials?`);
    if (!confirmed) return;

    setErrorMessage(null);
    setNotice(null);
    setSigningOutProvider(provider);

    try {
      const response = await fetch(`${clientApiBaseUrl}/api/train/credentials/${provider.toLowerCase()}/signout`, {
        method: "POST",
        credentials: "include",
      });
      if (!response.ok) {
        const detail = await parseApiErrorMessage(response, `Could not sign out ${provider}.`);
        setErrorMessage(detail);
        return;
      }

      setOmittedProviders((current) => {
        const next = new Set(current);
        next.delete(provider);
        return next;
      });
      setCredentialProvider(null);
      setCredentialForm({ username: "", password: "" });
      setNotice(`${provider} signed out.`);
      await loadCredentialStatus();
    } catch {
      setErrorMessage(`Could not sign out ${provider}.`);
    } finally {
      setSigningOutProvider((current) => (current === provider ? null : current));
    }
  };

  const toggleSelectedSchedule = (scheduleId: string) => {
    setSelectedScheduleIds((current) => {
      if (current.includes(scheduleId)) {
        return current.filter((id) => id !== scheduleId);
      }
      return [...current, scheduleId];
    });
  };

  const moveRank = (index: number, direction: "up" | "down") => {
    setSelectedScheduleIds((current) => {
      const next = [...current];
      const target = direction === "up" ? index - 1 : index + 1;
      if (target < 0 || target >= current.length) {
        return current;
      }
      const item = next[index];
      next[index] = next[target];
      next[target] = item;
      return next;
    });
  };

  const createTask = async () => {
    setCreatingTask(true);
    setErrorMessage(null);
    setNotice(null);

    if (selectedSchedules.length === 0) {
      setErrorMessage("Select at least one schedule for strict Task creation.");
      setCreatingTask(false);
      return;
    }

    const ranked = selectedSchedules.map((schedule, index) => ({
      schedule_id: schedule.schedule_id,
      departure_at: schedule.departure_at,
      rank: index + 1,
      provider: schedule.provider,
    }));

    try {
      const response = await fetch(`${clientApiBaseUrl}/api/train/tasks`, {
        method: "POST",
        credentials: "include",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          dep: searchForm.dep,
          arr: searchForm.arr,
          date: searchForm.date,
          selected_trains_ranked: ranked,
          passengers: {
            adults: createForm.adults,
            children: createForm.children,
          },
          seat_class: createForm.seatClass,
          auto_pay: createForm.autoPay && autoPayAvailable,
          notify: createForm.notify,
        }),
      });

      if (!response.ok) {
        const detail = await parseApiErrorMessage(response, "Could not create Task.");
        setErrorMessage(detail);
        return;
      }

      const payload = (await response.json()) as {
        task: TrainTaskSummary;
        deduplicated: boolean;
      };

      setNotice(payload.deduplicated ? "Task already active (deduplicated)." : "Task created and queued.");
      await reloadTasks();
    } catch {
      setErrorMessage("Could not create Task.");
    } finally {
      setCreatingTask(false);
    }
  };

  const sendTaskAction = async (taskId: string, action: "pause" | "resume" | "cancel" | "delete") => {
    if (action === "cancel") {
      const confirmed = window.confirm("Cancel this active task?");
      if (!confirmed) return;
    }

    setErrorMessage(null);
    try {
      const response = await fetch(`${clientApiBaseUrl}/api/train/tasks/${taskId}/${action}`, {
        method: "POST",
        credentials: "include",
      });
      if (!response.ok) {
        const detail = await parseApiErrorMessage(response, `Could not ${action} task.`);
        setErrorMessage(detail);
        return;
      }
      await reloadTasks();
    } catch {
      setErrorMessage(`Could not ${action} task.`);
    }
  };

  const cancelTaskTicket = async (taskId: string) => {
    const confirmed = window.confirm("Cancel this reservation ticket?");
    if (!confirmed) return;

    setErrorMessage(null);
    setNotice(null);
    setCancellingTaskId(taskId);

    try {
      const detailResponse = await fetch(`${clientApiBaseUrl}/api/train/tasks/${taskId}`, {
        credentials: "include",
        cache: "no-store",
      });
      if (!detailResponse.ok) {
        const detail = await parseApiErrorMessage(detailResponse, "Could not load task detail.");
        setErrorMessage(detail);
        return;
      }

      const detailPayload = (await detailResponse.json()) as { artifacts: TrainArtifact[] };
      const ticketArtifact = detailPayload.artifacts.find((artifact) => artifact.kind === "ticket");
      if (!ticketArtifact) {
        setErrorMessage("No ticket artifact found for this task.");
        return;
      }

      const cancelResponse = await fetch(`${clientApiBaseUrl}/api/train/tickets/${ticketArtifact.id}/cancel`, {
        method: "POST",
        credentials: "include",
      });
      const cancelPayload = (await cancelResponse.json().catch(() => null)) as { detail?: string } | null;
      if (!cancelResponse.ok) {
        setErrorMessage(cancelPayload?.detail ?? "Could not cancel ticket.");
        return;
      }

      setNotice(cancelPayload?.detail ?? "Ticket cancellation request completed.");
      await reloadTasks();
    } catch {
      setErrorMessage("Could not cancel ticket.");
    } finally {
      setCancellingTaskId((current) => (current === taskId ? null : current));
    }
  };

  const payAwaitingPaymentTask = async (taskId: string) => {
    const confirmed = window.confirm("Process payment for this awaiting reservation now?");
    if (!confirmed) return;

    setErrorMessage(null);
    setNotice(null);
    setPayingTaskId(taskId);
    try {
      const response = await fetch(`${clientApiBaseUrl}/api/train/tasks/${taskId}/pay`, {
        method: "POST",
        credentials: "include",
      });
      if (!response.ok) {
        const detail = await parseApiErrorMessage(response, "Could not process payment.");
        setErrorMessage(detail);
        return;
      }

      setNotice("Payment processed.");
      await reloadTasks({ refreshCompleted: true });
    } catch {
      setErrorMessage("Could not process payment.");
    } finally {
      setPayingTaskId((current) => (current === taskId ? null : current));
    }
  };

  return (
    <section className="space-y-8">
      {errorMessage ? <p className="rounded-xl bg-rose-50 px-3 py-2 text-sm text-rose-700">{errorMessage}</p> : null}
      {notice ? <p className="rounded-xl bg-emerald-50 px-3 py-2 text-sm text-emerald-700">{notice}</p> : null}

      <div className="rounded-2xl border border-blossom-100 bg-white p-6 shadow-petal">
        <div className="flex items-center justify-between gap-3">
          <h2 className="text-lg font-semibold text-slate-800">Search schedules</h2>
          <button
            type="button"
            onClick={() => {
              setCredentialPanelOpen((current) => {
                const next = !current;
                if (!next) {
                  setCredentialProvider(null);
                }
                return next;
              });
            }}
            aria-label={credentialPanelOpen ? "Hide provider credentials" : "Show provider credentials"}
            title={credentialPanelOpen ? "Hide provider credentials" : "Show provider credentials"}
            className={`inline-flex h-10 w-10 items-center justify-center rounded-full border shadow-sm transition focus:outline-none focus:ring-2 ${
              searchUnlocked
                ? "border-emerald-200 bg-emerald-50 text-emerald-700 hover:bg-emerald-100 focus:ring-emerald-100"
                : "border-slate-200 bg-slate-100 text-slate-500 hover:bg-slate-200 focus:ring-slate-200"
            }`}
          >
            <svg viewBox="0 0 24 24" className="h-5 w-5" fill="none" stroke="currentColor" strokeWidth="1.8">
              <path d="M8 11V8a4 4 0 1 1 8 0v3" strokeLinecap="round" />
              <rect x="6" y="11" width="12" height="9" rx="2.5" />
            </svg>
          </button>
        </div>

        {!searchUnlocked ? (
          <p className="mt-2 text-xs text-amber-700">Connect to a provider to unlock search.</p>
        ) : null}

        {credentialPanelOpen ? (
          <div className="mt-4 rounded-2xl border border-blossom-100 bg-blossom-50/30 p-4">
            {credentialLoading ? (
              <p className="text-sm text-slate-500">Checking provider credentials...</p>
            ) : (
              <>
                <div className="grid gap-3 md:grid-cols-2">
                  {(["KTX", "SRT"] as const).map((provider) => {
                    const statusInfo = provider === "KTX" ? credentialStatus?.ktx : credentialStatus?.srt;
                    const isVerified = Boolean(statusInfo?.verified);
                    const isSkipped = omittedProviders.has(provider) && !isVerified;
                    const username = statusInfo?.username || "-";
                    return (
                      <div key={provider} className="rounded-xl border border-blossom-100 bg-white px-3 py-3">
                        <div className="flex items-start justify-between gap-3">
                          <div className="min-w-0">
                            <p className="text-xs uppercase tracking-[0.14em] text-blossom-500">{provider}</p>
                            <p className="mt-1 break-all text-sm font-medium text-slate-700">{username}</p>
                            <p className={`text-xs ${isVerified ? "text-emerald-600" : "text-amber-700"}`}>
                              {isVerified ? "Connected" : isSkipped ? "Skipped (disabled)" : statusInfo?.detail || "Not connected"}
                            </p>
                          </div>
                          <div className="flex shrink-0 flex-col gap-2 self-center">
                            <button
                              type="button"
                              onClick={() => {
                                setCredentialProvider(provider);
                                setCredentialForm({
                                  username: statusInfo?.username ?? "",
                                  password: "",
                                });
                              }}
                              className={SMALL_BUTTON_CLASS}
                            >
                              {isVerified ? "Change" : "Connect"}
                            </button>
                            <button
                              type="button"
                              onClick={() => void signOutProvider(provider)}
                              disabled={!statusInfo?.configured || signingOutProvider === provider}
                              className={SMALL_DANGER_BUTTON_CLASS}
                            >
                              {signingOutProvider === provider ? "Signing out..." : "Sign out"}
                            </button>
                          </div>
                        </div>
                      </div>
                    );
                  })}
                </div>

                {activeCredentialProvider ? (
                  <form onSubmit={onSubmitCredentials} className="mt-4 rounded-2xl border border-blossom-100 bg-white p-4">
                    <h3 className="text-base font-semibold text-slate-800">{activeCredentialProvider} login required</h3>
                    <p className="mt-1 text-sm text-slate-500">
                      Enter {activeCredentialProvider} credentials. They are encrypted at rest and re-verified automatically.
                    </p>
                    <div className="mt-4 grid gap-3 md:grid-cols-2">
                      <label className="text-sm text-slate-700">
                        {activeCredentialProvider} username
                        <input
                          type="text"
                          value={credentialForm.username}
                          onChange={(event) => setCredentialForm((current) => ({ ...current, username: event.target.value }))}
                          className={FIELD_BASE_CLASS}
                          placeholder="email, phone, or membership #"
                          required
                        />
                      </label>
                      <label className="text-sm text-slate-700">
                        {activeCredentialProvider} password
                        <input
                          type="password"
                          value={credentialForm.password}
                          onChange={(event) => setCredentialForm((current) => ({ ...current, password: event.target.value }))}
                          className={FIELD_BASE_CLASS}
                          required
                        />
                      </label>
                    </div>
                    <div className="mt-4 flex items-center gap-2">
                      <button
                        type="submit"
                        disabled={credentialSubmitting}
                        className={PRIMARY_BUTTON_CLASS}
                      >
                        {credentialSubmitting ? "Verifying..." : `Connect ${activeCredentialProvider}`}
                      </button>
                      <button
                        type="button"
                        onClick={() => continueWithoutProvider(activeCredentialProvider)}
                        disabled={credentialSubmitting}
                        className={SMALL_BUTTON_CLASS}
                      >
                        Continue without {activeCredentialProvider}
                      </button>
                      <button
                        type="button"
                        onClick={() => setCredentialProvider(null)}
                        className={SMALL_BUTTON_CLASS}
                      >
                        Cancel
                      </button>
                    </div>
                  </form>
                ) : null}
              </>
            )}
          </div>
        ) : null}

        {searchUnlocked ? (
          <form onSubmit={onSearch} className="mt-4">
            <div className="grid gap-4 lg:grid-cols-[minmax(0,2fr)_minmax(0,1fr)]">
              <div className="rounded-2xl border border-blossom-100 bg-blossom-50/40 p-4">
                <p className="text-xs font-medium uppercase tracking-[0.14em] text-blossom-500">Station / Date / Time</p>
                <div className="mt-3 grid grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)] items-end gap-2 sm:gap-3">
                  <label className="text-sm text-slate-700">
                    Departure station
                    <select
                      value={searchForm.dep}
                      onChange={(event) => setSearchForm((cur) => ({ ...cur, dep: event.target.value }))}
                      className={FIELD_BASE_CLASS}
                      required
                      disabled={stationsLoading || stations.length === 0 || !searchUnlocked}
                    >
                      {stations.map((station) => (
                        <option key={station.name} value={station.name}>
                          {station.name}
                        </option>
                      ))}
                    </select>
                  </label>
                  <div className="flex items-center justify-center self-end">
                    <button
                      type="button"
                      onClick={() =>
                        setSearchForm((cur) => ({
                          ...cur,
                          dep: cur.arr,
                          arr: cur.dep,
                        }))
                      }
                      disabled={stationsLoading || stations.length === 0 || !searchUnlocked}
                      aria-label="Swap departure and arrival stations"
                      title="Swap departure and arrival stations"
                      className="inline-flex h-10 w-10 items-center justify-center rounded-full border border-blossom-200 bg-blossom-50 text-blossom-700 shadow-sm transition hover:bg-blossom-100 focus:outline-none focus:ring-2 focus:ring-blossom-100 disabled:cursor-not-allowed disabled:opacity-60"
                    >
                      <svg viewBox="0 0 24 24" className="h-5 w-5" fill="none" stroke="currentColor" strokeWidth="1.8">
                        <path d="M4 8h13" strokeLinecap="round" />
                        <path d="m14 5 3 3-3 3" strokeLinecap="round" strokeLinejoin="round" />
                        <path d="M20 16H7" strokeLinecap="round" />
                        <path d="m10 13-3 3 3 3" strokeLinecap="round" strokeLinejoin="round" />
                      </svg>
                    </button>
                  </div>
                  <label className="text-sm text-slate-700">
                    Arrival station
                    <select
                      value={searchForm.arr}
                      onChange={(event) => setSearchForm((cur) => ({ ...cur, arr: event.target.value }))}
                      className={FIELD_BASE_CLASS}
                      required
                      disabled={stationsLoading || stations.length === 0 || !searchUnlocked}
                    >
                      {stations.map((station) => (
                        <option key={station.name} value={station.name}>
                          {station.name}
                        </option>
                      ))}
                    </select>
                  </label>
                </div>

                <div className="mt-3 grid gap-3 md:grid-cols-2">
                  <label className="text-sm text-slate-700">
                    Date
                    <input
                      type="date"
                      value={searchForm.date}
                      onChange={(event) => setSearchForm((cur) => ({ ...cur, date: event.target.value }))}
                      className={FIELD_BASE_CLASS}
                      required
                      disabled={!searchUnlocked}
                    />
                  </label>
                  <div className="grid grid-cols-2 gap-2">
                    <label className="text-sm text-slate-700">
                      Time start
                      <input
                        type="time"
                        value={searchForm.start}
                        onChange={(event) => setSearchForm((cur) => ({ ...cur, start: event.target.value }))}
                        className={FIELD_BASE_CLASS}
                        required
                        disabled={!searchUnlocked}
                      />
                    </label>
                    <label className="text-sm text-slate-700">
                      Time end
                      <input
                        type="time"
                        value={searchForm.end}
                        onChange={(event) => setSearchForm((cur) => ({ ...cur, end: event.target.value }))}
                        className={FIELD_BASE_CLASS}
                        required
                        disabled={!searchUnlocked}
                      />
                    </label>
                  </div>
                </div>

                {!ktxVerified || !srtVerified ? (
                  <p className="mt-3 text-xs text-amber-700">
                    Providers without verified credentials are disabled for search.
                  </p>
                ) : null}

                <div className="mt-4 flex flex-wrap items-center gap-4 text-sm text-slate-700">
                  <label className={`inline-flex items-center gap-2 ${!srtVerified ? "text-slate-400" : ""}`}>
                    <input
                      type="checkbox"
                      checked={searchForm.providers.SRT}
                      disabled={!srtVerified || !searchUnlocked}
                      onChange={(event) =>
                        setSearchForm((cur) => ({ ...cur, providers: { ...cur.providers, SRT: event.target.checked } }))
                      }
                    />
                    SRT
                  </label>
                  <label className={`inline-flex items-center gap-2 ${!ktxVerified ? "text-slate-400" : ""}`}>
                    <input
                      type="checkbox"
                      checked={searchForm.providers.KTX}
                      disabled={!ktxVerified || !searchUnlocked}
                      onChange={(event) =>
                        setSearchForm((cur) => ({ ...cur, providers: { ...cur.providers, KTX: event.target.checked } }))
                      }
                    />
                    KTX
                  </label>
                </div>
              </div>

              <div className="rounded-2xl border border-blossom-100 bg-blossom-50/40 p-4">
                <p className="text-xs font-medium uppercase tracking-[0.14em] text-blossom-500">Passenger / Seat Class</p>
                <div className="mt-3 space-y-3">
                  <label className="text-sm text-slate-700">
                    Seat class
                    <select
                      value={createForm.seatClass}
                      onChange={(event) =>
                        setCreateForm((cur) => ({ ...cur, seatClass: event.target.value as TrainSeatClass }))
                      }
                      className={FIELD_BASE_CLASS}
                    >
                      <option value="general_preferred">General Preferred</option>
                      <option value="general">General</option>
                      <option value="special_preferred">Special Preferred</option>
                      <option value="special">Special</option>
                    </select>
                  </label>

                  <div className="grid grid-cols-2 gap-2">
                    <label className="text-sm text-slate-700">
                      Adults
                      <input
                        type="number"
                        min={1}
                        max={9}
                        value={createForm.adults}
                        onChange={(event) => setCreateForm((cur) => ({ ...cur, adults: Number(event.target.value) }))}
                        className={FIELD_BASE_CLASS}
                      />
                    </label>
                    <label className="text-sm text-slate-700">
                      Children
                      <input
                        type="number"
                        min={0}
                        max={9}
                        value={createForm.children}
                        onChange={(event) => setCreateForm((cur) => ({ ...cur, children: Number(event.target.value) }))}
                        className={FIELD_BASE_CLASS}
                      />
                    </label>
                  </div>

                </div>
              </div>
            </div>

            <div className="mt-4 flex justify-end">
              <button
                type="submit"
                disabled={searchDisabled}
                className={PRIMARY_BUTTON_CLASS}
              >
                {searching ? "Searching..." : "Search"}
              </button>
            </div>
          </form>
        ) : null}
      </div>

      {searchUnlocked && showRanking ? (
            <div className="rounded-2xl border border-blossom-100 bg-white p-6 shadow-petal">
              <h2 className="text-lg font-semibold text-slate-800">Select schedules ({selectedDateLabel})</h2>
              <div className="mt-4 overflow-x-auto">
                <table className="min-w-full table-fixed text-left text-sm">
                  <thead>
                    <tr className="text-slate-500">
                      <th className="px-2 pb-2 text-center">Status</th>
                      <th className="px-2 pb-2">Train</th>
                      <th className="px-2 pb-2">Departure</th>
                      <th className="px-2 pb-2">Arrival</th>
                      <th className="px-2 pb-2">Duration</th>
                      <th className="px-2 pb-2 text-center" colSpan={2}>
                        Availability
                      </th>
                    </tr>
                  </thead>
                  <tbody>
                    {schedules.map((schedule) => {
                      const checked = selectedScheduleIds.includes(schedule.schedule_id);
                      return (
                        <tr
                          key={schedule.schedule_id}
                          role="button"
                          tabIndex={0}
                          aria-pressed={checked}
                          onClick={() => toggleSelectedSchedule(schedule.schedule_id)}
                          onKeyDown={(event) => {
                            if (event.key === "Enter" || event.key === " ") {
                              event.preventDefault();
                              toggleSelectedSchedule(schedule.schedule_id);
                            }
                          }}
                          className={`cursor-pointer border-t border-blossom-100 transition ${
                            checked ? "bg-blossom-100/70" : "hover:bg-blossom-50/50"
                          }`}
                        >
                          <td className="px-2 py-2 align-middle text-center">
                            <span
                              aria-hidden="true"
                              className={`mx-auto inline-flex h-5 w-5 items-center justify-center rounded-full border transition ${
                                checked
                                  ? "border-blossom-500 bg-blossom-500 text-white"
                                  : "border-slate-300 bg-slate-100 text-transparent"
                              }`}
                            >
                              {checked ? (
                                <svg
                                  viewBox="0 0 20 20"
                                  className="h-3.5 w-3.5"
                                  fill="none"
                                  stroke="currentColor"
                                  strokeWidth="2.2"
                                >
                                  <path d="M4 10.5 8 14l8-8" strokeLinecap="round" strokeLinejoin="round" />
                                </svg>
                              ) : null}
                            </span>
                          </td>
                          <td className="px-2 py-2">
                            {schedule.provider} {schedule.train_no}
                          </td>
                          <td className="px-2 py-2">{formatTimeKst(schedule.departure_at)}</td>
                          <td className="px-2 py-2">{formatTimeKst(schedule.arrival_at)}</td>
                          <td className="px-2 py-2">{formatTransitDuration(schedule.departure_at, schedule.arrival_at)}</td>
                          <td className="px-2 py-2 text-center">
                            <span
                              title={schedule.availability.general ? "General available" : "General sold out"}
                              className={`inline-flex h-6 w-6 items-center justify-center rounded-full text-[11px] font-semibold ${
                                schedule.availability.general ? "bg-blossom-500 text-white" : "bg-slate-200 text-slate-500"
                              }`}
                            >
                              G
                            </span>
                          </td>
                          <td className="px-2 py-2 text-center">
                            <span
                              title={schedule.availability.special ? "Special available" : "Special sold out"}
                              className={`inline-flex h-6 w-6 items-center justify-center rounded-full text-[11px] font-semibold ${
                                schedule.availability.special ? "bg-blossom-500 text-white" : "bg-slate-200 text-slate-500"
                              }`}
                            >
                              S
                            </span>
                          </td>
                        </tr>
                      );
                    })}
                  </tbody>
                </table>
              </div>

              <div className="mt-5 rounded-xl border border-blossom-100 bg-blossom-50/50 p-4">
                {selectedSchedules.length === 0 ? (
                  <p className="text-sm text-slate-500">Select schedules from the table to create a strict Task.</p>
                ) : null}
                {selectedSchedules.length === 1 ? (
                  <div className="rounded-lg border border-blossom-100 bg-white px-3 py-2 text-sm text-slate-700">
                    <span className="font-medium">Selected:</span> {selectedSchedules[0].provider} {selectedSchedules[0].train_no} ·{" "}
                    {formatDateTimeKst(selectedSchedules[0].departure_at)}
                  </div>
                ) : null}
                {selectedSchedules.length > 1 ? (
                  <>
                    <p className="text-sm font-medium text-slate-700">Priority order</p>
                    <ul className="mt-3 space-y-2 text-sm">
                      {selectedSchedules.map((schedule, index) => (
                        <li
                          key={schedule.schedule_id}
                          className="flex items-center justify-between rounded-lg border border-blossom-100 bg-white px-3 py-2"
                        >
                          <div className="flex items-center gap-2">
                            <span className="inline-flex h-6 min-w-6 items-center justify-center rounded-full bg-blossom-500 px-2 text-xs font-semibold text-white">
                              {index + 1}
                            </span>
                            <span>
                              {schedule.provider} {schedule.train_no} · {formatDateTimeKst(schedule.departure_at)}
                            </span>
                          </div>
                          <div className="flex items-center gap-2">
                            <button
                              type="button"
                              onClick={() => moveRank(index, "up")}
                              className={SMALL_BUTTON_CLASS}
                            >
                              Up
                            </button>
                            <button
                              type="button"
                              onClick={() => moveRank(index, "down")}
                              className={SMALL_BUTTON_CLASS}
                            >
                              Down
                            </button>
                          </div>
                        </li>
                      ))}
                    </ul>
                  </>
                ) : null}

                <div className="mt-4 flex flex-wrap items-center justify-between gap-3 rounded-lg border border-blossom-100 bg-white px-3 py-3">
                  <div className="text-sm text-slate-600">
                    <p>
                      <span className="font-medium">Provider:</span>{" "}
                      {selectedSchedules.length > 0 ? selectedProviderList.join(" + ") : "Select schedules first"}
                    </p>
                    <p>
                      <span className="font-medium">Seat:</span> {SEAT_CLASS_LABELS[createForm.seatClass]} ·{" "}
                      <span className="font-medium">Passengers:</span> {createForm.adults} adult / {createForm.children} child
                    </p>
                  </div>
                  <div className="flex flex-col gap-1">
                    <div className="flex flex-wrap items-center gap-3">
                      <button
                        type="button"
                        role="switch"
                        aria-checked={createForm.autoPay}
                        onClick={() => {
                          if (!autoPayAvailable) return;
                          setCreateForm((cur) => ({ ...cur, autoPay: !cur.autoPay }));
                        }}
                        disabled={!autoPayAvailable}
                        title={autoPayAvailable ? "Auto-pay" : "Save wallet details first to enable auto-pay"}
                        className={`inline-flex items-center gap-2 rounded-full border px-3 py-1.5 text-xs font-medium transition focus:outline-none focus:ring-2 focus:ring-blossom-100 disabled:cursor-not-allowed disabled:opacity-60 ${
                          createForm.autoPay
                            ? "border-blossom-300 bg-blossom-50 text-blossom-700"
                            : "border-slate-200 bg-white text-slate-600"
                        }`}
                      >
                        <span>Auto-pay</span>
                        <span
                          className={`relative inline-flex h-5 w-9 items-center rounded-full transition ${
                            createForm.autoPay ? "bg-blossom-500" : "bg-slate-300"
                          }`}
                        >
                          <span
                            className={`inline-block h-4 w-4 rounded-full bg-white shadow transition ${
                              createForm.autoPay ? "translate-x-4" : "translate-x-0.5"
                            }`}
                          />
                        </span>
                      </button>

                      <button
                        type="button"
                        role="switch"
                        aria-checked={createForm.notify}
                        onClick={() => setCreateForm((cur) => ({ ...cur, notify: !cur.notify }))}
                        className={`inline-flex items-center gap-2 rounded-full border px-3 py-1.5 text-xs font-medium transition focus:outline-none focus:ring-2 focus:ring-blossom-100 ${
                          createForm.notify
                            ? "border-blossom-300 bg-blossom-50 text-blossom-700"
                            : "border-slate-200 bg-white text-slate-600"
                        }`}
                      >
                        <span>Notify</span>
                        <span
                          className={`relative inline-flex h-5 w-9 items-center rounded-full transition ${
                            createForm.notify ? "bg-blossom-500" : "bg-slate-300"
                          }`}
                        >
                          <span
                            className={`inline-block h-4 w-4 rounded-full bg-white shadow transition ${
                              createForm.notify ? "translate-x-4" : "translate-x-0.5"
                            }`}
                          />
                        </span>
                      </button>

                      <button
                        type="button"
                        onClick={createTask}
                        disabled={createDisabled}
                        className={PRIMARY_BUTTON_CLASS}
                      >
                        {creatingTask ? "Creating Task..." : "Create Task"}
                      </button>
                    </div>

                  </div>
                </div>
                {!autoPayAvailable ? (
                  <div className="mt-3">
                    <div className="inline-flex items-center gap-1 rounded-full border border-amber-200 bg-amber-50/90 px-3 py-1.5 text-xs text-amber-700 shadow-sm">
                      <span className="font-medium">Wallet required for auto-pay.</span>
                      <span>Configure in</span>
                      <Link
                        href="/settings/payment"
                        className="font-medium underline decoration-amber-300 underline-offset-2 hover:text-amber-800"
                      >
                        Payment settings
                      </Link>
                      <span>.</span>
                    </div>
                  </div>
                ) : null}
              </div>
            </div>
          ) : searchUnlocked && hasSearched ? (
            <div className="rounded-2xl border border-blossom-100 bg-white p-6 text-sm text-slate-500 shadow-petal">
              No schedules returned yet. Adjust filters and search again.
            </div>
          ) : null}

      <div className="grid gap-4 lg:grid-cols-2">
        <div className="rounded-2xl border border-blossom-100 bg-white p-6 shadow-petal">
          <h2 className="text-lg font-semibold text-slate-800">Active Tasks</h2>
          <ul className="mt-4 space-y-3 text-sm">
            {activeTasks.length === 0 ? <li className="text-slate-500">No active tasks.</li> : null}
            {activeTasks.map((task) => {
              const info = taskInfoFromSpec(task);
              return (
                <li key={task.id} className="rounded-xl border border-blossom-100 p-3">
                <div className="flex items-center justify-between gap-2">
                  <div>
                    <p className="font-medium text-slate-700">{task.state}</p>
                    <p className="text-xs text-slate-500">
                      Last attempt: {task.last_attempt_at ? formatDateTimeKst(task.last_attempt_at) : "-"}
                    </p>
                    <p className="mt-1 text-xs text-slate-600">Schedule: {info.scheduleLabel}</p>
                    <p className="text-xs text-slate-600">
                      Route: {info.dep} {"->"} {info.arr}
                    </p>
                    <p className="text-xs text-slate-600">Passengers: {info.passengerLabel}</p>
                  </div>
                  <Link href={`/modules/train/tasks/${task.id}`} className="text-xs font-medium text-blossom-600 hover:text-blossom-700">
                    Detail
                  </Link>
                </div>
                <div className="mt-3 flex flex-wrap gap-2">
                  {task.state !== "PAUSED" ? (
                    <button
                      type="button"
                      onClick={() => sendTaskAction(task.id, "pause")}
                      className={SMALL_BUTTON_CLASS}
                    >
                      Pause
                    </button>
                  ) : (
                    <button
                      type="button"
                      onClick={() => sendTaskAction(task.id, "resume")}
                      className={SMALL_BUTTON_CLASS}
                    >
                      Resume
                    </button>
                  )}
                  {isAwaitingPaymentTask(task) ? (
                    <button
                      type="button"
                      onClick={() => void payAwaitingPaymentTask(task.id)}
                      disabled={payingTaskId === task.id}
                      className={payingTaskId === task.id ? SMALL_DISABLED_BUTTON_CLASS : SMALL_SUCCESS_BUTTON_CLASS}
                    >
                      {payingTaskId === task.id ? "Paying..." : "Pay"}
                    </button>
                  ) : null}
                  {isAwaitingPaymentTask(task) ? (
                    <button
                      type="button"
                      onClick={() => void cancelTaskTicket(task.id)}
                      disabled={cancellingTaskId === task.id || payingTaskId === task.id}
                      className={SMALL_DANGER_BUTTON_CLASS}
                    >
                      {cancellingTaskId === task.id ? "Cancelling..." : "Cancel reservation"}
                    </button>
                  ) : null}
                  <button
                    type="button"
                    onClick={() => sendTaskAction(task.id, "cancel")}
                    className={SMALL_DANGER_BUTTON_CLASS}
                  >
                    Cancel
                  </button>
                </div>
                </li>
              );
            })}
          </ul>
        </div>

        <div className="rounded-2xl border border-blossom-100 bg-white p-6 shadow-petal">
          <h2 className="text-lg font-semibold text-slate-800">Completed Tasks</h2>
          <ul className="mt-4 space-y-3 text-sm">
            {completedTasks.length === 0 ? <li className="text-slate-500">No completed tasks.</li> : null}
            {completedTasks.map((task) => {
              const info = taskInfoFromSpec(task);
              const ticketBadge = getTaskTicketBadge(task);
              return (
                <li key={task.id} className="rounded-xl border border-blossom-100 p-3">
                <div className="flex items-center justify-between gap-2">
                  <div>
                    <div className="flex flex-wrap items-center gap-2">
                      <p className="font-medium text-slate-700">{task.state}</p>
                      {ticketBadge ? (
                        <span
                          className={`inline-flex rounded-full px-2 py-0.5 text-[11px] font-medium ${ticketBadge.className}`}
                        >
                          {ticketBadge.label}
                        </span>
                      ) : null}
                    </div>
                    <p className="text-xs text-slate-500">
                      Completed: {task.completed_at ? formatDateTimeKst(task.completed_at) : "-"}
                    </p>
                    <p className="mt-1 text-xs text-slate-600">Schedule: {info.scheduleLabel}</p>
                    <p className="text-xs text-slate-600">
                      Route: {info.dep} {"->"} {info.arr}
                    </p>
                    <p className="text-xs text-slate-600">Passengers: {info.passengerLabel}</p>
                  </div>
                  <Link href={`/modules/train/tasks/${task.id}`} className="text-xs font-medium text-blossom-600 hover:text-blossom-700">
                    Detail
                  </Link>
                </div>
                <div className="mt-3 flex flex-wrap gap-2">
                  {isAwaitingPaymentTask(task) ? (
                    <button
                      type="button"
                      onClick={() => void payAwaitingPaymentTask(task.id)}
                      disabled={payingTaskId === task.id || !autoPayAvailable}
                      title={autoPayAvailable ? "Pay now" : "Payment settings required"}
                      className={
                        payingTaskId === task.id || !autoPayAvailable
                          ? SMALL_DISABLED_BUTTON_CLASS
                          : SMALL_SUCCESS_BUTTON_CLASS
                      }
                    >
                      {payingTaskId === task.id ? "Paying..." : "Pay"}
                    </button>
                  ) : null}
                  {isAwaitingPaymentTask(task) ? (
                    <button
                      type="button"
                      onClick={() => void cancelTaskTicket(task.id)}
                      disabled={cancellingTaskId === task.id || payingTaskId === task.id}
                      className={SMALL_DANGER_BUTTON_CLASS}
                    >
                      {cancellingTaskId === task.id ? "Cancelling..." : "Cancel"}
                    </button>
                  ) : shouldShowCompletedCancel(task) ? (
                    <button
                      type="button"
                      onClick={() => void cancelTaskTicket(task.id)}
                      disabled={cancellingTaskId === task.id}
                      className={SMALL_DANGER_BUTTON_CLASS}
                    >
                      {cancellingTaskId === task.id ? "Cancelling..." : "Cancel"}
                    </button>
                  ) : (
                    <button
                      type="button"
                      onClick={() => sendTaskAction(task.id, "delete")}
                      className={SMALL_DANGER_BUTTON_CLASS}
                    >
                      Delete
                    </button>
                  )}
                </div>
                </li>
              );
            })}
          </ul>
        </div>
      </div>
    </section>
  );
}
