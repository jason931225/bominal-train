import { describe, expect, it } from "vitest";

import { formatStationLabel } from "@/lib/train/stations-i18n";

describe("formatStationLabel", () => {
  it("returns Korean station name for ko locale", () => {
    expect(formatStationLabel("수서", "ko")).toBe("수서");
  });

  it("returns romanized English with Korean fallback label for en locale by default", () => {
    expect(formatStationLabel("수서", "en")).toBe("Suseo (수서)");
  });

  it("returns compact romanized English only for en locale when compact mode is requested", () => {
    expect(formatStationLabel("수서", "en", { compact: true })).toBe("Suseo");
  });

  it("falls back to Korean when no romanized mapping exists", () => {
    expect(formatStationLabel("임시역", "en")).toBe("임시역");
  });
});
