export const ROUTES = {
  applicationReview: "/application-review",
  dashboard: "/dashboard",
  login: "/login",
  register: "/register",
  forgotPassword: "/forgot-password",
  resetPassword: "/reset-password",
  authCallback: "/auth/callback",

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
