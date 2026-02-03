import { cookies } from "next/headers";

import { serverApiBaseUrl } from "@/lib/api-base";
import { UI_CARD_LG, UI_KICKER, UI_TITLE_LG } from "@/lib/ui";
import { requireAdminUser } from "@/lib/server-auth";

export default async function AdminPage() {
  const user = await requireAdminUser();

  const response = await fetch(`${serverApiBaseUrl}/api/admin`, {
    headers: { cookie: cookies().toString() },
    cache: "no-store",
  });

  let message = "Admin route loaded";
  if (response.ok) {
    const body = (await response.json()) as { message?: string };
    message = body.message ?? message;
  }

  return (
    <section className={`mx-auto max-w-3xl ${UI_CARD_LG}`}>
      <p className={UI_KICKER}>Admin</p>
      <h1 className={`mt-2 ${UI_TITLE_LG}`}>bominal admin stub</h1>
      <p className="mt-3 text-slate-600">Signed in as: {user.email}</p>
      <p className="mt-1 text-slate-600">API says: {message}</p>
    </section>
  );
}
