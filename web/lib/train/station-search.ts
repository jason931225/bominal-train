import type { Locale } from "@/lib/i18n";
import type { TrainStation } from "@/lib/types";
import { formatStationLabel, ROMANIZED_STATION_NAME } from "@/lib/train/stations-i18n";

export type StationSearchMatch = {
  station: TrainStation;
  score: number;
  primaryLabel: string;
  secondaryLabel: string | null;
};

const HANGUL_RANGE = /[\uac00-\ud7a3]/;
const CONSONANTS = new Set([
  "ㄱ",
  "ㄲ",
  "ㄴ",
  "ㄷ",
  "ㄸ",
  "ㄹ",
  "ㅁ",
  "ㅂ",
  "ㅃ",
  "ㅅ",
  "ㅆ",
  "ㅇ",
  "ㅈ",
  "ㅉ",
  "ㅊ",
  "ㅋ",
  "ㅌ",
  "ㅍ",
  "ㅎ",
]);
const VOWELS = new Set([
  "ㅏ",
  "ㅐ",
  "ㅑ",
  "ㅒ",
  "ㅓ",
  "ㅔ",
  "ㅕ",
  "ㅖ",
  "ㅗ",
  "ㅘ",
  "ㅙ",
  "ㅚ",
  "ㅛ",
  "ㅜ",
  "ㅝ",
  "ㅞ",
  "ㅟ",
  "ㅠ",
  "ㅡ",
  "ㅢ",
  "ㅣ",
]);

const KEY_TO_JAMO: Record<string, string> = {
  r: "ㄱ",
  R: "ㄲ",
  s: "ㄴ",
  e: "ㄷ",
  E: "ㄸ",
  f: "ㄹ",
  a: "ㅁ",
  q: "ㅂ",
  Q: "ㅃ",
  t: "ㅅ",
  T: "ㅆ",
  d: "ㅇ",
  w: "ㅈ",
  W: "ㅉ",
  c: "ㅊ",
  z: "ㅋ",
  x: "ㅌ",
  v: "ㅍ",
  g: "ㅎ",
  k: "ㅏ",
  o: "ㅐ",
  O: "ㅒ",
  i: "ㅑ",
  j: "ㅓ",
  p: "ㅔ",
  P: "ㅖ",
  u: "ㅕ",
  h: "ㅗ",
  y: "ㅛ",
  n: "ㅜ",
  b: "ㅠ",
  m: "ㅡ",
  l: "ㅣ",
};

const COMBINE_VOWEL: Record<string, string> = {
  "ㅗㅏ": "ㅘ",
  "ㅗㅐ": "ㅙ",
  "ㅗㅣ": "ㅚ",
  "ㅜㅓ": "ㅝ",
  "ㅜㅔ": "ㅞ",
  "ㅜㅣ": "ㅟ",
  "ㅡㅣ": "ㅢ",
};

const COMBINE_FINAL: Record<string, string> = {
  "ㄱㅅ": "ㄳ",
  "ㄴㅈ": "ㄵ",
  "ㄴㅎ": "ㄶ",
  "ㄹㄱ": "ㄺ",
  "ㄹㅁ": "ㄻ",
  "ㄹㅂ": "ㄼ",
  "ㄹㅅ": "ㄽ",
  "ㄹㅌ": "ㄾ",
  "ㄹㅍ": "ㄿ",
  "ㄹㅎ": "ㅀ",
  "ㅂㅅ": "ㅄ",
};

const CHOSEONG = [
  "ㄱ",
  "ㄲ",
  "ㄴ",
  "ㄷ",
  "ㄸ",
  "ㄹ",
  "ㅁ",
  "ㅂ",
  "ㅃ",
  "ㅅ",
  "ㅆ",
  "ㅇ",
  "ㅈ",
  "ㅉ",
  "ㅊ",
  "ㅋ",
  "ㅌ",
  "ㅍ",
  "ㅎ",
];
const JUNGSEONG = [
  "ㅏ",
  "ㅐ",
  "ㅑ",
  "ㅒ",
  "ㅓ",
  "ㅔ",
  "ㅕ",
  "ㅖ",
  "ㅗ",
  "ㅘ",
  "ㅙ",
  "ㅚ",
  "ㅛ",
  "ㅜ",
  "ㅝ",
  "ㅞ",
  "ㅟ",
  "ㅠ",
  "ㅡ",
  "ㅢ",
  "ㅣ",
];
const JONGSEONG = [
  "",
  "ㄱ",
  "ㄲ",
  "ㄳ",
  "ㄴ",
  "ㄵ",
  "ㄶ",
  "ㄷ",
  "ㄹ",
  "ㄺ",
  "ㄻ",
  "ㄼ",
  "ㄽ",
  "ㄾ",
  "ㄿ",
  "ㅀ",
  "ㅁ",
  "ㅂ",
  "ㅄ",
  "ㅅ",
  "ㅆ",
  "ㅇ",
  "ㅈ",
  "ㅊ",
  "ㅋ",
  "ㅌ",
  "ㅍ",
  "ㅎ",
];

const KEY_ADJACENCY: Record<string, string[]> = {
  a: ["q", "w", "s", "z"],
  b: ["v", "g", "h", "n"],
  c: ["x", "d", "f", "v"],
  d: ["s", "e", "r", "f", "c", "x"],
  e: ["w", "s", "d", "r"],
  f: ["d", "r", "t", "g", "v", "c"],
  g: ["f", "t", "y", "h", "b", "v"],
  h: ["g", "y", "u", "j", "n", "b"],
  i: ["u", "j", "k", "o"],
  j: ["h", "u", "i", "k", "m", "n"],
  k: ["j", "i", "o", "l", "m"],
  l: ["k", "o", "p"],
  m: ["n", "j", "k"],
  n: ["b", "h", "j", "m"],
  o: ["i", "k", "l", "p"],
  p: ["o", "l"],
  q: ["w", "a"],
  r: ["e", "d", "f", "t"],
  s: ["a", "w", "e", "d", "x", "z"],
  t: ["r", "f", "g", "y"],
  u: ["y", "h", "j", "i"],
  v: ["c", "f", "g", "b"],
  w: ["q", "a", "s", "e"],
  x: ["z", "s", "d", "c"],
  y: ["t", "g", "h", "u"],
  z: ["a", "s", "x"],
};

function normalizeText(input: string): string {
  return input
    .trim()
    .toLowerCase()
    .normalize("NFKC")
    .replace(/[^\p{L}\p{N}]/gu, "");
}

function hasHangul(input: string): boolean {
  return HANGUL_RANGE.test(input);
}

function composeHangul(lead: string, vowel: string, tail = ""): string {
  const leadIndex = CHOSEONG.indexOf(lead);
  const vowelIndex = JUNGSEONG.indexOf(vowel);
  const tailIndex = JONGSEONG.indexOf(tail);
  if (leadIndex < 0 || vowelIndex < 0 || tailIndex < 0) {
    return `${lead}${vowel}${tail}`;
  }
  const code = 0xac00 + (leadIndex * 21 + vowelIndex) * 28 + tailIndex;
  return String.fromCharCode(code);
}

function convertJamoSegmentToHangul(jamoSequence: string[]): string {
  let out = "";
  let i = 0;

  while (i < jamoSequence.length) {
    const lead = jamoSequence[i];

    if (!CONSONANTS.has(lead) || i + 1 >= jamoSequence.length || !VOWELS.has(jamoSequence[i + 1])) {
      out += lead;
      i += 1;
      continue;
    }

    i += 1;
    let vowel = jamoSequence[i];
    if (i + 1 < jamoSequence.length && VOWELS.has(jamoSequence[i + 1])) {
      const combinedVowel = COMBINE_VOWEL[`${vowel}${jamoSequence[i + 1]}`];
      if (combinedVowel) {
        vowel = combinedVowel;
        i += 1;
      }
    }
    i += 1;

    let tail = "";
    if (i < jamoSequence.length && CONSONANTS.has(jamoSequence[i])) {
      const firstTail = jamoSequence[i];
      const hasNextVowel = i + 1 < jamoSequence.length && VOWELS.has(jamoSequence[i + 1]);
      if (!hasNextVowel) {
        const maybeSecondTail = i + 1 < jamoSequence.length ? jamoSequence[i + 1] : "";
        const canComposeDoubleTail =
          maybeSecondTail &&
          CONSONANTS.has(maybeSecondTail) &&
          COMBINE_FINAL[`${firstTail}${maybeSecondTail}`] &&
          !(i + 2 < jamoSequence.length && VOWELS.has(jamoSequence[i + 2]));

        if (canComposeDoubleTail) {
          tail = COMBINE_FINAL[`${firstTail}${maybeSecondTail}`];
          i += 2;
        } else {
          tail = firstTail;
          i += 1;
        }
      }
    }

    out += composeHangul(lead, vowel, tail);
  }

  return out;
}

export function convertDubeolsikLatinToHangul(input: string): string {
  if (!input) return "";
  const chars = Array.from(input);
  const outputParts: string[] = [];
  let pendingJamo: string[] = [];

  const flushPending = () => {
    if (pendingJamo.length === 0) return;
    outputParts.push(convertJamoSegmentToHangul(pendingJamo));
    pendingJamo = [];
  };

  for (const ch of chars) {
    const mapped = KEY_TO_JAMO[ch];
    if (!mapped) {
      flushPending();
      outputParts.push(ch);
      continue;
    }
    pendingJamo.push(mapped);
  }

  flushPending();
  return outputParts.join("");
}

function keyboardDistance(a: string, b: string): number {
  const left = normalizeText(a);
  const right = normalizeText(b);
  if (!left || !right) return Number.POSITIVE_INFINITY;

  const rows = left.length + 1;
  const cols = right.length + 1;
  const dp: number[][] = Array.from({ length: rows }, () => Array(cols).fill(0));

  for (let i = 0; i < rows; i += 1) dp[i][0] = i;
  for (let j = 0; j < cols; j += 1) dp[0][j] = j;

  for (let i = 1; i < rows; i += 1) {
    for (let j = 1; j < cols; j += 1) {
      const l = left[i - 1];
      const r = right[j - 1];
      let substitution = l === r ? 0 : 1;
      if (substitution > 0 && KEY_ADJACENCY[l]?.includes(r)) {
        substitution = 0.45;
      }
      dp[i][j] = Math.min(
        dp[i - 1][j] + 1,
        dp[i][j - 1] + 1,
        dp[i - 1][j - 1] + substitution,
      );
    }
  }

  return dp[rows - 1][cols - 1];
}

function scoreLabel(query: string, label: string): number {
  if (!query || !label) return 0;

  const normalizedQuery = normalizeText(query);
  const normalizedLabel = normalizeText(label);
  if (!normalizedQuery || !normalizedLabel) return 0;

  if (normalizedQuery === normalizedLabel) return 1;
  if (normalizedLabel.startsWith(normalizedQuery)) {
    return Math.max(0.72, 0.95 - normalizedQuery.length * 0.01);
  }
  if (normalizedLabel.includes(normalizedQuery)) {
    return Math.max(0.55, 0.82 - normalizedQuery.length * 0.01);
  }

  const distance = keyboardDistance(normalizedQuery, normalizedLabel);
  const maxDistance = normalizedQuery.length <= 4 ? 1.2 : normalizedQuery.length <= 7 ? 1.8 : 2.2;
  if (distance > maxDistance) return 0;

  return 0.56 - distance * 0.08;
}

function hasExactNormalizedMatch(query: string, station: TrainStation): boolean {
  const normalizedQuery = normalizeText(query);
  if (!normalizedQuery) return false;
  const normalizedNameKo = normalizeText(station.name);
  const romanized = ROMANIZED_STATION_NAME[station.name] ?? station.name;
  const normalizedNameEn = normalizeText(romanized);
  if (normalizedQuery === normalizedNameKo || normalizedQuery === normalizedNameEn) {
    return true;
  }
  const converted = normalizeText(convertDubeolsikLatinToHangul(query));
  return converted.length > 0 && converted === normalizedNameKo;
}

export function rankStationCandidates(
  query: string,
  stations: TrainStation[],
  options?: { locale?: Locale; limit?: number },
): StationSearchMatch[] {
  const locale = options?.locale ?? "ko";
  const limit = Math.max(1, options?.limit ?? 3);

  if (stations.length < 1) {
    return [];
  }

  const trimmed = query.trim();
  if (!trimmed) {
    return stations.slice(0, limit).map((station, index) => ({
      station,
      score: 0.2 - index * 0.01,
      primaryLabel: formatStationLabel(station.name, locale),
      secondaryLabel: locale === "ko" ? ROMANIZED_STATION_NAME[station.name] ?? null : station.name,
    }));
  }

  const convertedQuery = convertDubeolsikLatinToHangul(query);
  const queryLooksHangul = hasHangul(trimmed);

  const ranked: StationSearchMatch[] = [];

  for (const station of stations) {
    const nameKo = station.name;
    const nameEn = ROMANIZED_STATION_NAME[station.name] ?? station.name;

    const scoreKo = scoreLabel(query, nameKo);
    const scoreEn = scoreLabel(query, nameEn);
    const scoreConverted = convertedQuery === query ? 0 : scoreLabel(convertedQuery, nameKo);
    const bestCoreScore = Math.max(scoreKo, scoreEn, scoreConverted);
    if (bestCoreScore <= 0) continue;

    let score = bestCoreScore;
    if (!queryLooksHangul && scoreConverted > 0) score += 0.06;
    if (locale === "ko" && scoreKo >= scoreEn) score += 0.03;
    if (locale === "en" && scoreEn >= scoreKo) score += 0.03;

    ranked.push({
      station,
      score,
      primaryLabel: formatStationLabel(station.name, locale),
      secondaryLabel: locale === "ko" ? nameEn : station.name,
    });
  }

  ranked.sort((a, b) => {
    if (b.score !== a.score) {
      return b.score - a.score;
    }
    return a.station.name.localeCompare(b.station.name, "ko");
  });

  return ranked.slice(0, limit);
}

export function shouldAutoCommitTopSuggestion(matches: StationSearchMatch[], query: string): boolean {
  const normalizedQuery = normalizeText(query);
  if (!normalizedQuery || matches.length < 1) {
    return false;
  }

  const top = matches[0];
  if (!top) return false;
  if (hasExactNormalizedMatch(query, top.station)) {
    return true;
  }

  const runnerUpScore = matches[1]?.score ?? 0;
  const lead = top.score - runnerUpScore;
  return top.score >= 0.98 && lead >= 0.08;
}
