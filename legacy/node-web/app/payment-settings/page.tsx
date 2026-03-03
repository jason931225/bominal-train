import { redirect } from "next/navigation";

import { ROUTES } from "@/lib/routes";

export default async function PaymentSettingsPage() {
  redirect(ROUTES.settings.payment);
}
