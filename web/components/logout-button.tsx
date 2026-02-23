"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";

import { useLocale } from "@/components/locale-provider";
import { clientApiBaseUrl } from "@/lib/api-base";
import { ROUTES } from "@/lib/routes";
import { UI_BUTTON_DANGER_SM, UI_BUTTON_OUTLINE } from "@/lib/ui";

type LogoutButtonProps = {
  variant?: "pill" | "menu";
};

export function LogoutButton({ variant = "pill" }: LogoutButtonProps) {
  const router = useRouter();
  const [submitting, setSubmitting] = useState(false);
  const { t } = useLocale();

  const navigateAfterLogout = () => {
    if (typeof window !== "undefined") {
      // Force layout/user state reset after session teardown.
      window.location.assign(ROUTES.login);
      return;
    }
    router.push(ROUTES.login);
  };

  const handleLogout = async () => {
    setSubmitting(true);
    try {
      await fetch(`${clientApiBaseUrl}/api/auth/logout`, {
        method: "POST",
        credentials: "include",
      });
      navigateAfterLogout();
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <button
      type="button"
      onClick={handleLogout}
      disabled={submitting}
      className={
        variant === "menu"
          ? `w-full justify-start ${UI_BUTTON_DANGER_SM}`
          : `${UI_BUTTON_OUTLINE} text-blossom-700`
      }
    >
      {submitting ? t("auth.signingOut") : t("auth.signOut")}
    </button>
  );
}
