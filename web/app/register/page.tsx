import Link from "next/link";
import { redirect } from "next/navigation";

import { RegisterForm } from "@/components/register-form";
import { UI_BODY_MUTED, UI_CARD_LG, UI_TITLE_LG } from "@/lib/ui";
import { getOptionalUser } from "@/lib/server-auth";

export default async function RegisterPage() {
  const user = await getOptionalUser();
  if (user) {
    redirect("/dashboard");
  }

  return (
    <section className={`mx-auto w-full max-w-md ${UI_CARD_LG}`}>
      <h1 className={UI_TITLE_LG}>Create your bominal account</h1>
      <p className={`mt-2 ${UI_BODY_MUTED}`}>Email and password authentication, ready for modular growth.</p>

      <div className="mt-6">
        <RegisterForm />
      </div>

      <p className="mt-6 text-sm text-slate-600">
        Already have an account?{" "}
        <Link href="/login" className="font-medium text-blossom-600 hover:text-blossom-700">
          Sign in
        </Link>
      </p>
    </section>
  );
}
