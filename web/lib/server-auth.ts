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
import type { AuthMeResponse, BominalUser } from "@/lib/types";

/**
 * Fetch current user from API with request-level caching.
 * Multiple calls in the same request reuse the cached result.
 */
const fetchMe = cache(async (): Promise<BominalUser | null> => {
  const cookieHeader = cookies().toString();
  if (!cookieHeader) {
    return null;
  }

  const response = await fetch(`${serverApiBaseUrl}/api/auth/me`, {
    method: "GET",
    headers: {
      cookie: cookieHeader,
    },
    cache: "no-store",
  });

  if (!response.ok) {
    return null;
  }

  const data = (await response.json()) as AuthMeResponse;
  return data.user;
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
    redirect("/login");
  }
  return user;
}

/**
 * Require admin role. Redirects to /dashboard if not admin.
 * Use for admin-only pages like maintenance dashboard.
 */
export async function requireAdminUser(): Promise<BominalUser> {
  const user = await requireUser();
  if (user.role !== "admin") {
    redirect("/dashboard?denied=1");
  }
  return user;
}
