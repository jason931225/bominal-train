import React from "react";

import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { ModuleTile } from "@/components/module-tile";
import type { BominalModule } from "@/lib/types";

const TRAIN_MODULE: BominalModule = {
  slug: "train",
  name: "Train",
  coming_soon: false,
  enabled: true,
  capabilities: [
    "train.search",
    "train.tasks.create",
    "train.tasks.control",
    "train.credentials.manage",
    "train.tickets.manage",
    "wallet.payment_card",
  ],
};

describe("ModuleTile provider flags", () => {
  it("renders KTX/SRT provider badges for the train module", () => {
    render(<ModuleTile module={TRAIN_MODULE} locale="en" />);

    expect(screen.getByText("KTX")).toBeInTheDocument();
    expect(screen.getByText("SRT")).toBeInTheDocument();

    expect(screen.queryByText("Search")).not.toBeInTheDocument();
    expect(screen.queryByText("Create Tasks")).not.toBeInTheDocument();
  });

  it("keeps provider badges hidden on mobile breakpoints and shows compact summary", () => {
    render(<ModuleTile module={TRAIN_MODULE} locale="en" />);

    const badge = screen.getByText("KTX");
    const container = badge.parentElement;
    const summary = screen.getByText("KTX · SRT");

    expect(container).not.toBeNull();
    expect(container?.className).toContain("hidden");
    expect(container?.className).toContain("md:flex");
    expect(summary.className).toContain("md:hidden");
  });

  it("renders capability-derived badges for enabled custom modules", () => {
    const customModule: BominalModule = {
      slug: "custom",
      name: "Custom",
      coming_soon: false,
      enabled: true,
      capabilities: ["train.search", "custom.sync_jobs", "custom-export"],
    };
    render(<ModuleTile module={customModule} locale="en" />);

    expect(screen.getByText("Search")).toBeInTheDocument();
    expect(screen.getByText("Sync Jobs")).toBeInTheDocument();
    expect(screen.getByText("Custom Export")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: /Custom/i })).toHaveAttribute("href", "/modules/custom");
  }, 20_000);

  it("renders muted fallback badges for restaurant and calendar modules", () => {
    const restaurantModule: BominalModule = {
      slug: "restaurant",
      name: "Restaurant",
      coming_soon: true,
      enabled: false,
      capabilities: [],
    };
    const calendarModule: BominalModule = {
      slug: "calendar",
      name: "Calendar",
      coming_soon: true,
      enabled: false,
      capabilities: [],
    };

    const { rerender } = render(<ModuleTile module={restaurantModule} locale="en" />);
    expect(screen.getByText("Catchtable")).toBeInTheDocument();
    expect(screen.getByText("Resy")).toBeInTheDocument();
    expect(screen.getByText("OpenTable")).toBeInTheDocument();
    expect(screen.getByText("Coming soon")).toBeInTheDocument();

    rerender(<ModuleTile module={calendarModule} locale="en" />);
    expect(screen.getAllByText("None")).toHaveLength(2);
  });

  it("falls back to module slug badge when there are no provider badges", () => {
    const bareModule: BominalModule = {
      slug: "ops",
      name: "Ops",
      coming_soon: false,
      enabled: true,
      capabilities: [],
    };

    render(<ModuleTile module={bareModule} locale="en" />);
    expect(screen.getByText("ops")).toBeInTheDocument();
  });

  it("handles capability labels with empty tail fallback", () => {
    const edgeModule: BominalModule = {
      slug: "edge",
      name: "Edge",
      coming_soon: false,
      enabled: true,
      capabilities: ["custom."],
    };

    render(<ModuleTile module={edgeModule} locale="en" />);
    expect(screen.getAllByText("Custom.")).toHaveLength(2);
  });
});
