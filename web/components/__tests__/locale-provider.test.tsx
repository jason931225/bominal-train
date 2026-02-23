import React from "react";

import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { LocaleProvider, useLocale } from "@/components/locale-provider";

function LocaleProbe() {
  const { locale, setLocale, t } = useLocale();
  return (
    <div>
      <div data-testid="locale">{locale}</div>
      <div data-testid="dashboard-label">{t("nav.dashboard")}</div>
      <button onClick={() => setLocale(locale === "en" ? "ko" : "en")}>toggle-locale</button>
    </div>
  );
}

describe("LocaleProvider", () => {
  it("throws when useLocale is used outside provider", () => {
    const renderOutside = () => render(<LocaleProbe />);
    expect(renderOutside).toThrow("useLocale must be used within LocaleProvider");
  });

  it("provides locale context and updates from both setLocale and initialLocale changes", async () => {
    const { rerender } = render(
      <LocaleProvider initialLocale="en">
        <LocaleProbe />
      </LocaleProvider>,
    );

    expect(screen.getByTestId("locale")).toHaveTextContent("en");
    expect(screen.getByTestId("dashboard-label")).toHaveTextContent("Dashboard");

    fireEvent.click(screen.getByRole("button", { name: "toggle-locale" }));
    expect(screen.getByTestId("locale")).toHaveTextContent("ko");
    expect(screen.getByTestId("dashboard-label")).toHaveTextContent("대시보드");

    rerender(
      <LocaleProvider initialLocale="ko">
        <LocaleProbe />
      </LocaleProvider>,
    );
    expect(screen.getByTestId("locale")).toHaveTextContent("ko");
    expect(screen.getByTestId("dashboard-label")).toHaveTextContent("대시보드");

    rerender(
      <LocaleProvider initialLocale="en">
        <LocaleProbe />
      </LocaleProvider>,
    );
    await waitFor(() => {
      expect(screen.getByTestId("locale")).toHaveTextContent("en");
      expect(screen.getByTestId("dashboard-label")).toHaveTextContent("Dashboard");
    });
  });
});
