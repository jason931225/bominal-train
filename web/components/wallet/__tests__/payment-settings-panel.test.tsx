import React from "react";

import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { encryptPaymentFields } from "@/lib/evervault";

vi.mock("@/lib/evervault", () => ({
  encryptPaymentFields: vi.fn(),
}));

function jsonResponse(payload: unknown, status = 200): Response {
  return new Response(JSON.stringify(payload), {
    status,
    headers: { "Content-Type": "application/json" },
  });
}

describe("PaymentSettingsPanel", () => {
  const fetchMock = vi.fn<typeof fetch>();

  beforeEach(() => {
    vi.clearAllMocks();
    vi.stubGlobal("fetch", fetchMock);
  });

  afterEach(() => {
    vi.unstubAllGlobals();
    vi.unstubAllEnvs();
  });

  it("sends encrypted wallet payload in evervault mode", async () => {
    vi.stubEnv("NEXT_PUBLIC_EVERVAULT_TEAM_ID", "team_test_123");
    vi.stubEnv("NEXT_PUBLIC_EVERVAULT_APP_ID", "app_test_123");
    vi.resetModules();
    const { LocaleProvider } = await import("@/components/locale-provider");
    const { PaymentSettingsPanel } = await import("@/components/wallet/payment-settings-panel");

    let submittedBody: Record<string, unknown> | null = null;
    fetchMock.mockImplementation(async (input: RequestInfo | URL, init?: RequestInit) => {
      const url = String(input);
      const method = init?.method ?? "GET";

      if (url.includes("/api/wallet/payment-card") && method === "GET") {
        return jsonResponse({
          configured: false,
          card_masked: null,
          expiry_month: null,
          expiry_year: null,
          source: null,
          brand: null,
          updated_at: null,
          detail: null,
        });
      }
      if (url.includes("/api/wallet/payment-card") && method === "POST") {
        submittedBody = JSON.parse(String(init?.body ?? "{}")) as Record<string, unknown>;
        return jsonResponse({
          configured: true,
          card_masked: "**** **** **** 1234",
          expiry_month: null,
          expiry_year: null,
          source: "evervault",
          brand: "visa",
          updated_at: null,
          detail: null,
        });
      }
      return jsonResponse({ detail: "not found" }, 404);
    });

    vi.mocked(encryptPaymentFields).mockResolvedValue({
      encrypted_card_number: "ev:card",
      encrypted_pin2: "ev:pin2",
      encrypted_birth_date: "ev:birth",
      encrypted_expiry: "ev:expiry",
      last4: "1234",
      brand: "visa",
    });

    render(
      <LocaleProvider initialLocale="en">
        <PaymentSettingsPanel />
      </LocaleProvider>,
    );

    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledTimes(1);
    });

    fireEvent.change(screen.getByLabelText("Card number"), { target: { value: "4111 1111 1111 1234" } });
    fireEvent.change(screen.getByLabelText("Expiry month"), { target: { value: "12" } });
    fireEvent.change(screen.getByLabelText("Expiry year"), { target: { value: "2030" } });
    fireEvent.change(screen.getByLabelText("Date of birth"), { target: { value: "1990-01-01" } });
    fireEvent.change(screen.getByLabelText("Card PIN (first 2 digits)"), { target: { value: "12" } });
    fireEvent.click(screen.getByRole("button", { name: "Save payment settings" }));

    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledTimes(2);
    });
    expect(encryptPaymentFields).toHaveBeenCalledWith(
      {
        card_number: "4111111111111234",
        pin2: "12",
        birth_date: "900101",
        expiry: "3012",
        last4: "1234",
      },
      { teamId: "team_test_123", appId: "app_test_123" },
    );
    expect(submittedBody).toEqual({
      encrypted_card_number: "ev:card",
      encrypted_pin2: "ev:pin2",
      encrypted_birth_date: "ev:birth",
      encrypted_expiry: "ev:expiry",
      last4: "1234",
      brand: "visa",
    });
  });

  it("falls back to legacy payload when evervault browser env is missing", async () => {
    vi.stubEnv("NEXT_PUBLIC_EVERVAULT_TEAM_ID", "");
    vi.stubEnv("NEXT_PUBLIC_EVERVAULT_APP_ID", "");
    vi.resetModules();
    const { LocaleProvider } = await import("@/components/locale-provider");
    const { PaymentSettingsPanel } = await import("@/components/wallet/payment-settings-panel");

    let submittedBody: Record<string, unknown> | null = null;
    fetchMock.mockImplementation(async (input: RequestInfo | URL, init?: RequestInit) => {
      const url = String(input);
      const method = init?.method ?? "GET";
      if (url.includes("/api/wallet/payment-card") && method === "GET") {
        return jsonResponse({
          configured: false,
          card_masked: null,
          expiry_month: null,
          expiry_year: null,
          source: null,
          brand: null,
          updated_at: null,
          detail: null,
        });
      }
      if (url.includes("/api/wallet/payment-card") && method === "POST") {
        submittedBody = JSON.parse(String(init?.body ?? "{}")) as Record<string, unknown>;
        return jsonResponse({
          configured: true,
          card_masked: "**** **** **** 1234",
          expiry_month: 12,
          expiry_year: 2030,
          source: "legacy",
          brand: null,
          updated_at: null,
          detail: null,
        });
      }
      return jsonResponse({ detail: "not found" }, 404);
    });

    render(
      <LocaleProvider initialLocale="en">
        <PaymentSettingsPanel />
      </LocaleProvider>,
    );

    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledTimes(1);
    });

    fireEvent.change(screen.getByLabelText("Card number"), { target: { value: "4111 1111 1111 1234" } });
    fireEvent.change(screen.getByLabelText("Expiry month"), { target: { value: "12" } });
    fireEvent.change(screen.getByLabelText("Expiry year"), { target: { value: "2030" } });
    fireEvent.change(screen.getByLabelText("Date of birth"), { target: { value: "1990-01-01" } });
    fireEvent.change(screen.getByLabelText("Card PIN (first 2 digits)"), { target: { value: "12" } });
    fireEvent.click(screen.getByRole("button", { name: "Save payment settings" }));

    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledTimes(2);
    });
    expect(encryptPaymentFields).not.toHaveBeenCalled();
    expect(submittedBody).toEqual({
      card_number: "4111111111111234",
      expiry_month: 12,
      expiry_year: 2030,
      birth_date: "1990-01-01",
      pin2: "12",
    });
  });
});
