import { PaymentSettingsPanel } from "@/components/wallet/payment-settings-panel";
import { requireUser } from "@/lib/server-auth";

export default async function PaymentSettingsPage() {
  await requireUser();

  return (
    <section>
      <PaymentSettingsPanel />
    </section>
  );
}
