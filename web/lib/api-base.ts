/**
 * API base URL configuration for bominal.
 *
 * Cookie-session auth uses SameSite=Lax; browser requests must stay same-origin.
 * Use relative client URLs so auth cookies are accepted consistently.
 */

/** API base URL for client-side requests (browser context). */
export const clientApiBaseUrl = "";

/** API base URL for server-side requests (container/internal network). */
export const serverApiBaseUrl =
  process.env.API_SERVER_URL ?? process.env.NEXT_PUBLIC_API_BASE_URL ?? "http://localhost:8000";
