import Link from "next/link";
import { redirect } from "next/navigation";

import { getOptionalUser } from "@/lib/server-auth";
import { UI_BODY_MUTED, UI_BUTTON_OUTLINE, UI_BUTTON_PRIMARY, UI_CARD_LG, UI_KICKER, UI_TITLE_LG } from "@/lib/ui";

export default async function HomePage() {
  const user = await getOptionalUser();
  if (user) {
    redirect("/dashboard");
  }

  return (
    <div className="space-y-10">
      <section className="relative overflow-hidden rounded-3xl border border-blossom-100 bg-white/75 shadow-petal">
        <div className="pointer-events-none absolute inset-0">
          <div className="absolute -left-24 -top-24 h-72 w-72 rounded-full bg-blossom-200/55 blur-3xl" />
          <div className="absolute -bottom-24 -right-24 h-72 w-72 rounded-full bg-blossom-100/70 blur-3xl" />
          <div className="absolute inset-0 bg-[linear-gradient(to_right,rgba(15,23,42,0.03)_1px,transparent_1px),linear-gradient(to_bottom,rgba(15,23,42,0.03)_1px,transparent_1px)] bg-[size:28px_28px]" />
        </div>

        <div className="relative p-8 sm:p-12">
          <p className={UI_KICKER}>Modular platform</p>
          <h1 className={`mt-3 ${UI_TITLE_LG}`}>
            bominal helps you automate train searches and keep your workflow tidy.
          </h1>
          <p className={`mt-4 max-w-2xl ${UI_BODY_MUTED}`}>
            A lightweight dashboard with secure session auth, background jobs, and a Train module that runs in KST.
          </p>

          <div className="mt-7 flex flex-wrap gap-3">
            <Link href="/register" className={UI_BUTTON_PRIMARY}>
              Create account
            </Link>
            <Link href="/login" className={UI_BUTTON_OUTLINE}>
              Sign in
            </Link>
          </div>
        </div>
      </section>

      <section className="grid gap-4 md:grid-cols-3">
        <div className={UI_CARD_LG}>
          <p className={UI_KICKER}>Train</p>
          <h2 className="mt-2 font-display text-xl font-semibold tracking-tight text-slate-900">Queue-backed tasks</h2>
          <p className={`mt-2 ${UI_BODY_MUTED}`}>
            Create Tasks that poll providers in the background and keep a detailed attempt timeline.
          </p>
        </div>
        <div className={UI_CARD_LG}>
          <p className={UI_KICKER}>Security</p>
          <h2 className="mt-2 font-display text-xl font-semibold tracking-tight text-slate-900">Encrypted secrets</h2>
          <p className={`mt-2 ${UI_BODY_MUTED}`}>
            Provider credentials are stored using envelope encryption, with safe metadata patterns throughout the app.
          </p>
        </div>
        <div className={UI_CARD_LG}>
          <p className={UI_KICKER}>Ops</p>
          <h2 className="mt-2 font-display text-xl font-semibold tracking-tight text-slate-900">Deploy-friendly</h2>
          <p className={`mt-2 ${UI_BODY_MUTED}`}>
            Built to run on a single small VM with Docker Compose, health checks, and a minimal admin dashboard.
          </p>
        </div>
      </section>
    </div>
  );
}
