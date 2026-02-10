import { redirect } from "next/navigation";

import { LandingHeroVideo } from "@/components/landing/landing-hero-video";
import { ROUTES } from "@/lib/routes";
import { getOptionalUser } from "@/lib/server-auth";

export default async function HomePage() {
  const user = await getOptionalUser();
  if (user) {
    redirect(ROUTES.dashboard);
  }

  return <LandingHeroVideo />;
}

