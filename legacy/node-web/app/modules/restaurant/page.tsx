import { redirect } from "next/navigation";

import { ROUTES } from "@/lib/routes";
import { requireApprovedUser } from "@/lib/server-auth";

export default async function RestaurantModulePage() {
  await requireApprovedUser();
  redirect(ROUTES.modules.train);
}
