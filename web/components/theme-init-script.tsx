import Script from "next/script";

import { THEME_COOKIE_MAX_AGE_SECONDS, THEME_RESOLVED_COOKIE_KEY, THEME_STORAGE_KEY } from "@/lib/theme";

const initThemeScript = `
(function() {
  try {
    var key = "${THEME_STORAGE_KEY}";
    var resolvedKey = "${THEME_RESOLVED_COOKIE_KEY}";
    var cookieMaxAge = ${THEME_COOKIE_MAX_AGE_SECONDS};
    var raw = window.localStorage.getItem(key);
    var mode = (raw === "auto" || raw === "spring" || raw === "summer" || raw === "autumn" || raw === "winter") ? raw : "auto";
    var month = new Date().getMonth() + 1;
    var season = mode;
    if (mode === "auto") {
      if (month >= 3 && month <= 5) season = "spring";
      else if (month >= 6 && month <= 8) season = "summer";
      else if (month >= 9 && month <= 11) season = "autumn";
      else season = "winter";
    }

    var root = document.documentElement;
    root.dataset.themeMode = mode;
    root.dataset.theme = season;
    document.cookie = key + "=" + mode + "; Path=/; Max-Age=" + cookieMaxAge + "; SameSite=Lax";
    document.cookie = resolvedKey + "=" + season + "; Path=/; Max-Age=" + cookieMaxAge + "; SameSite=Lax";

    window.requestAnimationFrame(function() {
      root.classList.add("theme-ready");
    });
  } catch (error) {
    // noop
  }
})();
`;

export function ThemeInitScript() {
  return <Script id="bominal-theme-init" strategy="beforeInteractive" dangerouslySetInnerHTML={{ __html: initThemeScript }} />;
}
