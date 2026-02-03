import "server-only";

import { cache } from "react";
import { cookies } from "next/headers";
import { redirect } from "next/navigation";

import { serverApiBaseUrl } from "@/lib/api-base";
import type { AuthMeResponse, BominalUser } from "@/lib/types";

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

export async function getOptionalUser(): Promise<BominalUser | null> {
  return fetchMe();
}

export async function requireUser(): Promise<BominalUser> {
  const user = await fetchMe();
  if (!user) {
    redirect("/login");
  }
  return user;
}

export async function requireAdminUser(): Promise<BominalUser> {
  const user = await requireUser();
  if (user.role !== "admin") {
    redirect("/dashboard?denied=1");
  }
  return user;
}
