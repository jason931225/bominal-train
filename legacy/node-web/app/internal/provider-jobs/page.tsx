import { CredentialsForm } from "@/components/internal/credentials-form";
import { PaymentMethodForm } from "@/components/internal/payment-method-form";
import { ProviderJobConsole } from "@/components/internal/provider-job-console";
import { requireApprovedUser } from "@/lib/server-auth";

export default async function InternalProviderJobsPage() {
  await requireApprovedUser();

  return (
    <section className="space-y-6">
      <header>
        <p className="text-xs uppercase tracking-[0.16em] text-blossom-500">bominal internal</p>
        <h1 className="mt-1 text-3xl font-display font-semibold tracking-tight text-slate-900">Provider jobs debug console</h1>
        <p className="mt-2 max-w-3xl text-sm text-slate-600">
          Internal-only skeleton for SRT credentials, Evervault-encrypted payment method submission, and provider-job controls.
        </p>
      </header>

      <div className="grid gap-6 xl:grid-cols-2">
        <CredentialsForm />
        <PaymentMethodForm />
      </div>

      <ProviderJobConsole />
    </section>
  );
}
