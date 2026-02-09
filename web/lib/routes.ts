export const ROUTES = {
  dashboard: "/dashboard",
  login: "/login",
  register: "/register",

  modules: {
    train: "/modules/train",
    restaurant: "/modules/restaurant",
    calendar: "/modules/calendar",
  },

  settings: {
    account: "/settings/account",
    payment: "/settings/payment",
  },

  admin: {
    root: "/admin",
    maintenance: "/admin/maintenance",
  },
} as const;

export function isPathPrefix(pathname: string, prefix: string): boolean {
  return pathname === prefix || pathname.startsWith(`${prefix}/`);
}
