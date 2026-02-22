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
});
