import { redirect } from "next/navigation";

import { ROUTES } from "@/lib/routes";
import { requireApprovedUser } from "@/lib/server-auth";

export default async function CalendarModulePage() {
  await requireApprovedUser();
  redirect(ROUTES.modules.train);
}
