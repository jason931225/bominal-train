import { clientApiBaseUrl } from "@/lib/api-base";
import type { TrainStation } from "@/lib/types";

type StationsCacheEntry = {
  stations: TrainStation[];
  fetchedAt: number;
};

const STATIONS_CACHE_STORAGE_KEY = "bominal_train_stations_cache_v1";
const DEFAULT_STATIONS_CACHE_TTL_MS = 12 * 60 * 60 * 1000;
const IS_TEST_ENV = process.env.NODE_ENV === "test";

let stationsCacheEntry: StationsCacheEntry | null = null;
let stationsInFlight: Promise<StationsCacheEntry> | null = null;

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function readStations(value: unknown): TrainStation[] {
  if (!Array.isArray(value)) return [];
  const stations: TrainStation[] = [];
  for (const row of value) {
    if (!isRecord(row)) continue;
    if (typeof row.name !== "string") continue;
    stations.push(row as unknown as TrainStation);
  }
  return stations;
}

function isFresh(entry: StationsCacheEntry, ttlMs: number): boolean {
  return Date.now() - entry.fetchedAt <= ttlMs;
}

function readSessionCache(): StationsCacheEntry | null {
  if (typeof window === "undefined") return null;
  try {
    const raw = window.sessionStorage.getItem(STATIONS_CACHE_STORAGE_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as unknown;
    if (!isRecord(parsed)) return null;
    const fetchedAt = typeof parsed.fetchedAt === "number" ? parsed.fetchedAt : null;
    if (!fetchedAt || !Number.isFinite(fetchedAt)) return null;
    return {
      fetchedAt,
      stations: readStations(parsed.stations),
    };
  } catch {
    return null;
  }
}

function writeSessionCache(entry: StationsCacheEntry): void {
  if (typeof window === "undefined") return;
  try {
    window.sessionStorage.setItem(STATIONS_CACHE_STORAGE_KEY, JSON.stringify(entry));
  } catch {
    // Best-effort only.
  }
}

function cacheEntry(entry: StationsCacheEntry): StationsCacheEntry {
  stationsCacheEntry = entry;
  writeSessionCache(entry);
  return entry;
}

async function fetchStationsFromApi(): Promise<StationsCacheEntry> {
  const response = await fetch(`${clientApiBaseUrl}/api/train/stations`, {
    credentials: "include",
    cache: "no-store",
  });
  if (!response.ok) {
    throw new Error("stations_load_error");
  }

  const payload = (await response.json()) as { stations?: unknown };
  return cacheEntry({
    stations: readStations(payload.stations),
    fetchedAt: Date.now(),
  });
}

export function clearTrainStationsCache(): void {
  stationsCacheEntry = null;
  stationsInFlight = null;
  if (typeof window === "undefined") return;
  try {
    window.sessionStorage.removeItem(STATIONS_CACHE_STORAGE_KEY);
  } catch {
    // Best-effort only.
  }
}

export async function getTrainStationsCached(options?: { force?: boolean; ttlMs?: number }): Promise<TrainStation[]> {
  if (IS_TEST_ENV) {
    const fetched = await fetchStationsFromApi();
    return fetched.stations;
  }

  const ttlMs = Math.max(0, options?.ttlMs ?? DEFAULT_STATIONS_CACHE_TTL_MS);
  const cached = stationsCacheEntry ?? readSessionCache();
  if (!options?.force && cached && isFresh(cached, ttlMs)) {
    stationsCacheEntry = cached;
    return cached.stations;
  }

  if (stationsInFlight) {
    const inFlight = await stationsInFlight;
    return inFlight.stations;
  }

  stationsInFlight = fetchStationsFromApi();
  try {
    const fetched = await stationsInFlight;
    return fetched.stations;
  } catch (error) {
    if (cached) {
      return cached.stations;
    }
    throw error;
  } finally {
    stationsInFlight = null;
  }
}
