/**
 * Server-side authentication utilities for Next.js Server Components.
 * 
 * These functions run only on the server and use the session cookie
 * to authenticate requests to the API. Uses React's cache() for
 * request-level deduplication.
 * 
 * @module server-auth
 */

import "server-only";

import { cache } from "react";
import { cookies } from "next/headers";
import { redirect } from "next/navigation";

import { serverApiBaseUrl } from "@/lib/api-base";
import { ROUTES } from "@/lib/routes";
import type { AuthMeResponse, BominalUser } from "@/lib/types";

const SERVER_AUTH_FETCH_TIMEOUT_MS = 3000;

async function fetchWithTimeout(input: string, init: RequestInit): Promise<Response> {
  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), SERVER_AUTH_FETCH_TIMEOUT_MS);
  try {
    return await fetch(input, { ...init, signal: controller.signal });
  } finally {
    clearTimeout(timeoutId);
  }
}

/**
 * Fetch current user from API with request-level caching.
 * Multiple calls in the same request reuse the cached result.
 */
const fetchMe = cache(async (): Promise<BominalUser | null> => {
  const cookieStore = await cookies();
  const cookieHeader = cookieStore.toString();
  if (!cookieHeader) {
    return null;
  }

  let response: Response;
  try {
    response = await fetchWithTimeout(`${serverApiBaseUrl}/api/auth/me`, {
      method: "GET",
      headers: {
        cookie: cookieHeader,
      },
      cache: "no-store",
    });
  } catch {
    // Auth backend may be unavailable/slow during startup; treat as unauthenticated.
    return null;
  }

  if (!response.ok) {
    return null;
  }

  try {
    const data = (await response.json()) as AuthMeResponse;
    return data.user;
  } catch {
    return null;
  }
});

/**
 * Get the current user if authenticated, or null if not.
 * Use for optional auth pages (e.g., landing pages with conditional UI).
 */
export async function getOptionalUser(): Promise<BominalUser | null> {
  return fetchMe();
}

/**
 * Require authentication. Redirects to /login if not authenticated.
 * Use for protected pages that require any authenticated user.
 */
export async function requireUser(): Promise<BominalUser> {
  const user = await fetchMe();
  if (!user) {
    redirect(ROUTES.login);
  }
  return user;
}

export function isApprovedUser(user: BominalUser): boolean {
  return String(user.access_status || "").toLowerCase() === "approved";
}

export function postLoginRouteForUser(user: BominalUser): string {
  return isApprovedUser(user) ? ROUTES.dashboard : ROUTES.applicationReview;
}

export async function requireApprovedUser(): Promise<BominalUser> {
  const user = await requireUser();
  if (!isApprovedUser(user)) {
    redirect(ROUTES.applicationReview);
  }
  return user;
}

export async function requirePendingReviewUser(): Promise<BominalUser> {
  const user = await requireUser();
  if (isApprovedUser(user)) {
    redirect(ROUTES.dashboard);
  }
  return user;
}

/**
 * Require admin role. Redirects to /dashboard if not admin.
 * Use for admin-only pages like maintenance dashboard.
 */
export async function requireAdminUser(): Promise<BominalUser> {
  const user = await requireApprovedUser();
  if (user.role !== "admin") {
    redirect(`${ROUTES.dashboard}?denied=1`);
  }
  return user;
}
