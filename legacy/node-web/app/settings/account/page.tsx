import { AccountSettingsPanel } from "@/components/account/account-settings-panel";
import { requireUser } from "@/lib/server-auth";

export default async function AccountSettingsPage({
  searchParams,
}: {
  searchParams?: Promise<{ email?: string; code?: string; email_change?: string }>;
}) {
  const user = await requireUser();
  const resolved = (await searchParams) ?? {};
  const prefill =
    resolved.email_change === "1" && resolved.email && resolved.code
      ? { email: resolved.email, code: resolved.code }
      : null;
  return <AccountSettingsPanel initialUser={user} prefillEmailChange={prefill} />;
}
