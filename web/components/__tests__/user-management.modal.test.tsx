import React from "react";

import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { UserManagement } from "@/components/admin/user-management";
import { LocaleProvider } from "@/components/locale-provider";

type JsonLike = Record<string, unknown>;

function jsonResponse(payload: JsonLike): Response {
  return new Response(JSON.stringify(payload), {
    status: 200,
    headers: { "Content-Type": "application/json" },
  });
}

describe("UserManagement modal overlay", () => {
  const fetchMock = vi.fn<typeof fetch>();

  beforeEach(() => {
    vi.clearAllMocks();
    vi.stubGlobal("fetch", fetchMock);
    fetchMock.mockImplementation((input) => {
      const url = String(input);
      if (url.includes("/api/admin/users?") && url.includes("access_status=pending")) {
        return Promise.resolve(
          jsonResponse({
            users: [],
            total: 0,
            page: 1,
            page_size: 25,
          }),
        );
      }
      if (url.includes("/api/admin/users?")) {
        return Promise.resolve(
          jsonResponse({
            users: [
              {
                id: "u1",
                email: "user@example.com",
                display_name: "User",
                role: "user",
                access_status: "approved",
                access_reviewed_at: null,
                created_at: "2026-02-26T00:00:00Z",
                last_seen_at: null,
                session_count: 1,
                task_count: 0,
              },
            ],
            total: 1,
            page: 1,
            page_size: 10,
          }),
        );
      }
      if (url.endsWith("/api/admin/users/u1")) {
        return Promise.resolve(
          jsonResponse({
            id: "u1",
            email: "user@example.com",
            display_name: "User",
            phone_number: null,
            role: "user",
            access_status: "approved",
            access_reviewed_at: null,
            created_at: "2026-02-26T00:00:00Z",
            updated_at: "2026-02-26T00:00:00Z",
            email_verified_at: null,
            session_count: 1,
            active_session_count: 1,
            task_count: 0,
            secret_count: 0,
          }),
        );
      }
      return Promise.reject(new Error(`Unexpected request: ${url}`));
    });
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("renders the user detail overlay outside the component container", async () => {
    const { container } = render(
      <LocaleProvider initialLocale="en">
        <UserManagement />
      </LocaleProvider>,
    );

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "View" })).toBeInTheDocument();
    });

    fireEvent.click(screen.getByRole("button", { name: "View" }));

    await waitFor(() => {
      expect(screen.getByText("User details")).toBeInTheDocument();
    });

    const overlay = document.querySelector("div[class*='bg-black/40']");
    expect(overlay).not.toBeNull();
    expect(container.contains(overlay)).toBe(false);
  });
});
