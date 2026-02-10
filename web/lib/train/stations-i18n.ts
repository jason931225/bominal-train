import type { Locale } from "@/lib/i18n";

// Keep this mapping explicit to avoid algorithmic romanization surprises.
// Keys are the canonical station names returned by the API (Hangul + existing punctuation).
export const ROMANIZED_STATION_NAME: Record<string, string> = {
  "경주": "Gyeongju",
  "곡성": "Gokseong",
  "공주": "Gongju",
  "광명": "Gwangmyeong",
  "광주송정": "Gwangju Songjeong",
  "구례구": "Gurye-gu",
  "김천(구미)": "Gimcheon (Gumi)",
  "나주": "Naju",
  "남원": "Namwon",
  "대전": "Daejeon",
  "동대구": "Dongdaegu",
  "동탄": "Dongtan",
  "마산": "Masan",
  "목포": "Mokpo",
  "밀양": "Miryang",
  "부산": "Busan",
  "서대구": "Seodaegu",
  "서울": "Seoul",
  "수서": "Suseo",
  "순천": "Suncheon",
  "여수EXPO": "Yeosu EXPO",
  "여천": "Yeocheon",
  "영등포": "Yeongdeungpo",
  "오송": "Osong",
  "용산": "Yongsan",
  "울산(통도사)": "Ulsan (Tongdosa)",
  "익산": "Iksan",
  "전주": "Jeonju",
  "정읍": "Jeongeup",
  "진영": "Jinyeong",
  "진주": "Jinju",
  "창원": "Changwon",
  "창원중앙": "Changwon Jungang",
  "천안": "Cheonan",
  "천안아산": "Cheonan-Asan",
  "청량리": "Cheongnyangni",
  "평택지제": "Pyeongtaek Jije",
  "포항": "Pohang"
};

export function formatStationLabel(nameKo: string, locale: Locale): string {
  if (locale === "ko") return nameKo;
  const romanized = ROMANIZED_STATION_NAME[nameKo];
  if (!romanized) return nameKo;
  return `${romanized} (${nameKo})`;
}

