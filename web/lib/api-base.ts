/**
 * API base URL configuration for bominal.
 * 
 * Two URLs are provided for different contexts:
 * - clientApiBaseUrl: Used in browser (client components, fetch from browser)
 * - serverApiBaseUrl: Used in server components/actions (internal networking)
 * 
 * In production with Docker:
 * - NEXT_PUBLIC_API_BASE_URL = "" (relative, goes through Caddy proxy)
 * - API_SERVER_URL = "http://api:8000" (internal container network)
 */

/** API base URL for client-side requests (browser context). */
export const clientApiBaseUrl =
  process.env.NEXT_PUBLIC_API_BASE_URL ?? "";

/** API base URL for server-side requests (container/internal network). */
export const serverApiBaseUrl =
  process.env.API_SERVER_URL ?? process.env.NEXT_PUBLIC_API_BASE_URL ?? "http://localhost:8000";
