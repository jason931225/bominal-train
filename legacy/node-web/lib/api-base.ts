/**
 * API base URL configuration for bominal.
 *
 * Cookie-session auth uses SameSite=Lax; browser requests must stay same-origin.
 * Use relative client URLs so auth cookies are accepted consistently.
 */

function trimTrailingSlashes(value: string): string {
  return value.replace(/\/+$/, "");
}

/** API base URL for client-side requests (browser context). */
export const clientApiBaseUrl = "";

/**
 * Optional dedicated events base URL for browser SSE.
 *
 * Local dev can point this directly at API (for example `http://localhost:8000`)
 * to avoid Next dev rewrite/proxy stream quirks. Leave empty for same-origin.
 */
export const clientApiEventsBaseUrl = trimTrailingSlashes(
  process.env.NEXT_PUBLIC_API_EVENTS_BASE_URL ?? "",
);

/** API base URL for server-side requests (container/internal network). */
export const serverApiBaseUrl =
  process.env.API_SERVER_URL ?? process.env.NEXT_PUBLIC_API_BASE_URL ?? "http://localhost:8000";
