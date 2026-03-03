const KST_TIME_ZONE = "Asia/Seoul";
const DEFAULT_LOCALE = "ko-KR";

function intlLocale(locale?: string): string {
  if (locale === "en") return "en-US";
  if (locale === "ko") return "ko-KR";
  return DEFAULT_LOCALE;
}

function datePartsInKst(date: Date): { year: string; month: string; day: string } {
  const parts = new Intl.DateTimeFormat("en-CA", {
    timeZone: KST_TIME_ZONE,
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  }).formatToParts(date);

  const year = parts.find((part) => part.type === "year")?.value;
  const month = parts.find((part) => part.type === "month")?.value;
  const day = parts.find((part) => part.type === "day")?.value;

  if (!year || !month || !day) {
    throw new Error("Could not format KST date parts.");
  }

  return { year, month, day };
}

export function kstDateInputValue(date: Date = new Date()): string {
  const { year, month, day } = datePartsInKst(date);
  return `${year}-${month}-${day}`;
}

export function formatDateTimeKst(value: string | Date | null, locale?: string): string {
  if (!value) return "-";

  const date = value instanceof Date ? value : new Date(value);
  if (Number.isNaN(date.getTime())) return "-";

  const formatted = new Intl.DateTimeFormat(intlLocale(locale), {
    timeZone: KST_TIME_ZONE,
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  }).format(date);

  return `${formatted} KST`;
}
