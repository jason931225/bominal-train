import { describe, expect, it } from "vitest";

import type { TrainStation } from "@/lib/types";
import {
  convertDubeolsikLatinToHangul,
  rankStationCandidates,
  shouldAutoCommitTopSuggestion,
} from "@/lib/train/station-search";

const STATIONS: TrainStation[] = [
  { name: "수서", srt_code: "0551", srt_supported: true },
  { name: "부산", srt_code: "0020", srt_supported: true },
  { name: "동탄", srt_code: "0552", srt_supported: true },
  { name: "서울", srt_code: null, srt_supported: false },
  { name: "대전", srt_code: null, srt_supported: false },
];

describe("station search helpers", () => {
  it("converts dubeolsik latin input to hangul", () => {
    expect(convertDubeolsikLatinToHangul("tntj")).toBe("수서");
    expect(convertDubeolsikLatinToHangul("ehdxks")).toBe("동탄");
  });

  it("ranks korean and english station queries with top-3 cap", () => {
    const koMatches = rankStationCandidates("수서", STATIONS, { locale: "ko", limit: 3 });
    expect(koMatches[0]?.station.name).toBe("수서");
    expect(koMatches.length).toBeLessThanOrEqual(3);

    const enMatches = rankStationCandidates("suseo", STATIONS, { locale: "en", limit: 3 });
    expect(enMatches[0]?.station.name).toBe("수서");
    expect(enMatches[0]?.primaryLabel).toContain("Suseo");
    expect(enMatches.length).toBeLessThanOrEqual(3);
  });

  it("uses keyboard-layout conversion as hidden ranking signal", () => {
    const matches = rankStationCandidates("tntj", STATIONS, { locale: "ko", limit: 3 });
    expect(matches[0]?.station.name).toBe("수서");
  });

  it("returns no matches for nonsensical input", () => {
    const matches = rankStationCandidates("zzzzqqq", STATIONS, { locale: "en", limit: 3 });
    expect(matches).toHaveLength(0);
  });

  it("autocommit policy accepts strong matches and rejects weak fuzzy candidates", () => {
    const strong = rankStationCandidates("수서", STATIONS, { locale: "ko", limit: 3 });
    expect(shouldAutoCommitTopSuggestion(strong, "수서")).toBe(true);

    const weak = rankStationCandidates("zzzzqqq", STATIONS, { locale: "en", limit: 3 });
    expect(shouldAutoCommitTopSuggestion(weak, "zzzzqqq")).toBe(false);
  });
});
