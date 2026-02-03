"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";

import { clientApiBaseUrl } from "@/lib/api-base";
import { UI_BUTTON_DANGER_SM, UI_BUTTON_OUTLINE } from "@/lib/ui";

type LogoutButtonProps = {
  variant?: "pill" | "menu";
};

export function LogoutButton({ variant = "pill" }: LogoutButtonProps) {
  const router = useRouter();
  const [submitting, setSubmitting] = useState(false);

  const handleLogout = async () => {
    setSubmitting(true);
    try {
      await fetch(`${clientApiBaseUrl}/api/auth/logout`, {
        method: "POST",
        credentials: "include",
      });
      router.push("/login");
      router.refresh();
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
      {submitting ? "Signing out..." : "Sign out"}
    </button>
  );
}
