import { AccountSettingsPanel } from "@/components/account/account-settings-panel";
import { requireUser } from "@/lib/server-auth";

export default async function AccountSettingsPage() {
  const user = await requireUser();
  return <AccountSettingsPanel initialUser={user} />;
}
