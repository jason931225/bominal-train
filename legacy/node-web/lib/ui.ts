export const UI_CARD_BASE = "rounded-3xl border border-blossom-100 bg-white/95 shadow-petal";
export const UI_CARD_LG = `${UI_CARD_BASE} p-8`;
export const UI_CARD_MD = `${UI_CARD_BASE} p-6`;
export const UI_CARD_SM = `${UI_CARD_BASE} p-4`;

export const UI_LIQUID_GLASS_WHITE =
  "rounded-3xl border border-white/20 bg-gradient-to-b from-white/20 to-white/5 backdrop-blur-xl shadow-[0_12px_30px_-18px_rgba(15,23,42,0.55)]";
export const UI_LIQUID_GLASS_TEXT_WHITE =
  // Mobile Safari can be finicky with `bg-clip-text` + transparent text. Keep a
  // readable solid fallback on small screens and upgrade to gradient text on sm+.
  "text-white drop-shadow-[0_18px_50px_rgba(0,0,0,0.45)] sm:text-transparent sm:bg-clip-text sm:bg-gradient-to-b sm:from-white/95 sm:via-white/75 sm:to-white/45";

export const UI_KICKER = "text-xs uppercase tracking-[0.16em] text-blossom-500";
export const UI_TITLE_LG = "text-3xl font-display font-semibold tracking-tight text-slate-900";
export const UI_TITLE_MD = "text-2xl font-display font-semibold tracking-tight text-slate-900";
export const UI_BODY_MUTED = "text-sm text-slate-500";

export const UI_BUTTON_BASE =
  "inline-flex items-center justify-center rounded-full font-medium transition focus:outline-none focus:ring-2 disabled:cursor-not-allowed disabled:opacity-60";
export const UI_BUTTON_PRIMARY = `${UI_BUTTON_BASE} h-10 bg-blossom-500 px-4 text-sm text-white hover:bg-blossom-600 focus:ring-blossom-200`;
export const UI_BUTTON_OUTLINE = `${UI_BUTTON_BASE} h-10 border border-blossom-200 bg-white px-4 text-sm text-slate-700 hover:bg-blossom-50 focus:ring-blossom-100`;
export const UI_BUTTON_DANGER = `${UI_BUTTON_BASE} h-10 border border-rose-200 bg-white px-4 text-sm text-rose-700 hover:bg-rose-50 focus:ring-rose-100`;
export const UI_BUTTON_OUTLINE_TOUCH = `${UI_BUTTON_BASE} h-11 border border-blossom-200 bg-white px-4 text-sm text-slate-700 hover:bg-blossom-50 focus:ring-blossom-100 sm:h-10`;
export const UI_BUTTON_OUTLINE_SM = `${UI_BUTTON_BASE} h-11 border border-blossom-200 bg-white px-3 text-sm text-slate-700 hover:bg-blossom-50 focus:ring-blossom-100 sm:h-8 sm:px-2.5 sm:text-xs`;
export const UI_BUTTON_DANGER_SM = `${UI_BUTTON_BASE} h-11 border border-rose-200 bg-white px-3 text-sm text-rose-700 hover:bg-rose-50 focus:ring-rose-100 sm:h-8 sm:px-2.5 sm:text-xs`;

export const UI_FIELD =
  "h-10 w-full rounded-2xl border border-blossom-200 bg-white px-3 text-sm text-slate-700 shadow-sm outline-none transition focus:border-blossom-300 focus:ring-2 focus:ring-blossom-100";

export const UI_MENU_ITEM = "block rounded-xl px-3 py-2 text-sm text-slate-700 transition hover:bg-blossom-50";

export const UI_CHIP_BASE = "rounded-full px-2 py-1 text-xs font-medium tracking-wide";
export const UI_CHIP_BRAND = `${UI_CHIP_BASE} border border-blossom-200 bg-blossom-50 text-blossom-600`;
export const UI_CHIP_MUTED = `${UI_CHIP_BASE} border border-slate-200 bg-slate-100 text-slate-500`;
