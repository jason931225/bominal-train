import Script from "next/script";

const initThemeScript = `
(function() {
  try {
    var key = "bominal_theme_mode";
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
    var href = "/favicons/seasonal/bominal_" + season + ".ico";
    var icon = document.getElementById("bominal-theme-icon");
    if (!(icon instanceof HTMLLinkElement)) {
      icon = document.createElement("link");
      icon.id = "bominal-theme-icon";
      icon.rel = "icon";
      icon.type = "image/x-icon";
      document.head.appendChild(icon);
    }
    icon.setAttribute("href", href);

    var shortcut = document.getElementById("bominal-theme-shortcut-icon");
    if (!(shortcut instanceof HTMLLinkElement)) {
      shortcut = document.createElement("link");
      shortcut.id = "bominal-theme-shortcut-icon";
      shortcut.rel = "shortcut icon";
      shortcut.type = "image/x-icon";
      document.head.appendChild(shortcut);
    }
    shortcut.setAttribute("href", href);

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
