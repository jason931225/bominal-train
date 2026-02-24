import { PaymentSettingsPanel } from "@/components/wallet/payment-settings-panel";
import { requireApprovedUser } from "@/lib/server-auth";

export default async function PaymentSettingsPage() {
  await requireApprovedUser();

  return (
    <section>
      <PaymentSettingsPanel />
    </section>
  );
}
